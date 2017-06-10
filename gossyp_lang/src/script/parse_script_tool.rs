use std::result::Result;

use gossyp_base::*;
use gossyp_base::basic::*;

use super::super::lex::lex_tool::*;
use super::script::*;

///
/// Represents a parse error
///
#[derive(Serialize, Deserialize, Debug)]
pub struct ParseError {
    pub message: String,
    pub remaining: Vec<ScriptToken>
}

///
/// Tool that parses our scripting language
///
pub struct ParseScriptTool {
}

impl ParseError {
    fn new<'a>(state: &ParseState<'a>, message: &str) -> ParseError {
        ParseError { message: String::from(message), remaining: state.remaining.to_vec() }
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

        } else if let Some(identifier) = self.accept(ScriptLexerToken::Identifier) {
            // Could be Identifier '=' x to be an assignment
            if self.accept(ScriptLexerToken::symbol("=")).is_some() {
                // x = y
                Ok(Script::Assign(identifier.clone(), self.parse_expression()?))
            } else {
                // While commands are either <Expression> or <Expression> <Expression>, we
                // force the first expression to be an identifier at the moment
                self.parse_command(identifier)
            }

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
    fn parse_command(&mut self, initial_identifier: &ScriptToken) -> Result<Script, ParseError> {
        // Turn the initial identifier into an expression
        let identifier_expr = Expression::Identifier(initial_identifier.clone());

        // Starts with an expression specifying the command to run
        self.parse_expression_rhs(identifier_expr).and_then(move |command_expression| {
            // Followed by arguments (or an end-of-expression marker)
            if self.accept(ScriptLexerToken::Newline).is_some()
               || self.lookahead_is(ScriptLexerToken::symbol("}"))
               || self.lookahead_is(ScriptLexerToken::EndOfFile) {
                // Newline or EOF ends a command
                Ok(Script::RunCommand(command_expression))

            } else if !command_expression.is_apply() {
                // Anything else should be an argument expression
                self.parse_expression().and_then(move |argument_expression| {
                    Ok(Script::RunCommand(Expression::Apply(Box::new((command_expression, argument_expression)))))

                }).and_then(move |command| {
                    // Command must be followed by a newline
                    if self.accept(ScriptLexerToken::Newline).is_some()
                       || self.lookahead_is(ScriptLexerToken::EndOfFile) {
                        Ok(command)
                    } else {
                        Err(ParseError::new(self, "Found extra tokens after the end of a command"))
                    }

                })

            } else {
                // Can't apply more parameters to an Apply expression this way
                Err(ParseError::new(self, "Found extra tokens after the end of a command"))

            }
        })
    }

    ///
    /// Parses an Expression
    ///
    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        let left_expr = if self.lookahead_is(ScriptLexerToken::symbol("[")) {
            self.parse_array_expression(ScriptLexerToken::symbol("["), ScriptLexerToken::symbol("]"))
                .map(|array_entries| Expression::Array(array_entries))

        } else if self.lookahead_is(ScriptLexerToken::symbol("(")) {
            self.parse_array_expression(ScriptLexerToken::symbol("("), ScriptLexerToken::symbol(")"))
                .map(|tuple_entries| {
                    if tuple_entries.len() == 1 {
                        // (x) == x
                        tuple_entries[0].clone()
                    } else {
                        // (x, y) == tuple
                        Expression::Tuple(tuple_entries)
                    }
                })
        } else if self.lookahead_is(ScriptLexerToken::symbol("{")) {
            self.parse_map_expression()

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

        } else if self.lookahead_is(ScriptLexerToken::symbol("(")) {
            // a(b) = 'call command a with parameters b'
            self.parse_expression().map(|parameters| {
                Expression::Apply(Box::new((left_expr, parameters)))
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
    fn parse_array_expression(&mut self, open_bracket: ScriptLexerToken, close_bracket: ScriptLexerToken) -> Result<Vec<Expression>, ParseError> {
        // Opening '['
        if self.accept(open_bracket.clone()).is_none() {
            // Not an array
            return Err(ParseError::new(self, "Not an array"));
        }

        let mut components = vec![];

        // Array goes until the final ']'
        while self.accept(close_bracket.clone()).is_none() {
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
                && !self.lookahead_is(close_bracket.clone()) {
                // Expected ','
                return Err(ParseError::new(self, "Expected ',' or ']'"));
            }

            // Newlines allowed after the ','
            self.skip_newlines();
        }

        Ok(components)
    }

    ///
    /// Parses a map expression ('{ foo: bar ... }')
    ///
    fn parse_map_expression(&mut self) -> Result<Expression, ParseError> {
        // Opening '{'
        if self.accept(ScriptLexerToken::symbol("{")).is_none() {
            // Not an array
            return Err(ParseError::new(self, "Not a map"));
        }

        let mut components = vec![];

        // Array goes until the final '}'
        while self.accept(ScriptLexerToken::symbol("}")).is_none() {
            // <expr> : <expr>
            
            // Parse the key component
            let key_component = match self.parse_expression() {
                Err(failure)    => return Err(failure),
                Ok(component)   => component
            };

            // ':'
            if self.accept(ScriptLexerToken::symbol(":")).is_none() {
                return Err(ParseError::new(self, "Expecting ':'"));
            }

            // Parse the value component
            let value_component = match self.parse_expression() {
                Err(failure)    => return Err(failure),
                Ok(component)   => component
            };

            // Add to the components
            components.push((key_component, value_component));

            // Components separated by commas. Newlines are ignored
            self.skip_newlines();

            // Followed by a comma or the closing '}'
            if self.accept(ScriptLexerToken::symbol(",")).is_none()
                && !self.lookahead_is(ScriptLexerToken::symbol("}")) {
                // Expected ','
                return Err(ParseError::new(self, "Expected ',' or ']'"));
            }

            // Newlines allowed after the ','
            self.skip_newlines();
        }

        Ok(Expression::Map(components))
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
        if let Some(identifier) = self.accept(ScriptLexerToken::Identifier) {
            if self.accept(ScriptLexerToken::Symbol(String::from("="))).is_some() {
                self.parse_expression()
                    .map(|expr| {
                        Script::Let(identifier.clone(), expr)
                    })
            } else {
                Err(ParseError::new(self, "Was expecting '='"))
            }
        } else {
            Err(ParseError::new(self, "Was expecting an identifier for the new variable"))
        }
    }

    fn parse_var(&mut self) -> Result<Script, ParseError> {
        if let Some(identifier) = self.accept(ScriptLexerToken::Identifier) {
            if self.accept(ScriptLexerToken::Symbol(String::from("="))).is_some() {
                self.parse_expression()
                    .map(|expr| {
                        Script::Var(identifier.clone(), expr)
                    })
            } else {
                Err(ParseError::new(self, "Was expecting '='"))
            }
        } else {
            Err(ParseError::new(self, "Was expecting an identifier for the new variable"))
        }
    }

    fn parse_def(&mut self) -> Result<Script, ParseError> {
        unimplemented!()
    }

    fn parse_if(&mut self) -> Result<Script, ParseError> {
        // if expr { statement }
        let condition   = self.parse_expression()?;
        let block       = self.parse_statement_block()?;

        if self.accept(ScriptLexerToken::Else).is_some() {
            let else_block = self.parse_statement_block()?;

            Ok(Script::If(condition, Box::new(block), Some(Box::new(else_block))))
        } else {
            Ok(Script::If(condition, Box::new(block), None))
        }
    }

    fn parse_statement_block(&mut self) -> Result<Script, ParseError> {
        // { statements }
        if self.accept(ScriptLexerToken::symbol("{")).is_some() {
            let mut block = vec![];

            while self.accept(ScriptLexerToken::symbol("}")).is_none() {
                block.push(self.parse_statement()?)
            }

            if block.len() == 1 {
                Ok(block[0].clone())
            } else {
                Ok(Script::Sequence(block))
            }
        } else {
            Err(ParseError::new(self, "Was expecting '{'"))
        }
    }

    fn parse_using(&mut self) -> Result<Script, ParseError> {
        // using expr { statements }
        let using = self.parse_expression()?;
        let block = self.parse_statement_block()?;

        Ok(Script::Using(using, Box::new(block)))
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

    fn applies_to(script: &Script) -> Option<(Expression, Expression)> {
        match script {
            &Script::RunCommand(Expression::Apply(ref boxed_args)) => Some((**boxed_args).clone()),
            _ => None
        }
    }

    #[test]
    fn can_parse_command_statement() {
        let statement   = "some-command";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::RunCommand(Expression::Identifier(_)) => true, _ => false});
    }

    #[test]
    fn can_parse_let_statement() {
        let statement   = "let foo = bar";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::Let(_, Expression::Identifier(_)) => true, _ => false});
    }

    #[test]
    fn can_parse_if_statement() {
        let statement   = "if foo { bar }";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::If(Expression::Identifier(_), _, None) => true, _ => false});
    }

    #[test]
    fn can_parse_multi_line_if_statement() {
        let statement   = "if foo {\nbar\nbaz\n}";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::If(Expression::Identifier(_), _, None) => true, _ => false});
    }

    #[test]
    fn can_parse_if_else_statement() {
        let statement   = "if foo { bar } else { baz }";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::If(Expression::Identifier(_), _, Some(_)) => true, _ => false});
    }

    #[test]
    fn can_parse_var_statement() {
        let statement   = "var foo = bar";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::Var(_, Expression::Identifier(_)) => true, _ => false});
    }

    #[test]
    fn can_parse_assignment() {
        let statement   = "foo = bar";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::Assign(_, Expression::Identifier(_)) => true, _ => false});
    }

    #[test]
    fn extra_data_is_an_error() {
        let statement   = "some-command some-arg some-error";
        let parsed      = parse(statement);

        assert!(parsed.is_err());
    }

    #[test]
    fn can_parse_field_access() {
        let statement   = "some-command.some-field";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::RunCommand(Expression::FieldAccess(_)) => true, _ => false});
    }

    #[test]
    fn can_parse_array_indexing() {
        let statement   = "some-command[0]";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::RunCommand(Expression::Index(_)) => true, _ => false});
    }

    #[test]
    fn can_parse_apply_to_parameter() {
        let statement   = "some-command some-other-command(1,2)";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::RunCommand(_) => true, _ => false});
    }

    #[test]
    fn can_parse_command_statement_with_parameter() {
        let statement   = "some-command 1";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match applies_to(cmd) { Some((Expression::Identifier(_), Expression::Number(_))) => true, _ => false });
    }

    #[test]
    fn can_parse_command_statement_with_string_parameter() {
        let statement   = "some-command \"Foo\"";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match applies_to(cmd) { Some((Expression::Identifier(_), Expression::String(_))) => true, _ => false });
    }

    #[test]
    fn can_parse_command_statement_with_string_parameter_2() {
        let statement   = "some-command \"Foo\".\"bar\"";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match applies_to(cmd) { Some((Expression::Identifier(_), Expression::FieldAccess(_))) => true, _ => false });
    }

    #[test]
    fn can_parse_command_statement_with_empty_tuple_parameter() {
        let statement   = "some-command ()";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match applies_to(cmd) { Some((Expression::Identifier(_), Expression::Tuple(_))) => true, _ => false });
    }

    #[test]
    fn can_parse_command_statement_with_single_tuple_parameter() {
        let statement   = "some-command(1)";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match applies_to(cmd) { Some((Expression::Identifier(_), Expression::Number(_))) => true, _ => false });
    }

    #[test]
    fn can_parse_command_statement_with_multi_tuple_parameter() {
        let statement   = "some-command(1, 2, 3)";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match applies_to(cmd) { Some((Expression::Identifier(_), Expression::Tuple(_))) => true, _ => false });
    }

    #[test]
    fn can_parse_command_statement_with_complex_tuple_parameter() {
        let statement   = "some-command(some-command.access, indexed[2], 3)";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match applies_to(cmd) { Some((Expression::Identifier(_), Expression::Tuple(_))) => true, _ => false });
    }

    #[test]
    fn can_parse_command_statement_with_empty_array_parameter() {
        let statement   = "some-command([ ])";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match applies_to(cmd) { Some((Expression::Identifier(_), Expression::Array(_))) => true, _ => false });
    }

    #[test]
    fn can_parse_command_statement_with_single_array_parameter() {
        let statement   = "some-command([ 1 ])";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match applies_to(cmd) { Some((Expression::Identifier(_), Expression::Array(_))) => true, _ => false });
    }

    #[test]
    fn can_parse_command_statement_with_multi_array_parameter() {
        let statement   = "some-command([ 1, 2, 3 ])";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match applies_to(cmd) { Some((Expression::Identifier(_), Expression::Array(_))) => true, _ => false });
    }

    #[test]
    fn can_parse_command_statement_with_complex_array_parameter() {
        let statement   = "some-command([ some-command.access, indexed[2], 3 ])";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match applies_to(cmd) { Some((Expression::Identifier(_), Expression::Array(_))) => true, _ => false });
    }

    #[test]
    fn can_parse_command_statement_with_empty_map_parameter() {
        let statement   = "some-command { }";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match applies_to(cmd) { Some((Expression::Identifier(_), Expression::Map(_))) => true, _ => false });
    }

    #[test]
    fn can_parse_command_statement_with_single_map_parameter() {
        let statement   = "some-command { \"foo\": \"bar\" }";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match applies_to(cmd) { Some((Expression::Identifier(_), Expression::Map(_))) => true, _ => false });
    }

    #[test]
    fn can_parse_command_statement_with_multi_map_parameter() {
        let statement   = "some-command { foo: bar, baz: blarg }";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match applies_to(cmd) { Some((Expression::Identifier(_), Expression::Map(_))) => true, _ => false });
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

    #[test]
    fn can_parse_using_statement() {
        let statement   = "using foo { bar }";
        let parsed      = parse(statement);

        assert!(parsed.is_ok());

        let result = parsed.unwrap();
        assert!(result.len() == 1);

        let ref cmd = result[0];
        assert!(match cmd { &Script::Using(Expression::Identifier(_), _) => true, _ => false});
    }
}
