pub mod lex_script_tool;
pub mod parse_script_tool;
pub mod script;
pub mod bound_script;
pub mod script_interpreter;
pub mod bind_expression;
pub mod evaluate_statement;
pub mod evaluate_expression;
pub mod tool;

use self::lex_script_tool::*;
use self::parse_script_tool::*;
use self::script_interpreter::*;
use gossyp_base::*;
use gossyp_base::basic::*;

///
/// ToolSet for dealing with the scripting language
///
pub struct ScriptTools {
}

impl ToolSet for ScriptTools{
    fn create_tools(self, _: &Environment) -> Vec<(String, Box<Tool>)> {
        vec![
            (String::from(tool::LEX_SCRIPT),    Box::new(create_lex_script_tool())),
            (String::from(tool::PARSE_SCRIPT),  ParseScriptTool::new_tool()),
            (String::from(tool::EVAL_SCRIPT),   InterpretedScriptTool::new_script_eval_tool())
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
