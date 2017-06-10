pub mod lex_script_tool;
pub mod parse_script_tool;
pub mod script;
pub mod bound_script;
pub mod script_interpreter;
pub mod stateful_eval;
pub mod binding_environment;
pub mod bind_expression;
pub mod bind_statement;
pub mod evaluate_statement;
pub mod evaluate_expression;
pub mod tool;
pub mod evaluate;

use self::lex_script_tool::*;
use self::parse_script_tool::*;
use self::script_interpreter::*;
use self::stateful_eval::*;
use gossyp_base::*;
use gossyp_base::basic::*;

pub use self::evaluate::*;

///
/// ToolSet for dealing with the scripting language
///
pub struct ScriptTools {
}

impl ToolSet for ScriptTools {
    fn create_tools(self, _: &Environment) -> Vec<(String, Box<Tool>)> {
        vec![
            (String::from(tool::LEX_SCRIPT),                    Box::new(create_lex_script_tool())),
            (String::from(tool::PARSE_SCRIPT),                  ParseScriptTool::new_tool()),
            (String::from(tool::EVAL_SCRIPT),                   InterpretedScriptTool::new_script_eval_tool()),
            (String::from(tool::CREATE_EVALUATOR_WITH_STATE),   Box::new(make_dynamic_tool(create_evaluator_with_state_tool)))
        ]
    }
}

impl ScriptTools {
    ///
    /// Creates a new script toolset
    ///
    pub fn new() -> ScriptTools {
        ScriptTools { }
    }
}
