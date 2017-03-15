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
/// Looks ahead to the next syntactically relevant lexer match
///
fn lookahead<'a>(input: &'a [ScriptToken]) -> (Option<ScriptToken>, &'a [ScriptToken]) {
    unimplemented!();
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