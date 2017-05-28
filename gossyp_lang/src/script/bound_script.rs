use std::rc::*;
use serde_json::*;

use gossyp_base::*;

use super::script::*;

///
/// Represents an expression where the identifiers have been bound to particular
/// locations.
///
pub enum BoundExpression {
    /// Unquoted value
    Value(Value, ScriptToken),

    /// [ foo, bar, baz ]
    Array(Vec<BoundExpression>),

    /// ( foo, bar, baz )
    Tuple(Vec<BoundExpression>),

    /// { a: b, c: d }
    Map(Vec<(BoundExpression, BoundExpression)>),

    // -- Identifier bindings

    /// Identifier that was bound to a particular tool from the script environment
    Tool(Rc<Box<Tool>>, ScriptToken),

    /// Identifier that was bound to a particular variable from the script environment
    Variable(u32, ScriptToken),

    /// Identifier that is the name of a field
    Field(String, ScriptToken),

    // -- Evaluatable expressions

    /// a[b]
    Index(Box<(BoundExpression, BoundExpression)>),

    /// a.b
    FieldAccess(Box<(BoundExpression, BoundExpression)>),

    /// a(parameters)
    Apply(Box<(BoundExpression, BoundExpression)>)
}

///
/// Represents a script where the expressions have been bound to particular locations
///
pub enum BoundScript {
    /// Allocates space for variables before running a script
    AllocateVariables(u32, Box<BoundScript>),

    /// Runs a command
    RunCommand(BoundExpression),

    /// Runs a sequence of commands
    Sequence(Vec<BoundScript>),

    /// let a = b
    Let(u32, BoundExpression, ScriptToken),
    
    /// var a = b
    Var(u32, BoundExpression, ScriptToken),
    
    /// a = b
    Assign(u32, BoundExpression, ScriptToken),

    /// loop { stuff }
    Loop(Box<BoundScript>),

    /// while expr { stuff }
    While(BoundExpression, Box<BoundScript>),
    
    /// using expr { stuff }
    Using(BoundExpression, Box<BoundScript>),

    /// def tool pattern { stuff }
    Def(ScriptToken, BoundExpression, Box<BoundScript>)
}
