//!
//! The lex tool generates lexer tools from its input
//!

use std::result::Result;
use serde::*;
use serde_json::*;
use silkthread_base::*;

///
/// Input for the lexer tool
///
#[derive(Serialize, Deserialize)]
pub struct LexToolInput {
    /// Name of the tool that the lexer will define
    pub new_tool_name:  String,

    /// The symbols that the lexer will match
    pub symbols:        LexToolSymbol
}

///
/// Lexer symbol
///
#[derive(Serialize, Deserialize)]
pub struct LexToolSymbol {
    /// The name of the symbol that will be generated if this match is made
    pub symbol_name:    String,

    /// The rule that will be matched against this symbol
    pub match_rule:     String
}

///
/// Lexer generation tool
///
pub struct LexTool {
}

impl LexTool {
    pub fn new() -> LexTool {
        LexTool { }
    } 
}

impl Tool for LexTool {
    fn invoke_json(&self, input: Value, environment: &Environment) -> Result<Value, Value> {
        unimplemented!();
    }
}
