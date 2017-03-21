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

impl InterpretedScriptTool {
    ///
    /// Creates a new interpreted script tool from a set of statements
    ///
    pub fn from_statements(statements: Vec<Script>) -> InterpretedScriptTool {
        InterpretedScriptTool { statements: statements }
    }

    ///
    /// Evaluates a single expression
    ///
    pub fn evaluate_expression(expression: &Script, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
        unimplemented!()
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