use serde_json::*;

use super::super::lex::*;

///
/// Creates a lexing tool for the scripting language
///
pub fn create_lex_script_tool() -> StringLexingTool {
    // Parse the lexer
    let script_json = from_str::<Vec<LexToolSymbol>>(include_str!("syntax_lexer.json")).unwrap();

    // The name isn't used here, but define it anyway
    let lex_defn = LexToolInput { 
        new_tool_name:  String::from("lex-script"),
        symbols:        script_json
    };

    // Create the lexing tool with this definition
    StringLexingTool::from_lex_tool_input(&lex_defn)
}
