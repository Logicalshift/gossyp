use std::result::Result;

use serde_json::*;
use gossyp_base::Environment;

use super::script::*;
use super::lex_script_tool::*;
use super::parse_script_tool::*;
use super::binding_environment::*;
use super::bind_statement::*;
use super::evaluate_statement::*;
use super::script_interpreter::*;

///
/// Evaluates a simple gossyp script with an environment
///
pub fn gossyp_eval(script: &str, environment: &Environment) -> Result<Value, Value> {
    // Parse the script
    let lexed   = create_lex_script_tool().lex(script);
    let parsed  = ParseScriptTool::parse(&lexed).map_err(|parse_error| to_value(parse_error).unwrap())?;

    // Bind it
    let bound   = {
        let mut binding = BindingEnvironment::from_environment(environment);
        bind_statement(&Script::Sequence(parsed), &mut *binding)?
    };

    // Execute it
    let mut execution_environment = ScriptExecutionEnvironment::new();
    evaluate_statement(&bound, environment, &mut execution_environment)
}

#[cfg(test)]
mod test {
    use super::*;
    use gossyp_base::basic::*;

    #[test]
    fn can_evaluate_string() {
        let env = DynamicEnvironment::new();
        assert!(define_pure_tool(&env, "add_one", |x: i32| x+1).is_ok());

        assert!(gossyp_eval("add_one 1", &env) == Ok(json![vec![2]]));
    }
}