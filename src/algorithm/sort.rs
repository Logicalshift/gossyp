//!
//! The sort tool sorts JSON arrays, optionally using a supplied tool name
//!

use std::cmp::*;
use std::result::Result;
use serde_json::*;
use silkthread_base::*;

use super::compare::*;

///
/// Parameters that can be passed to the sort tool
///
#[derive(Serialize, Deserialize)]
pub struct SortParameters {
    /// Values to sort
    array: Vec<Value>,

    /// Tool to use to make comparisons
    compare_tool: Option<String>
}

impl SortParameters {
    pub fn new<'a>(array: Vec<Value>, compare_tool: Option<&'a str>) -> SortParameters {
        SortParameters { array: array, compare_tool: compare_tool.map(|s| String::from(s)) }
    }
}

///
/// Tool that can be used to sort JSON arrays
///
pub struct SortTool {
    /// Default comparison tool
    default_compare_tool: Box<Tool>
}

impl SortTool {
    /// 
    /// Creates a new sort tool, using the standard comparison tool
    ///
    pub fn new() -> SortTool {
        SortTool { default_compare_tool: Box::new(CompareTool::new()) }
    }

    ///
    /// Sorts an array of JSON values
    ///
    pub fn sort(mut array: Vec<Value>, compare_tool: &Box<Tool>, environment: &Environment) -> Vec<Value> {
        // Sort the array using the comparison tool for ordering
        array.sort_by(|v1, v2| {
            // Perform the comparison
            // We ignore errors from the comparison tool
            let compare_result  = compare_tool.invoke_json(json![ [ v1, v2 ] ], environment).unwrap_or(Value::Number(Number::from_f64(0.0).unwrap()));
            let ordering        = match compare_result { Value::Number(ref n) => n.as_f64().unwrap_or(0.0), _ => 0.0 };

            // Tool should return a number indicating ordering
            // (For now, we assume it's indicating equality in the case where it gives an error or a value other than a number)
            if ordering < 0.0 {
                Ordering::Less
            } else if ordering > 0.0 {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        array
    }

    ///
    /// Sorts an array of JSON values, using the default tool
    ///
    pub fn sort_default(&self, array: Vec<Value>, environment: &Environment) -> Vec<Value> {
        SortTool::sort(array, &self.default_compare_tool, environment)
    }
}

impl Tool for SortTool {
    fn invoke_json(&self, input: Value, environment: &Environment) -> Result<Value, Value> {
        if let Value::Object(sort_parameters) = input {
            if let Some(&Value::Array(ref array)) = sort_parameters.get("array") { 
                // We either sort using the default ordering (ie, CompareTool) or a custom tool
                if let Some(&Value::String(ref compare_tool_name)) = sort_parameters.get("compare_tool") {
                    // Tool comes from the current environment
                    let compare_tool = environment.get_json_tool(&compare_tool_name);

                    // Sort if we successfully fetched a tool, error if we did not
                    match compare_tool {
                        Ok(compare_tool) => {
                            Ok(Value::Array(SortTool::sort(array.clone(), &compare_tool, environment)))
                        },

                        Err(retrieve_error) => {
                            Err(json![ {
                                "error":        "Compare tool not found",
                                "description":  retrieve_error.message()
                            } ])
                        }
                    }
                } else {
                    // No compare tool name supplied
                    Ok(Value::Array(SortTool::sort(array.clone(), &self.default_compare_tool, environment)))
                }
            } else {
                // Parameters are an object but there's no array
                Err(json![ {
                    "error": "Parameters to sort-tool must be an array or of the form { \"array\": <array>, \"compare_tool\": <tool_name> }" 
                }])
            }
        } else if let Value::Array(array) = input {
            // Input is just an array
            Ok(Value::Array(self.sort_default(array, environment)))
        } else {
            Err(json![ {
                "error": "Parameters to sort-tool must be an array or of the form { \"array\": <array>, \"compare_tool\": <tool_name> }" 
            }])
        }
    }
}

#[cfg(test)]
mod test{
    use super::*;
    use silkthread_base::basic::*;

    #[test]
    fn can_sort_array() {
        let env     = EmptyEnvironment::new();
        let tool    = TypedTool::from(Box::new(SortTool::new()));

        assert!(tool.invoke(vec![ 2, 5, 3, 1, 4 ], &env) == Ok(vec![ 1,2,3,4,5 ]));
    }

    #[test]
    fn can_sort_using_object() {
        let env     = EmptyEnvironment::new();
        let tool    = TypedTool::from(Box::new(SortTool::new()));

        assert!(tool.invoke(SortParameters::new(vec![ json![2], json![5], json![3], json![1], json![4] ], None), &env) == Ok(vec![ 1,2,3,4,5 ]));
    }

    #[test]
    fn can_sort_using_custom_tool() {
        let env             = DynamicEnvironment::new();
        let tool            = TypedTool::from(Box::new(SortTool::new()));
        let compare_tool    = TypedTool::from(Box::new(CompareTool::new()));

        env.define("reverse-compare", Box::new(make_pure_tool(move |(a, b) : (Value, Value)| -> i32 {
            -compare_tool.invoke((a, b), &EmptyEnvironment::new()).unwrap_or(0)
        })));

        assert!(tool.invoke(SortParameters::new(vec![ json![2], json![5], json![3], json![1], json![4] ], Some("reverse-compare")), &env) == Ok(vec![ 5,4,3,2,1 ]));
    }

    #[test]
    fn missing_compare_tool_is_error() {
        let env = EmptyEnvironment::new();
        let tool: TypedTool<SortParameters, Vec<i32>> = TypedTool::from(Box::new(SortTool::new()));

        assert!(tool.invoke(SortParameters::new(vec![ json![2], json![5], json![3], json![1], json![4] ], Some("missing")), &env).is_err());
    }
}
