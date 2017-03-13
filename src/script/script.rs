
use super::super::lex::*;

#[derive(Serialize, Deserialize)]
enum Script {
    /// Run a command, with parameters
    RunCommand(Expression),

    /// Runs a sequence of comments
    Sequence(Vec<Script>),

    /// let a = b 
    Let(LexerMatch, Expression),

    /// var a = b
    Var(LexerMatch, Expression),

    /// a = b
    Assign(LexerMatch, Expression),

    /// loop { stuff }
    Loop(Box<Script>),

    /// while expr { stuff }
    While(Expression, Box<Script>),

    /// using expr { stuff }
    Using(Expression, Box<Script>),

    /// def tool pattern { stuff }
    Def(LexerMatch, Expression, Box<Script>)
}

#[derive(Serialize, Deserialize)]
pub enum Expression {

    // -- Constant values and data structures

    /// "Foo"
    String(LexerMatch),

    /// 12.3
    Number(LexerMatch),

    /// [ foo, bar, baz ]
    Array(Vec<Expression>),

    /// ( foo, bar, baz )
    Tuple(Vec<Expression>),

    /// { a: b, c: d }
    Map(Vec<(Expression, Expression)>),

    // -- Evaluatable expressions

    /// some-identifier
    Identifier(LexerMatch),

    /// a[b]
    Index(Box<(Expression, Expression)>),

    /// a.b
    FieldAccess(Box<(Expression, Expression)>),

    /// a (parameters)
    Apply(Box<(Expression, Expression)>)
}
