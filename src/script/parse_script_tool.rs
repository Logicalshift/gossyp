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

impl ParseError {
    pub fn new() -> ParseError {
        ParseError { }
    }
}

struct ParseState<'a> {
    remaining: &'a [ScriptToken]
}

impl<'a> ParseState<'a> {
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
    fn lookahead(&self) -> Option<(&'a ScriptToken, &'a [ScriptToken])> {
        let mut index   = 0;
        let len         = self.remaining.len();

        loop {
            if index >= len {
                return None;
            } else if ParseState::is_syntax(&self.remaining[index]) {
                let token       = &self.remaining[index];
                let remaining   = &self.remaining[index+1..len];
                return Some((token, remaining));
            }

            index += 1;
        }
    }

    ///
    /// If the next token matches the specified token, consumes it and returns
    /// its content.
    ///
    fn accept(&mut self, token: ScriptLexerToken) -> Option<&'a ScriptToken> {
        if let Some((lookahead, remaining)) = self.lookahead() {
            // Found a token
            if lookahead.token == token {
                self.remaining = remaining;
                Some(lookahead)
            } else {
                None
            }
        } else {
            // Reached the end of file
            None
        }
    }

    /*

    ///
    /// Parses a statement
    ///
    pub fn parse_statement<'a>(input: &'a [ScriptToken]) -> Result<(Script, &'a [ScriptToken]), ParseError> {
        if let Some((lookahead, remainder)) = lookahead(input) {
            match lookahead.token {
                // Newlines are ignored
                ScriptLexerToken::Newline       => parse_statement(remainder),

                ScriptLexerToken::Let           => parse_let(remainder),
                ScriptLexerToken::Var           => parse_var(remainder),
                ScriptLexerToken::Def           => parse_def(remainder),
                ScriptLexerToken::If            => parse_if(remainder),
                ScriptLexerToken::Using         => parse_using(remainder),
                ScriptLexerToken::While         => parse_while(remainder),
                ScriptLexerToken::Loop          => parse_loop(remainder),
                ScriptLexerToken::For           => parse_for(remainder),

                ScriptLexerToken::Identifier    => parse_command(input),

                // Unrecognised token
                _ => Err(ParseError::new())
            }
        } else {
            // EOF
            Err(ParseError::new())
        }
    }

    ///
    /// Parses a command
    ///
    /// Syntax '<expression>', '<expression> <expression>'
    ///
    pub fn parse_command<'a>(input: &'a [ScriptToken]) -> Result<(Script, &'a [ScriptToken]), ParseError> {
        unimplemented!();
    }

    ///
    /// Parses an Expression
    ///
    pub fn parse_expression<'a>(input: &'a [ScriptToken]) -> Result<(Expression, &'a [ScriptToken]), ParseError> {
        if let Some((lookahead1, remainder1)) = lookahead(input) {
            let expr1 = match lookahead1.token {
                ScriptLexerToken::Identifier    => Ok((Expression::Identifier(lookahead1.clone()), remainder)),
                ScriptLexerToken::Number        |
                ScriptLexerToken::HexNumber     => Ok((Expression::Number(lookahead1.clone()), remainder)),
                ScriptLexerToken::String        => Ok((Expression::String(lookahead1.clone()), remainder)),

                _ => Err(ParseError::new())
            };

            expr1
        } else {
            // EOF
            Err(ParseError::new())
        }
    }

    pub fn parse_let<'a>(input: &'a [ScriptToken]) -> Result<(Script, &'a [ScriptToken]), ParseError> {
        unimplemented!()
    }

    pub fn parse_var<'a>(input: &'a [ScriptToken]) -> Result<(Script, &'a [ScriptToken]), ParseError> {
        unimplemented!()
    }

    pub fn parse_def<'a>(input: &'a [ScriptToken]) -> Result<(Script, &'a [ScriptToken]), ParseError> {
        unimplemented!()
    }

    pub fn parse_if<'a>(input: &'a [ScriptToken]) -> Result<(Script, &'a [ScriptToken]), ParseError> {
        unimplemented!()
    }

    pub fn parse_using<'a>(input: &'a [ScriptToken]) -> Result<(Script, &'a [ScriptToken]), ParseError> {
        unimplemented!()
    }

    pub fn parse_while<'a>(input: &'a [ScriptToken]) -> Result<(Script, &'a [ScriptToken]), ParseError> {
        unimplemented!()
    }

    pub fn parse_loop<'a>(input: &'a [ScriptToken]) -> Result<(Script, &'a [ScriptToken]), ParseError> {
        unimplemented!()
    }

    pub fn parse_for<'a>(input: &'a [ScriptToken]) -> Result<(Script, &'a [ScriptToken]), ParseError> {
        unimplemented!()
    }
    */
}

impl ParseScriptTool {
    ///
    /// Creates a new script parsing tool
    ///
    pub fn new() -> ParseScriptTool {
        ParseScriptTool { }
    }
}