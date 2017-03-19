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

    fn lex_tokens(input: &str) -> Vec<String> {
        let lex_tool = create_lex_script_tool();

        lex_tool
            .lex(input)
            .iter()
            .map(|x| x.token.clone())
            .collect()
    }

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

    #[test]
    fn can_lex_identifier() {
        assert!(lex_tokens("something") == vec![ String::from("Identifier") ]);
    }

    #[test]
    fn can_lex_identifier_with_hyphen() {
        assert!(lex_tokens("something-something") == vec![ String::from("Identifier") ]);
    }

    #[test]
    fn can_lex_let_keyword() {
        assert!(lex_tokens("let") == vec![ String::from("let") ]);
    }

    #[test]
    fn can_lex_whitespace() {
        assert!(lex_tokens(" ") == vec![ String::from("Whitespace") ]);
    }

    #[test]
    fn can_lex_plus_symbol() {
        assert!(lex_tokens("+") == vec![ String::from("+") ]);
    }

    #[test]
    fn can_lex_dot_symbol() {
        assert!(lex_tokens(".") == vec![ String::from(".") ]);
    }

    #[test]
    fn can_lex_newline() {
        assert!(lex_tokens("\n") == vec![ String::from("Newline") ]);
    }

    #[test]
    fn can_lex_simple_number() {
        assert!(lex_tokens("1") == vec![ String::from("Number") ]);
    }

    #[test]
    fn can_lex_simple_string() {
        assert!(lex_tokens("\"Foo\"") == vec![ String::from("String") ]);
    }

    #[test]
    fn can_lex_two_strings() {
        assert!(lex_tokens("\"Foo\"\"Bar\"") == vec![ String::from("String"), String::from("String") ]);
    }

    #[test]
    fn can_lex_longer_number() {
        assert!(lex_tokens("123") == vec![ String::from("Number") ]);
    }

    #[test]
    fn can_lex_decimal_number() {
        assert!(lex_tokens("1.21") == vec![ String::from("Number") ]);
    }

    #[test]
    fn can_lex_decimal_with_exponent() {
        assert!(lex_tokens("1.2e10") == vec![ String::from("Number") ]);
    }

    #[test]
    fn can_lex_integer_with_exponent() {
        assert!(lex_tokens("1e10") == vec![ String::from("Number") ]);
    }

    #[test]
    fn can_lex_decimal_number_beginning_with_dot() {
        assert!(lex_tokens(".21") == vec![ String::from("Number") ]);
    }
}
