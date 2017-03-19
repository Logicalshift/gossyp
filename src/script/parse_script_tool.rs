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
    fn lookahead_is(&self, token: ScriptLexerToken) -> bool {
        if let Some((lookahead, remaining)) = self.lookahead() {
            if lookahead.token == token {
                // Token matches
                true
            } else {
                // Token does not match
                false
            }
        } else {
            // End of file
            if token == ScriptLexerToken::EndOfFile {
                // With lookahead_is we can look for the end of file (but we can't accept it because there's no token data associated with it)
                true
            } else {
                false
            }
        }
    }

    ///
    /// If the next token matches the specified token, consumes it and returns
    /// its content.
    ///
    fn accept(&mut self, token: ScriptLexerToken) -> Option<&'a ScriptToken> {
        if let Some((lookahead, remaining)) = self.lookahead() {
            if lookahead.token == token {
                // Token matches: remove it from the input and return it
                self.remaining = remaining;
                Some(lookahead)
            } else {
                // Next token does not match
                None
            }
        } else {
            // Reached the end of file
            None
        }
    }

    ///
    /// Parses a statement
    ///
    pub fn parse_statement(&mut self) -> Result<Script, ParseError> {
        if self.accept(ScriptLexerToken::Newline).is_some() {
            // Newlines are ignored
            self.parse_statement()

        } else if self.accept(ScriptLexerToken::Let).is_some() {
            // let identifier = expression
            self.parse_let()

        } else if self.accept(ScriptLexerToken::Var).is_some() {
            // var identifier = expression
            self.parse_var()

        } else if self.accept(ScriptLexerToken::Def).is_some() {
            // def fn args { statements }
            self.parse_def()

        } else if self.accept(ScriptLexerToken::If).is_some() {
            // if expression { statements }
            self.parse_if()

        } else if self.accept(ScriptLexerToken::Using).is_some() {
            // using expression { statements }
            self.parse_using()

        } else if self.accept(ScriptLexerToken::While).is_some() {
            // while expression { statements }
            self.parse_while()

        } else if self.accept(ScriptLexerToken::Loop).is_some() {
            // loop { statements }
            self.parse_loop()

        } else if self.accept(ScriptLexerToken::For).is_some() {
            // for identifier in expression { statements }
            self.parse_for()

        } else if self.lookahead_is(ScriptLexerToken::Identifier) {
            // While commands are either <Expression> or <Expression> <Expression>, we
            // force the first expression to be an identifier at the moment
            self.parse_command()

        } else {
            // Unrecognised token
            Err(ParseError::new())
        }
    }

    ///
    /// Parses a command
    ///
    /// Syntax '<expression>', '<expression> <expression>'
    ///
    pub fn parse_command(&mut self) -> Result<Script, ParseError> {
        // Starts with an expression specifying the command to run
        self.parse_expression().and_then(move |command_expression| {
            // Followed by arguments (or an end-of-expression marker)
            if self.accept(ScriptLexerToken::Newline).is_some()
               || self.lookahead_is(ScriptLexerToken::EndOfFile) {
                // Newline or EOF ends a command
                Ok(Script::RunCommand(command_expression, None))

            } else {
                // Anything else should be an argument expression
                self.parse_expression().and_then(move |argument_expression| {
                    Ok(Script::RunCommand(command_expression, Some(argument_expression)))

                }).and_then(move |command| {
                    // Command must be followed by a newline
                    if self.accept(ScriptLexerToken::Newline).is_some()
                       || self.lookahead_is(ScriptLexerToken::EndOfFile) {
                        Ok(command)
                    } else {
                        Err(ParseError::new())
                    }

                })

            }
        })
    }

    ///
    /// Parses an Expression
    ///
    pub fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        if self.accept(ScriptLexerToken::Newline).is_some() {
            // Ignore newlines within an expression
            self.parse_expression()

        } else if let Some(identifier) = self.accept(ScriptLexerToken::Identifier) {
            // Simple expression
            Ok(Expression::Identifier(identifier.clone()))

        } else if let Some(number) = self.accept(ScriptLexerToken::Number) {
            // Simple expression
            Ok(Expression::Number(number.clone()))

        } else if let Some(number) = self.accept(ScriptLexerToken::HexNumber) {
            // Hex numbers work like normal numbers
            Ok(Expression::Number(number.clone()))

        } else if let Some(string) = self.accept(ScriptLexerToken::String) {
            // Simple expression
            Ok(Expression::String(string.clone()))

        } else {
            // Syntax error
            Err(ParseError::new())

        }
    }

    pub fn parse_let(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    pub fn parse_var(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    pub fn parse_def(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    pub fn parse_if(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    pub fn parse_using(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    pub fn parse_while(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    pub fn parse_loop(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    pub fn parse_for(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }
}

impl ParseScriptTool {
    ///
    /// Creates a new script parsing tool
    ///
    pub fn new() -> ParseScriptTool {
        ParseScriptTool { }
    }
}