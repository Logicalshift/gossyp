//!
//! The script interpreter runs scripts directly from their parse tree. This is
//! a fairly slow way to run scripts but comparatively simple to implement.
//!

use std::result::Result;
use serde_json::*;

use gossyp_base::{Tool, Environment};
use gossyp_base::basic::{make_dynamic_tool};

use super::script::Script;
use super::evaluate_statement::evaluate_statement;
use super::bind_statement::bind_statement;
use super::binding_environment::BindingEnvironment;

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
    FieldMustBeIdentifier,

    /// Tried to declare a new variable with let or var which is already in use
    VariableNameAlreadyInUse
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
        // Bind the values contained within the script
        let mut binding_environment = BindingEnvironment::new(environment);
        let bound_script            = bind_statement(&self.statements, &mut *binding_environment)?;

        // Execute the script
        let mut script_environment = ScriptExecutionEnvironment::new();

        // Evaluate them
        evaluate_statement(&bound_script, environment, &mut script_environment)
    }
}

///
/// Represents an execution environment for a running script
///
pub struct ScriptExecutionEnvironment {
    /// Current values of the variables in this environment
    variable_values: Vec<Box<Value>>,
}

impl ScriptExecutionEnvironment {
    ///
    /// Creates a new script execution environment
    ///
    pub fn new() -> ScriptExecutionEnvironment {
        ScriptExecutionEnvironment { variable_values: vec![] }
    }

    ///
    /// Allocates variables in this environment
    ///
    #[inline]
    pub fn allocate_variables(&mut self, num_variables: u32) {
        // Just create any new variables with null values
        while self.variable_values.len() < num_variables as usize {
            self.variable_values.push(Box::new(Value::Null));
        }
    }

    ///
    /// Sets a variable to a value
    ///
    #[inline]
    pub fn set_variable(&mut self, pos: u32, value: Box<Value>) {
        // Trying to set a variable that has not been allocated is a no-op
        if (pos as usize) < self.variable_values.len() {
            self.variable_values[pos as usize] = value;
        }
    }

    ///
    /// Sets a variable to a value
    ///
    #[inline]
    pub fn get_variable(&self, pos: u32) -> &Value {
        &*self.variable_values[pos as usize]
    }
}
