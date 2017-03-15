use std::result::Result;

use super::script::*;

///
/// Represents a parse error
///
pub struct ParseError {

}

///
/// Tool that parses our scripting language
///
pub struct ParseScriptTool {
}

///
/// Returns true if a token is considered syntax (gets returned from lookahead)
///
fn is_syntax(token: &ScriptToken) -> bool {
    match token.token {
        ScriptLexerToken::Whitespace    |
        ScriptLexerToken::Comment       => false,

        _                               => true
    }
}

///
/// Looks ahead to the next syntactically relevant lexer match (and returns the tokens after it)
///
fn lookahead<'a>(input: &'a [ScriptToken]) -> Option<(&'a ScriptToken, &'a [ScriptToken])> {
    let mut index   = 0;
    let len         = input.len();

    loop {
        if index >= len {
            return None;
        } else if is_syntax(&input[index]) {
            return Some((&input[index], &input[index+1..len]));
        }

        index += 1;
    }
}

///
/// Parses a statement
///
pub fn parse_statement<'a>(input: &'a [ScriptToken]) -> Result<(Script, &'a [ScriptToken]), ParseError> {
    unimplemented!();
}

///
/// Parses a command
///
/// Syntax '<expression>', '<expression> <expression>'
///
pub fn parse_command<'a>(input: &'a [ScriptToken]) -> Result<(Script, &'a [ScriptToken]), ParseError> {
    unimplemented!();
}

impl ParseScriptTool {
    ///
    /// Creates a new script parsing tool
    ///
    pub fn new() -> ParseScriptTool {
        ParseScriptTool { }
    }
}