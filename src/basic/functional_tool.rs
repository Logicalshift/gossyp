use std::result::Result;
use std::error::Error;
use serde::*;
use serde_json::*;

use super::super::tool::*;
use super::super::environment::*;

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

#[cfg(test)]
mod test {
    use super::super::super::*;
    use super::super::*;
    use std::result::Result;
    use serde_json::*;

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
