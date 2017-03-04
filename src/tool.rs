//!
//! # Tools
//!

use std::result::Result;
use std::rc::Rc;
use serde_json::*;

use environment::*;

///
/// Trait implemented by things that represent a tool
///
/// A tool is simply a routine that takes some input, does some processing
/// and returns a value or an error. The main difference between a tool and
/// a simple function is that a tool's input and output must be simple data,
/// which for Rust we define as 'can be serialized to JSON'.
///
/// Tools also have the requirement that they are encapsulated and can instantiate
/// themselves with no dependencies other than those they can discover from
/// their environment.
///
/// These two requirements mean that tools can be invoked simply by specifying
/// the input data (without necessarily having to know the exact Rust type involved!).
/// Test cases for tools can be specified as simple JSON data with no need for any
/// actual code. Tools can be turned into stand-alone command line programs or
/// web endpoints with no modification.
///
pub trait Tool {
    ///
    /// Invokes this tool with its input and output specified using JSON
    ///
    fn invoke_json(&self, input: Value, environment: &Environment) -> Result<Value, Value>;
}

impl<T: Tool> Tool for Rc<T> {
    #[inline]
    fn invoke_json(&self, input: Value, environment: &Environment) -> Result<Value, Value> {
        (**self).invoke_json(input, environment)
    }
}

impl<T: Tool> Tool for Box<T> {
    #[inline]
    fn invoke_json(&self, input: Value, environment: &Environment) -> Result<Value, Value> {
        (**self).invoke_json(input, environment)
    }
}
