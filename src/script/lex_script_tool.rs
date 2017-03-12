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

#[cfg(test)]
mod test {
    use std::error::Error;
    use super::*;

    #[test]
    fn can_parse_syntax_json() {
        let script_json = from_str::<Value>(include_str!("syntax_lexer.json"));

        if script_json.is_err() {
            println!("{:?}", script_json);
            println!("{:?}", script_json.unwrap_err().description());

            assert!(false);
        }
    }

    #[test]
    fn json_can_be_deserialized() {
        let script_json = from_str::<Vec<LexToolSymbol>>(include_str!("syntax_lexer.json"));

        if script_json.is_err() {
            println!("{:?}", script_json);
        }

        script_json.unwrap();
    }

    #[test]
    fn can_create_tool() {
        let _tool = create_lex_script_tool();
    }
}