use std::sync::{Mutex, Arc};
use std::result::Result;

use serde_json::Value;
use gossyp_base::environment::Environment;

use super::binding_environment::*;
use super::script_interpreter::*;
use super::bind_statement::*;
use super::evaluate_statement::*;
use super::script::*;
use super::bound_script::*;

///
/// Represents a tool that can be used to evaluate scripts and maintains
/// state (useful for evaluation in a REPL environment)
///
pub struct StatefulEvalTool {
    binding:    Arc<Mutex<BindingEnvironment>>,
    execution:  Arc<Mutex<ScriptExecutionEnvironment>>
}

impl StatefulEvalTool {
    ///
    /// Evaluates an unbound statement using this tool
    ///
    pub fn evaluate_unbound_statement(&self, script: &Script, environment: &Environment) -> Result<Value, Value> {
        self.evaluate_statement(&self.bind_statement(script)?, environment)
    }

    ///
    /// Binds a statement to this tool
    ///
    pub fn bind_statement(&self, script: &Script) -> Result<BoundScript, Value> {
        bind_statement(script, &mut *self.binding.lock().unwrap())
    }

    ///
    /// Evaluates a statement in the environment represented by this tool
    ///
    pub fn evaluate_statement(&self, script: &BoundScript, environment: &Environment) -> Result<Value, Value> {
        evaluate_statement(script, environment, &mut *self.execution.lock().unwrap())
    }
}
