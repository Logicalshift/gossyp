use std::result::Result;

use serde_json::*;

use super::bound_script::*;
use super::evaluate_expression::*;
use super::binding_environment::*;
use super::bind_statement::*;
use super::script::*;
use super::script_interpreter::*;

///
/// Creates an execution error
///
fn generate_script_error(error: ScriptEvaluationError, script: &BoundScript) -> Value {
    json![{
        "error":                error
        /* "failed-statement":     script (TODO!) */
    }]
}

///
/// Evaluates the result of executing a sequence of steps
///
pub fn evaluate_sequence(sequence: &Vec<BoundScript>, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    // Execute the script
    let mut result = vec![];
    for statement in sequence.iter() {
        // Evaluate the next statement
        let next_result = evaluate_statement(statement, environment)?;

        // The script result is built up from the result of each statement
        // TODO: unless there's something like a return statement?
        result.push(next_result);
    }

    // Script is done
    Ok(Value::Array(result))
}

///
/// Evaluates the result of executing a single statement
///
pub fn evaluate_statement(statement: &BoundScript, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    match statement {
        &BoundScript::RunCommand(ref expr)  => evaluate_expression(expr, environment),
        &BoundScript::Sequence(ref steps)   => evaluate_sequence(steps, environment),

        _                                   => Err(generate_script_error(ScriptEvaluationError::StatementNotImplemented, statement))
    }
}

///
/// Evaluates the result of executing a single statement
///
pub fn evaluate_unbound_statement(statement: &Script, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    let bound = {
        let mut binding_environment = BindingEnvironment::new(environment.get_environment());
        bind_statement(statement, &mut *binding_environment)?
    };

    evaluate_statement(&bound, environment)
}

#[cfg(test)]
mod test {
    use gossyp_base::basic::*;
    use super::*;

    #[test]
    fn can_execute_run_command() {
        let tool_expr           = Script::RunCommand(Expression::identifier("test"));
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = evaluate_unbound_statement(&tool_expr, &mut env);

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

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = evaluate_unbound_statement(&tool_expr, &mut env);

        assert!(result == Ok(Value::Array(vec![
            Value::String(String::from("test 1")),
            Value::String(String::from("test 2"))
        ])));
    }
}
