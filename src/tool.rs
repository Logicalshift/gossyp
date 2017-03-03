//!
//! # Tools
//!

use std::result::Result;
use std::rc::Rc;
use std::error::Error;
use serde::*;
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

///
/// Represents a tool made from a function
///
pub struct FnTool<TIn: Deserialize, TOut: Serialize, TErr: Serialize> {
    function: Box<Fn(TIn) -> Result<TOut, TErr>>
}

///
/// Creates a Tool from a function
///
pub fn make_tool<TIn: Deserialize, TOut: Serialize, TErr: Serialize, F: 'static+Fn(TIn) -> Result<TOut, TErr>>(function: F) -> FnTool<TIn, TOut, TErr> {
    FnTool { function: Box::new(function) }
}

impl<TIn, TOut, TErr> Tool for FnTool<TIn, TOut, TErr> 
where TIn: Deserialize, TOut: Serialize, TErr: Serialize {
    fn invoke_json(&self, input: Value, _environment: &Environment) -> Result<Value, Value> {
        // Decode
        let input_decoded = from_value::<TIn>(input);

        // Chain into the tool itself
        match input_decoded {
            Ok(input_decoded) => {
                // Successfully decoded as the tool's input format
                let result = (self.function)(input_decoded);

                // Encode the error or the result
                // The encoding can go wrong, and we need to handle it slightly differently for the error or the success case
                match result {
                    Ok(res) => {
                        let encoded = to_value(res);
                        match encoded {
                            Ok(final_value) => Ok(final_value),
                            Err(erm)        => Err(json![{
                                "error":        "JSON encode failed",
                                "description":  erm.description()
                            }])
                        }
                    },
                    
                    Err(erm) => {
                        let encoded = to_value(erm);
                        match encoded {
                            Ok(final_value) => Ok(final_value),
                            Err(erm)        => Err(json![{
                                "error":        "Error encode failed",
                                "description":  erm.description()
                            }])
                        }
                    }
                }
            },

            Err(input_error) => {
                // Input does not match the format expected by the tool
                Err(json![{
                    "error":        "JSON input decode failed",
                    "description":  input_error.description(),
                }])
            }
        }
    }
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

#[cfg(test)]
mod test {
    use super::*;
    use std::result::Result;

    #[derive(Serialize, Deserialize)]
    struct TestIn {
        input: i32
    }

    #[derive(Serialize, Deserialize, PartialEq, Eq)]
    struct TestOut {
        output: i32
    }

    fn test_tool(x: TestIn) -> Result<TestOut, ()> {
        Ok(TestOut { output: x.input + 1 })
    }

    #[test]
    fn can_call_tool_via_json_interface() {
        let tool        = make_tool(test_tool);
        let environment = EmptyEnvironment::new();
        let result      = tool.invoke_json(json![{ "input": 4 }], &environment);

        assert!(result == Ok(json![{ "output": 5 }]));
    }
}
