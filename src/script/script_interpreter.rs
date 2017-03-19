//!
//! The script interpreter runs scripts directly from their parse tree. This is
//! a fairly slow way to run scripts but comparatively simple to implement.
//!

use std::result::Result;
use serde_json::*;

use silkthread_base::*;

use super::script::*;

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
}

impl Tool for InterpretedScriptTool {
    fn invoke_json(&self, input: Value, environment: &Environment) -> Result<Value, Value> {
        unimplemented!()
    }
}
