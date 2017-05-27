//!
//! The script interpreter runs scripts directly from their parse tree. This is
//! a fairly slow way to run scripts but comparatively simple to implement.
//!

use std::result::Result;
use serde_json::*;

use gossyp_base::*;
use gossyp_base::basic::*;

use super::script::*;
use super::evaluate_statement::*;
use super::bind_statement::*;
use super::bound_script::*;

///
/// A tool representing a script that will be interepreted
///
pub struct InterpretedScriptTool {
    statements: Script
}

///
/// Script evaluation error
///
#[derive(Serialize, Deserialize)]
pub enum ScriptEvaluationError {
    /// Tried to evaluate an expression type that's not implemented yet
    ExpressionNotImplemented,

    /// Tried to evaluate a statement type that's not implemented yet
    StatementNotImplemented,

    /// Tried to look up a tool and it couldn't be found
    ToolNameNotFound,

    /// Found an expression that can't be treated as a tool where a tool name was expected
    ExpressionDoesNotEvaluateToTool,

    /// Expressions used as keys in a map must evaluate to a string
    MapKeysMustEvaluateToAString,

    /// In index expression like foo[bar], foo must be either an array, a string or a map
    IndexMustApplyToAnArrayOrAMap,

    /// When indexing an array or a string, the index must be a number
    ArrayIndexMustBeANumber,

    /// When indexing a map, the index must be a string
    MapIndexMustBeAString,

    /// When indexing an array, the index must be in the array bounds
    IndexOutOfBounds,

    /// Object value is not present in a map
    ObjectValueNotPresent,

    /// In a field access (a.b), the '.b' part must be an identifier
    FieldMustBeIdentifier
}

impl InterpretedScriptTool {
    ///
    /// Creates a tool that can evaluate a script
    ///
    pub fn new_script_eval_tool() -> Box<Tool> {
        Box::new(make_dynamic_tool(|script: Vec<Script>, environment: &Environment| {
            let script_tool = InterpretedScriptTool::from_statements(script);

            script_tool.invoke_json(Value::Null, environment)
        }))
    }

    ///
    /// Creates a new interpreted script tool from a set of statements
    ///
    pub fn from_statements(statements: Vec<Script>) -> InterpretedScriptTool {
        InterpretedScriptTool { statements: Script::Sequence(statements) }
    }
}

impl Tool for InterpretedScriptTool {
    fn invoke_json(&self, _input: Value, environment: &Environment) -> Result<Value, Value> {
        // Make the environment that this script will run in
        let mut script_environment = ScriptExecutionEnvironment::new(environment);

        // Bind the values contained within the script
        let bound_script = bind_statement(&self.statements, &mut script_environment)?;

        // Evaluate them
        evaluate_statement(&bound_script, &mut script_environment)
    }
}

///
/// Represents an execution environment for a running script
///
pub struct ScriptExecutionEnvironment<'a> {
    /// Current values of the variables in this environment
    variable_values: Vec<Value>,

    /// The environment where tools are drawn from
    parent_environment: &'a Environment
}

impl<'a> ScriptExecutionEnvironment<'a> {
    ///
    /// Creates a new script execution environment
    ///
    pub fn new(parent_environment: &'a Environment) -> ScriptExecutionEnvironment<'a> {
        ScriptExecutionEnvironment { variable_values: vec![], parent_environment: parent_environment }
    }

    ///
    /// Fetches a tool from this environment
    ///
    #[inline]
    pub fn get_json_tool(&self, tool: &str) -> Result<Box<Tool>, RetrieveToolError> {
        self.parent_environment.get_json_tool(tool)
    }

    ///
    /// Invokes a JSON tool in this environment
    ///
    #[inline]
    pub fn invoke_tool(&self, tool: &Box<Tool>, input: Value) -> Result<Value, Value> {
        tool.invoke_json(input, self.parent_environment)
    }
}
