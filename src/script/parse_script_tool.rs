use std::result::Result;

use silkthread_base::*;
use silkthread_base::basic::*;

use super::super::lex::lex_tool::*;
use super::script::*;

///
/// Represents a parse error
///
#[derive(Serialize, Deserialize, Debug)]
pub struct ParseError {
    pub message: String
}

///
/// Tool that parses our scripting language
///
pub struct ParseScriptTool {
}

impl ParseError {
    fn new<'a>(state: &ParseState<'a>, message: &str) -> ParseError {
        println!("{}", message);
        println!("{:?}", state.remaining);

        ParseError { message: String::from(message) }
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
        if let Some((lookahead, _remaining)) = self.lookahead() {
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
    fn parse_statement(&mut self) -> Result<Script, ParseError> {
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
            Err(ParseError::new(self, "Token cannot begin a statement"))
        }
    }

    ///
    /// Parses a command
    ///
    /// Syntax '<expression>', '<expression> <expression>'
    ///
    fn parse_command(&mut self) -> Result<Script, ParseError> {
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
                        Err(ParseError::new(self, "Found extra tokens after the end of a command"))
                    }

                })

            }
        })
    }

    ///
    /// Parses an Expression
    ///
    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        let left_expr = if self.lookahead_is(ScriptLexerToken::symbol("[")) {
            self.parse_array_expression()

        } else {
            // Simple expression
            self.parse_simple_expression()

        };

        // Look for a RHS
        left_expr.and_then(|left_expr| self.parse_expression_rhs(left_expr))
    }

    ///
    /// Parses the RHS of an expression (if there is one)
    ///
    fn parse_expression_rhs(&mut self, left_expr: Expression) -> Result<Expression, ParseError> {
        if self.accept(ScriptLexerToken::symbol(".")).is_some() {
            // 'a.b' field access
            let right_expr = self.parse_expression();

            right_expr.map(|right_expr| Expression::FieldAccess(Box::new((left_expr, right_expr))))

        } else if self.accept(ScriptLexerToken::symbol("[")).is_some() {
            // a[b] indexing
            let right_expr = self.parse_expression();

            right_expr.and_then(|right_expr| {
                if self.accept(ScriptLexerToken::symbol("]")).is_some() {
                    // Got all of a[b]
                    Ok(Expression::Index(Box::new((left_expr, right_expr))))
                } else {
                    // Missing ']'
                    Err(ParseError::new(self, "Missing ']'"))
                }
            })

        } else {
            Ok(left_expr)

        }
    }

    ///
    /// Skips any newlines
    ///
    fn skip_newlines(&mut self) {
        while self.accept(ScriptLexerToken::Newline).is_some() { }
    }

    ///
    /// Parses an array expression
    ///
    fn parse_array_expression(&mut self) -> Result<Expression, ParseError> {
        // Opening '['
        if self.accept(ScriptLexerToken::symbol("[")).is_none() {
            // Not an array
            return Err(ParseError::new(self, "Not an array"));
        }

        let mut components = vec![];

        // Array goes until the final ']'
        while self.accept(ScriptLexerToken::symbol("]")).is_none() {
            // Read the next component
            let next_component = self.parse_expression();
            
            // Add to the components
            match next_component {
                Err(failure)    => return Err(failure),
                Ok(component)   => components.push(component)
            };

            // Components separated by commas. Newlines are ignored
            self.skip_newlines();

            // Followed by a comma or the closing ']'
            if self.accept(ScriptLexerToken::symbol(",")).is_none()
                && !self.lookahead_is(ScriptLexerToken::symbol("]")) {
                // Expected ','
                return Err(ParseError::new(self, "Expected ',' or ']'"));
            }

            // Newlines allowed after the ','
            self.skip_newlines();
        }

        Ok(Expression::Array(components))
    }

    ///
    /// Parses a simple expression
    ///
    fn parse_simple_expression(&mut self) -> Result<Expression, ParseError> {
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
            Err(ParseError::new(self, "Syntax error (was expecting an expression)"))

        }
    }

    fn parse_let(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    fn parse_var(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    fn parse_def(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    fn parse_if(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    fn parse_using(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    fn parse_while(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    fn parse_loop(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    fn parse_for(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }
}

impl ParseScriptTool {
    ///
    /// Creates a new tool from the parse script tool
    ///
    pub fn new_tool() -> Box<Tool> {
        Box::new(make_tool(move |script: Vec<LexerMatch>| -> Result<Vec<Script>, ParseError> {
            ParseScriptTool::parse(&script)
        }))
    }

    ///
    /// Tries to parse a script from the output of the lexer
    ///
    pub fn parse(input: &[LexerMatch]) -> Result<Vec<Script>, ParseError> {
        // Convert to script tokens
        let as_script_token: Vec<ScriptToken> = input
            .iter()
            .map(|token| ScriptToken::from_lexer_match(token))
            .collect();

        // Parse until we reach the end of the file
        let mut parser = ParseState { remaining: &as_script_token };
        let mut result = vec![];

        while !parser.lookahead_is(ScriptLexerToken::EndOfFile) {
            let next_statement = parser.parse_statement();

            match next_statement {
                // Fail out if we get a parse failure
                Err(failure)        => return Err(failure),

                // Build out the result otherwise
                Ok(next_statement)  => result.push(next_statement)
            }

            // Swallow any trailing newlines
            parser.skip_newlines();
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::lex_script_tool::*;

    ///
    /// Performs lexing
    ///
    fn lex(text: &str) -> Vec<LexerMatch> {
        let lexer = create_lex_script_tool();

        lexer.lex(text)
    }

    fn parse(text: &str) -> Result<Vec<Script>, ParseError> {
        let lexed = lex(text);
        ParseScriptTool::parse(&lexed)
    }

    #[test]
    fn can_parse_command_statement() {
        let statement   = "some-command";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::RunCommand(Expression::Identifier(_), None) => true, _ => false});
    }

    #[test]
    fn can_parse_field_access() {
        let statement   = "some-command.some-field";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::RunCommand(Expression::FieldAccess(_), None) => true, _ => false});
    }

    #[test]
    fn can_parse_array_indexing() {
        let statement   = "some-command[0]";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::RunCommand(Expression::Index(_), None) => true, _ => false});
    }

    #[test]
    fn can_parse_command_statement_with_parameter() {
        let statement   = "some-command 1";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::RunCommand(Expression::Identifier(_), Some(Expression::Number(_))) => true, _ => false});
    }

    #[test]
    fn can_parse_command_statement_with_empty_array_parameter() {
        let statement   = "some-command [ ]";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::RunCommand(Expression::Identifier(_), Some(Expression::Array(_))) => true, _ => false});
    }

    #[test]
    fn can_parse_command_statement_with_single_array_parameter() {
        let statement   = "some-command [ 1 ]";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::RunCommand(Expression::Identifier(_), Some(Expression::Array(_))) => true, _ => false});
    }

    #[test]
    fn can_parse_command_statement_with_multi_array_parameter() {
        let statement   = "some-command [ 1, 2, 3 ]";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::RunCommand(Expression::Identifier(_), Some(Expression::Array(_))) => true, _ => false});
    }

    #[test]
    fn can_parse_command_statement_with_complex_array_parameter() {
        let statement   = "some-command [ some-command.access, indexed[2], 3 ]";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::RunCommand(Expression::Identifier(_), Some(Expression::Array(_))) => true, _ => false});
    }

    #[test]
    fn cannot_try_to_put_multiple_statements_on_one_line() {
        let statement   = "some-command 1 some-other-command";
        let parsed      = parse(statement);

        assert!(parsed.is_err());
    }

    #[test]
    fn can_parse_multiple_lines() {
        let statement   = "some-command\nsome-other-command";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());
        assert!(parsed.unwrap().len() == 2);
    }
}
