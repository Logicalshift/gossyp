
use super::super::lex::*;

///
/// Tokens that can exist in a script
///
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub enum ScriptLexerToken {
    Unknown,
    EndOfFile,

    Identifier, 
    String,
    Number,
    HexNumber,

    Newline,
    Whitespace,
    Comment,

    Let,
    Var,
    If,
    Using,
    While,
    Do,
    Loop,
    For,
    In,
    Def,

    Symbol(String)
}

impl ScriptLexerToken {
    ///
    /// Shortcut for generating a symbol token
    ///
    #[inline]
    pub fn symbol(sym: &str) -> ScriptLexerToken {
        ScriptLexerToken::Symbol(String::from(sym))
    }
}

///
/// Token matched from the script
///
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct ScriptToken {
    pub token:      ScriptLexerToken,
    pub start:      i32,
    pub end:        i32,
    pub matched:    String
}

impl ScriptToken {
    ///
    /// Creates a new script token
    ///
    pub fn new(lexer_token: ScriptLexerToken, start: i32, end: i32, matched: String) -> ScriptToken {
        ScriptToken { 
            token:      lexer_token,
            start:      start,
            end:        end,
            matched:    matched
        }
    }

    ///
    /// Creates a new script token from a generic `LexerMatch` object
    ///
    pub fn from_lexer_match(lexer_match: &LexerMatch) -> ScriptToken {
        let token: &str     = &lexer_match.token;
        let script_token    = match token {
            "let"           => ScriptLexerToken::Let,
            "var"           => ScriptLexerToken::Var,
            "if"            => ScriptLexerToken::If,
            "using"         => ScriptLexerToken::Using,
            "while"         => ScriptLexerToken::While,
            "do"            => ScriptLexerToken::Do,
            "loop"          => ScriptLexerToken::Loop,
            "for"           => ScriptLexerToken::For,
            "in"            => ScriptLexerToken::In,
            "def"           => ScriptLexerToken::Def,

            "." | "," | ":" | "+" | "-" | "*" | "/" | "|" | "&" | "=" | "==" | "!=" | ">" | "<" | "<=" | ">=" | "!" | "?" | "||" | "&&" | "(" | ")" | "{" | "}" | "[" | "]"
                            => ScriptLexerToken::Symbol(lexer_match.token.clone()),
            
            "String"        => ScriptLexerToken::String,
            "Number"        => ScriptLexerToken::Number,
            "HexNumber"     => ScriptLexerToken::HexNumber,
            "Identifier"    => ScriptLexerToken::Identifier,
            "Newline"       => ScriptLexerToken::Newline,
            "Whitespace"    => ScriptLexerToken::Whitespace,
            "Comment"       => ScriptLexerToken::Comment,
            
            _               => ScriptLexerToken::Unknown
        };

        ScriptToken::new(script_token, lexer_match.start, lexer_match.end, lexer_match.matched.clone())
    }
}

///
/// Representation of a parsed script
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Script {
    /// Run a command, with parameters
    RunCommand(Expression),

    /// Runs a sequence of comments
    Sequence(Vec<Script>),

    /// let a = b 
    Let(ScriptToken, Expression),

    /// var a = b
    Var(ScriptToken, Expression),

    /// a = b
    Assign(ScriptToken, Expression),

    /// loop { stuff }
    Loop(Box<Script>),

    /// while expr { stuff }
    While(Expression, Box<Script>),

    /// using expr { stuff }
    Using(Expression, Box<Script>),

    /// def tool pattern { stuff }
    Def(ScriptToken, Expression, Box<Script>)
}

///
/// Representation of an expression from the script
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Expression {

    // -- Constant values and data structures

    /// "Foo"
    String(ScriptToken),

    /// 12.3
    Number(ScriptToken),

    /// [ foo, bar, baz ]
    Array(Vec<Expression>),

    /// ( foo, bar, baz )
    Tuple(Vec<Expression>),

    /// { a: b, c: d }
    Map(Vec<(Expression, Expression)>),

    // -- Evaluatable expressions

    /// some-identifier
    Identifier(ScriptToken),

    /// a[b]
    Index(Box<(Expression, Expression)>),

    /// a.b
    FieldAccess(Box<(Expression, Expression)>),

    /// a (parameters)
    Apply(Box<(Expression, Expression)>)
}

impl Expression {
    ///
    /// Creates a new string expression
    ///
    pub fn string(s: &str) -> Expression {
        Expression::String(ScriptToken { token: ScriptLexerToken::String, start: 0, end: s.len() as i32, matched: String::from(s) })
    }

    ///
    /// Creates a new identifier expression
    ///
    pub fn identifier(id: &str) -> Expression {
        Expression::String(ScriptToken { token: ScriptLexerToken::Identifier, start: 0, end: id.len() as i32, matched: String::from(id) })
    }

    ///
    /// True if this is an Apply expression
    ///
    pub fn is_apply(&self) -> bool {
        match self {
            &Expression::Apply(_)   => true,
            _                       => false
        }
    }
}