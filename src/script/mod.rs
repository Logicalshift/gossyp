pub mod lex_script_tool;
pub mod tool;

use self::lex_script_tool::*;
use silkthread_base::*;
use silkthread_base::basic::*;

///
/// ToolSet for dealing with the scripting language
///
pub struct ScriptToolSet {
}

impl ToolSet for ScriptToolSet {
    fn create_tools(self, _: &Environment) -> Vec<(String, Box<Tool>)> {
        vec![
            (String::from(tool::LEX_SCRIPT), Box::new(create_lex_script_tool()))
        ]
    }
}

impl ScriptToolSet {
    ///
    /// Creates a new script toolset
    ///
    pub fn new() -> ScriptToolSet {
        ScriptToolSet { }
    }
}
