use std::result::Result;
use std::marker::PhantomData;
use std::error::Error;
use serde::*;
use serde_json::*;

use super::super::tool::*;
use super::super::environment::*;

///
/// Represents a tool made from a function
///
pub struct FnTool<TIn: Deserialize<'static>, TOut: Serialize, TErr: Serialize> {
    function: Box<Fn(TIn, &Environment) -> Result<TOut, TErr> + Send + Sync>
}

///
/// Creates a Tool from a function that can produce an error and uses an environment
///
pub fn make_dynamic_tool<TIn: Deserialize<'static>, TOut: Serialize, TErr: Serialize, F: 'static+Send+Sync+Fn(TIn, &Environment) -> Result<TOut, TErr>>(function: F) -> FnTool<TIn, TOut, TErr> {
    FnTool { function: Box::new(function) }
}

///
/// Creates a Tool from a function that can produce an error but does not use an environment
///
pub fn make_tool<TIn: Deserialize<'static>, TOut: Serialize, TErr: Serialize, F: 'static+Send+Sync+Fn(TIn) -> Result<TOut, TErr>>(function: F) -> FnTool<TIn, TOut, TErr> {
    make_dynamic_tool(move |input, _| function(input))
}

///
/// Creates a Tool from a function that cannot produce an error and doesn't use an environment
///
pub fn make_pure_tool<TIn: Deserialize<'static>, TOut: Serialize, F: 'static+Send+Sync+Fn(TIn) -> TOut>(function: F) -> FnTool<TIn, TOut, ()> {
    make_dynamic_tool(move |input, _| Ok(function(input)))
}

impl<TIn, TOut, TErr> Tool for FnTool<TIn, TOut, TErr> 
where for<'de> TIn: Deserialize<'de>, TOut: Serialize, TErr: Serialize {
    fn invoke_json(&self, input: Value, environment: &Environment) -> Result<Value, Value> {
        // Decode
        let input_decoded = from_value::<TIn>(input);

        // Chain into the tool itself
        match input_decoded {
            Ok(input_decoded) => {
                // Successfully decoded as the tool's input format
                let result = (self.function)(input_decoded, environment);

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

///
/// Represents a tool with Rust types
///
pub struct TypedTool<TIn: Serialize, TOut: Deserialize<'static>> {
    param1: PhantomData<TIn>,
    param2: PhantomData<TOut>,
    tool: Box<Tool>

    // tool could be &'a Tool and it would work as Box<Tool> implements Tool - except rust's deref behaviour 
    // gets in the way: trying to pass a reference to a box actually passes a reference to the boxed thing, which 
    // then doesn't work because the size is unknown
}

impl<TIn: Serialize, TOut> TypedTool<TIn, TOut> 
where for<'de> TOut: Deserialize<'de> {
    ///
    /// Creates an object that can be used to invoke a tool with typed parameters instead of pure JSON
    ///
    pub fn from(tool: Box<Tool>) -> TypedTool<TIn, TOut> {
        TypedTool { param1: PhantomData, param2: PhantomData, tool: tool }
    }

    ///
    /// Invokes this tool
    ///
    pub fn invoke(&self, input: TIn, environment: &Environment) -> Result<TOut, Value> {
        let json_input = to_value(input);

        match json_input {
            Ok(json_input) => {
                let json_output = self.tool.invoke_json(json_input, environment);

                match json_output {
                    Ok(json_output) => {
                        let result = from_value::<TOut>(json_output);
                        match result {
                            Ok(final_value) => Ok(final_value),
                            Err(erm)        => Err(json![{
                                "error":        "Result decode failed",
                                "description":  erm.description()
                            }])
                        }
                    },

                    Err(json_error) => {
                        Err(json_error)
                    }
                }
            },

            Err(erm) => {
                Err(json![{
                    "error":        "Input encode failed",
                    "description":  erm.description()
                }])
            }
        }
    }
}

///
/// Extension trait for environments that makes it possible to get a Rust typed tool immediately
///
pub trait TypedEnvironment {
    ///
    /// Retrieves a tool using a Rust interface by name
    ///
    fn get_typed_tool<TIn: Serialize, TOut>(&self, name: &str) -> Result<TypedTool<TIn, TOut>, RetrieveToolError>
    where for<'de> TOut: Deserialize<'de>;
}

impl<TEnv> TypedEnvironment for TEnv where TEnv: Environment {
    ///
    /// Retrieves a tool using a Rust interface by name
    ///
    fn get_typed_tool<TIn: Serialize, TOut>(&self, name: &str) -> Result<TypedTool<TIn, TOut>, RetrieveToolError> 
    where for<'de> TOut: Deserialize<'de> {
        self.get_json_tool(name).map(|tool| TypedTool::from(tool))
    }
}

#[cfg(test)]
mod test {
    use super::super::super::*;
    use super::super::*;
    use std::result::Result;
    use serde_json::*;              // Rust says unused, but the json! macro will not work without this

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

    #[test]
    fn can_call_tool_via_typed_interface() {
        let tool        = make_pure_tool(|x: i32| { x+1 });
        let environment = EmptyEnvironment::new();

        let typed_tool  = TypedTool::from(Box::new(tool));
        let result      = typed_tool.invoke(4, &environment);

        assert!(result == Ok(5));
    }
}
