use std::result::Result;

use serde_json::*;

use super::evaluate_expression::*;
use super::script::*;
use super::script_interpreter::*;

///
/// Creates an execution error
///
fn generate_script_error(error: ScriptEvaluationError, script: &Script) -> Value {
    json![{
        "error":                error,
        "failed-statement":     script
    }]
}

///
/// Evaluates the result of executing a single statement
///
pub fn evaluate_statement(statement: &Script, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    match statement {
        &Script::RunCommand(ref expr)   => evaluate_expression(expr, environment),

        _                               => Err(generate_script_error(ScriptEvaluationError::StatementNotImplemented, statement))
    }
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
        let result              = evaluate_statement(&tool_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Success"))));
    }
}
