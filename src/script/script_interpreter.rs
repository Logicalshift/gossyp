//!
//! The script interpreter runs scripts directly from their parse tree. This is
//! a fairly slow way to run scripts but comparatively simple to implement.
//!

use std::result::Result;
use serde_json::*;

use silkthread_base::*;

use super::script::*;

///
/// A tool representing a script that will be interepreted
///
pub struct InterpretedScriptTool {
    statements: Vec<Script>
}

///
/// Creates an unquoted version of a string
///
fn unquote_string(string: &str) -> String {
    let chars: Vec<char>    = string.chars().collect();
    let mut result          = String::new();
    let mut index           = 1;
    while index < chars.len()-1 {
        // Push character
        let chr = chars[index];

        match chr {
            '\\' => { 
                let quoted = chars[index+1];
                match quoted {
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    quoted => result.push(quoted)
                }
            },
            chr => result.push(chr)
        }

        // Next character
        index += 1;
    }

    result
}

///
/// Script evaluation error
///
#[derive(Serialize, Deserialize)]
pub enum ScriptEvaluationError {
    /// Tried to evaluate an expression type that's not implemented yet
    ExpressionNotImplemented,

    /// Tried to look up a tool and it couldn't be found
    ToolNameNotFound,

    /// Found an expression that can't be treated as a tool where a tool name was expected
    ExpressionDoesNotEvaluateToTool
}

///
/// Creates an execution error
///
fn generate_expression_error(error: ScriptEvaluationError, expr: &Expression) -> Value {
    json![{
        "error":                error,
        "failed-expression":    expr
    }]
}

impl InterpretedScriptTool {
    ///
    /// Creates a new interpreted script tool from a set of statements
    ///
    pub fn from_statements(statements: Vec<Script>) -> InterpretedScriptTool {
        InterpretedScriptTool { statements: statements }
    }

    ///
    /// Attempts to evaluate an expression to a tool
    ///
    pub fn evaluate_expression_to_tool(expression: &Expression, environment: &mut ScriptExecutionEnvironment) -> Result<Box<Tool>, Value> {
        match expression {
            &Expression::Identifier(ref name)   => environment.parent_environment.get_json_tool(&name.matched).map_err(|_| generate_expression_error(ScriptEvaluationError::ToolNameNotFound, expression)),
            _                                   => Err(generate_expression_error(ScriptEvaluationError::ExpressionDoesNotEvaluateToTool, expression))
        }
    }

    ///
    /// Calls an expression representing a tool and calls it with the specified parameters
    ///
    pub fn call_tool(tool_name: &Expression, parameters: Value, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
        InterpretedScriptTool::evaluate_expression_to_tool(tool_name, environment).and_then(|tool| {
            // TODO: create environment for the tool to run in

            // Invoke the tool and generate the final result
            tool.invoke_json(parameters, environment.parent_environment)
        })
    }

    ///
    /// Evaluates a single expression
    ///
    pub fn evaluate_expression(expression: &Expression, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
        match expression {
            &Expression::String(ref s)          => Ok(Value::String(unquote_string(&s.matched))),

            &Expression::Identifier(_)          => InterpretedScriptTool::call_tool(expression, Value::Null, environment),

            _                                   => Err(generate_expression_error(ScriptEvaluationError::ExpressionNotImplemented, expression))
        }
    }

    ///
    /// Evaluates the result of executing a single statement
    ///
    pub fn evaluate_statement(statement: &Script, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
        unimplemented!()
    }
}

impl Tool for InterpretedScriptTool {
    fn invoke_json(&self, input: Value, environment: &Environment) -> Result<Value, Value> {
        // Make the environment that this script will run in
        let mut script_environment = ScriptExecutionEnvironment::new(environment);

        // Execute the script
        let mut result = vec![];
        for statement in self.statements.iter() {
            // Evaluate the next statement
            let next_result = match Self::evaluate_statement(statement, &mut script_environment) {
                Ok(result) => result,

                // Fail immediately if any statement generates an error
                Err(fail) => return Err(fail)
            };

            // The script result is built up from the result of each statement
            // TODO: unless there's something like a return statement?
            result.push(next_result);
        }

        // Script is done
        Ok(Value::Array(result))
    }
}

///
/// Represents an execution environment for a running script
///
pub struct ScriptExecutionEnvironment<'a> {
    /// The environment where tools are drawn from
    parent_environment: &'a Environment
}

impl<'a> ScriptExecutionEnvironment<'a> {
    ///
    /// Creates a new script execution environment
    ///
    pub fn new(parent_environment: &'a Environment) -> ScriptExecutionEnvironment<'a> {
        ScriptExecutionEnvironment { parent_environment: parent_environment }
    }
}

#[cfg(test)]
mod test {
    use silkthread_base::basic::*;
    use super::*;

    #[test]
    fn can_evaluate_string() {
        let string_expr         = Expression::string("\"Foo\"");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&string_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Foo"))));
    }

    #[test]
    fn can_call_tool() {
        let tool_expr           = Expression::identifier("test");
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = InterpretedScriptTool::call_tool(&tool_expr, Value::Null, &mut env);

        assert!(result == Ok(Value::String(String::from("Success"))));
    }

    #[test]
    fn can_evaluate_tool_call() {
        let tool_expr           = Expression::identifier("test");
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&tool_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Success"))));
    }
}
