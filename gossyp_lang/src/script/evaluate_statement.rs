use std::result::Result;

use serde_json::*;
use gossyp_base::environment::Environment;

use super::bound_script::*;
use super::evaluate_expression::*;
use super::binding_environment::*;
use super::bind_statement::*;
use super::script::*;
use super::script_interpreter::*;

///
/// Enumeration representing how a failed bound statement can be described
///
#[derive(Deserialize, Serialize, Clone)]
pub enum FailedBoundStatement {
    RunCommand(FailedBoundExpression),
    Sequence(Vec<FailedBoundStatement>),
    Let(ScriptToken),
    Var(ScriptToken),
    Assign(ScriptToken),
    Loop(Box<FailedBoundStatement>),
    While(FailedBoundExpression),
    Using(FailedBoundExpression),
    Def(ScriptToken)
}

///
/// Generates a JS value indicating the statement that failed
///
fn generate_failed_bound_statement(script: &BoundScript) -> FailedBoundStatement {
    use self::FailedBoundStatement::*;

    match script {
        &BoundScript::AllocateVariables(_, ref script)  => generate_failed_bound_statement(&**script),
        &BoundScript::RunCommand(ref expr)              => RunCommand(generate_failed_bound_expression(expr)),
        &BoundScript::Sequence(ref statements)          => Sequence(statements.iter().map(|statement| generate_failed_bound_statement(statement)).collect()),
        &BoundScript::Assign(_, _, ref token)           => Assign(token.clone()),
        &BoundScript::Let(_, _, ref token)              => Let(token.clone()),
        &BoundScript::Var(_, _, ref token)              => Var(token.clone()),
        &BoundScript::Loop(ref loop_box)                => Loop(Box::new(generate_failed_bound_statement(&**loop_box))),
        &BoundScript::While(ref expr, _)                => While(generate_failed_bound_expression(expr)),
        &BoundScript::Using(ref expr, _)                => Using(generate_failed_bound_expression(expr)),
        &BoundScript::Def(ref token, _, _)              => Def(token.clone()),
    }
}

///
/// Creates an execution error
///
fn generate_script_error(error: ScriptEvaluationError, script: &BoundScript) -> Value {
    json![{
        "error":                    error,
        "failed-bound-statement":   generate_failed_bound_statement(script)
    }]
}

///
/// Evaluates the result of executing a sequence of steps
///
pub fn evaluate_sequence(sequence: &Vec<BoundScript>, environment: &Environment, execution_environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    // Execute the script
    let mut result = vec![];
    for statement in sequence.iter() {
        // Evaluate the next statement
        let next_result = evaluate_statement(statement, environment, execution_environment)?;

        // The script result is built up from the result of each statement
        // TODO: unless there's something like a return statement?
        result.push(next_result);
    }

    // Script is done
    Ok(Value::Array(result))
}

///
/// Allocates variables before continuing
///
fn evaluate_allocate_variables(num_variables: u32, continuation: &BoundScript, environment: &Environment, execution_environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    execution_environment.allocate_variables(num_variables);
    evaluate_statement(continuation, environment, execution_environment)
}

///
/// Assigns a value to a particular variable
///
fn evaluate_assignment(variable_index: u32, expr: &BoundExpression, environment: &Environment, execution_environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    let expression_value = evaluate_expression(expr, environment, execution_environment)?;
    execution_environment.set_variable(variable_index, Box::new(expression_value.clone()));

    Ok(expression_value)
}

///
/// Evaluates the result of executing a single statement
///
pub fn evaluate_statement(statement: &BoundScript, environment: &Environment, execution_environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    match statement {
        &BoundScript::AllocateVariables(num, ref continuation)  => evaluate_allocate_variables(num, &**continuation, environment, execution_environment),
        &BoundScript::RunCommand(ref expr)                      => evaluate_expression(expr, environment, execution_environment),
        &BoundScript::Sequence(ref steps)                       => evaluate_sequence(steps, environment, execution_environment),
        &BoundScript::Var(index, ref expr, _)                   => evaluate_assignment(index, expr, environment, execution_environment),
        &BoundScript::Assign(index, ref expr, _)                => evaluate_assignment(index, expr, environment, execution_environment),

        _                                                       => Err(generate_script_error(ScriptEvaluationError::StatementNotImplemented, statement))
    }
}

///
/// Evaluates the result of executing a single statement
///
pub fn evaluate_unbound_statement(statement: &Script, environment: &Environment, execution_environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    let mut binding_environment = BindingEnvironment::from_environment(environment);
    let bound                   = bind_statement(statement, &mut *binding_environment)?;

    evaluate_statement(&bound, environment, execution_environment)
}

#[cfg(test)]
mod test {
    use gossyp_base::basic::*;
    use super::*;
    use super::super::evaluate::*;

    #[test]
    fn can_execute_run_command() {
        let tool_expr           = Script::RunCommand(Expression::identifier("test"));
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new();
        let result              = evaluate_unbound_statement(&tool_expr, &tool_environment, &mut env);

        assert!(result == Ok(Value::String(String::from("Success"))));
    }

    #[test]
    fn can_execute_sequence() {
        let tool_expr           = Script::Sequence(
            vec![
                Script::RunCommand(Expression::string("\"test 1\"")),
                Script::RunCommand(Expression::string("\"test 2\""))
            ]);
        let tool_environment    = DynamicEnvironment::new();

        let mut env             = ScriptExecutionEnvironment::new();
        let result              = evaluate_unbound_statement(&tool_expr, &tool_environment, &mut env);

        assert!(result == Ok(Value::Array(vec![
            Value::String(String::from("test 1")),
            Value::String(String::from("test 2"))
        ])));
    }

    #[test]
    fn commands_have_own_environment() {
        let environment = DynamicEnvironment::new();

        // Tool that defines a new tool in its environment
        assert!(define_dynamic_tool(&environment, "make_tool", |_: (), env| define_pure_tool(env, "subtool", |_: ()| ())).is_ok());

        // Tools should get their own sub-environment so the new tool should not 'escape'. Need to do make_tool().subtool() to call the subtool
        assert!(gossyp_eval("subtool", &environment).is_err());
        assert!(gossyp_eval("make_tool", &environment).is_ok());
        assert!(gossyp_eval("subtool", &environment).is_err());
    }

    #[test]
    fn commands_can_access_parent_environment() {
        let environment = DynamicEnvironment::new();

        // Tool that defines a new tool in its environment
        assert!(define_pure_tool(&environment, "one", |_: ()| 1).is_ok());
        assert!(define_dynamic_tool(&environment, "call_one", |_: (), env| -> Result<Value, Value> {
            Ok(env.get_json_tool("one")
                  .map_err(|_| json![ "Couldn't find tool" ])?
                  .invoke_json(Value::Null, env)?)
            }).is_ok());

        // Tools should get their own sub-environment so the new tool should not 'escape'. Need to do make_tool().subtool() to call the subtool
        assert!(gossyp_eval("one", &environment).is_ok());
        assert!(gossyp_eval("call_one", &environment).map_err(|x| { println!("{:?}", x); x }).is_ok());
    }

    /*
    #[test]
    fn can_call_subtools() {
        let environment = DynamicEnvironment::new();

        // Tool that defines a new tool in its environment
        assert!(define_dynamic_tool(&environment, "make_tool", |_: (), env| define_pure_tool(env, "subtool", |_: ()| ())).is_ok());

        let subtool_result = gossyp_eval("make_tool().subtool()", &environment);
        if !subtool_result.is_ok() { println!("{:?}", subtool_result); }
        assert!(subtool_result.is_ok());
    }
    */
}
