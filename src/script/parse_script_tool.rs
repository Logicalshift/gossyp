use std::result::Result;

use super::script::*;
use super::super::lex::lex_tool::*;

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
/// Parses a parse_command
///
/// Syntax '<expression>', '<expression> <expression>'
///
pub fn parse_command<'a>(input: &'a [LexerMatch]) -> Result<(Script, &'a [LexerMatch]), ParseError> {
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