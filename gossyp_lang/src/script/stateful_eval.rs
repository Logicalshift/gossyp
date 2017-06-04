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
#[derive(Clone)]
pub struct StatefulEvalTool {
    binding:    Arc<Mutex<Box<BindingEnvironment>>>,
    execution:  Arc<Mutex<ScriptExecutionEnvironment>>
}

impl StatefulEvalTool {
    pub fn new() -> StatefulEvalTool {
        StatefulEvalTool {
            binding: Arc::new(Mutex::new(BindingEnvironment::new())),
            execution: Arc::new(Mutex::new(ScriptExecutionEnvironment::new()))
        }
    }

    ///
    /// Evaluates an unbound statement using this tool
    ///
    pub fn evaluate_unbound_statement(&self, script: &Script, environment: &Environment) -> Result<Value, Value> {
        self.evaluate_statement(&self.bind_statement(script, environment)?, environment)
    }

    ///
    /// Binds a statement to this tool
    ///
    pub fn bind_statement(&self, script: &Script, environment: &Environment) -> Result<BoundScript, Value> {
        // Merge the stuff in the external environment with the stored environment
        let our_environment             = &mut **self.binding.lock().unwrap();
        let their_environment           = BindingEnvironment::from_environment(environment);
        let mut combined_environment    = BindingEnvironment::combine(our_environment, &*their_environment);

        // Bind to the combined environments
        bind_statement(script, &mut *combined_environment)
    }

    ///
    /// Evaluates a statement in the environment represented by this tool
    ///
    pub fn evaluate_statement(&self, script: &BoundScript, environment: &Environment) -> Result<Value, Value> {
        evaluate_statement(script, environment, &mut *self.execution.lock().unwrap())
    }
}

#[cfg(test)]
mod test {
    use gossyp_base::basic::*;
    use super::*;
    use serde_json::*;

    #[test]
    fn can_bind_variable_using_stateful_tool() {
        let eval    = StatefulEvalTool::new();
        let env     = EmptyEnvironment::new();

        // var x = 1
        let assign_x = eval.evaluate_unbound_statement(&Script::Var(ScriptToken::identifier("x"), Expression::Number(ScriptToken::number("1"))), &env);
        assert!(assign_x.is_ok());

        // x
        let val_of_x = eval.evaluate_unbound_statement(&Script::RunCommand(Expression::Identifier(ScriptToken::identifier("x"))), &env);
        assert!(val_of_x == Ok(json![ 1 ]));
    }

    #[test]
    fn can_bind_tool_from_passed_in_environment() {
        let eval    = StatefulEvalTool::new();
        let env     = DynamicEnvironment::new();

        env.define("test-tool", Box::new(make_pure_tool(|_: ()| 42)));

        // test-tool
        let val_of_test_tool = eval.evaluate_unbound_statement(&Script::RunCommand(Expression::Identifier(ScriptToken::identifier("test-tool"))), &env);
        assert!(val_of_test_tool == Ok(json![ 42 ]));
    }
}
