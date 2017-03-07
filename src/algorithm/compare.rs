use std::result::Result;
use serde_json::*;
use silkthread_base::*;

///
/// The compare tool compares two JSON objects and returns -1, 0, or 1 depending on if the first is
/// less than or greater than the other
///
pub struct CompareTool {
}

impl CompareTool {
    ///
    /// Creates a new compare tool
    ///
    pub fn new() -> CompareTool {
        CompareTool { }
    }

    fn compare_array(array_values: &Vec<Value>, right: &Value) -> i32 {
        match right {
            &Value::Array(ref right_array) => {
                if array_values.len() < right_array.len() {
                    // Lengths differ
                    -1
                } else if array_values.len() > right_array.len() {
                    // Lengths differ
                    1
                } else {
                    // Check the values in the array
                    for index in 0..array_values.len() {
                        let compare = CompareTool::compare_values(&array_values[index], &right_array[index]);
                        if compare != 0 {
                            return compare;
                        }
                    }

                    // Arrays are identical if we reach here
                    0
                }
            },
            &Value::Bool(_)     => -1,
            &Value::Null        => -1,
            &Value::Number(_)   => -1,
            &Value::Object(_)   => -1,
            &Value::String(_)   => -1
        }
    }

    fn compare_bool(val: bool, right: &Value) -> i32 {
        match right {
            &Value::Array(_)        => 1,
            &Value::Bool(right_val) => {
                if val < right_val {
                    -1
                } else if val > right_val {
                    1
                } else {
                    0
                }
            },
            &Value::Null            => -1,
            &Value::Number(_)       => -1,
            &Value::Object(_)       => -1,
            &Value::String(_)       => -1
        }
    }

    fn compare_null(right: &Value) -> i32 {
        match right {
            &Value::Array(_)    => 1,
            &Value::Bool(_)     => 1,
            &Value::Null        => 0,
            &Value::Number(_)   => -1,
            &Value::Object(_)   => -1,
            &Value::String(_)   => -1
        }
    }

    fn compare_number(num: &Number, right: &Value) -> i32 {
        match right {
            &Value::Array(_)                => 1,
            &Value::Bool(_)                 => 1,
            &Value::Null                    => 1,
            &Value::Number(ref right_num)   => {
                if let (Some(lnum), Some(rnum)) = (num.as_i64(), right_num.as_i64()) {
                    // Try comparing as integers first
                    if lnum < rnum {
                        -1
                    } else if lnum > rnum {
                        1
                    } else {
                        0
                    }
                } else if let (Some(lnum), Some(rnum)) = (num.as_u64(), right_num.as_u64()) {
                    // Possible that one side can only be represented as a u64, so try that too
                    if lnum < rnum {
                        -1
                    } else if lnum > rnum {
                        1
                    } else {
                        0
                    }
                } else if let (Some(lnum), Some(rnum)) = (num.as_f64(), right_num.as_f64()) {
                    // If we can't compare as integers, try comparing as floats
                    if lnum < rnum {
                        -1
                    } else if lnum > rnum {
                        1
                    } else {
                        0
                    }
                } else {
                    // Numbers don't have a common format!
                    0
                }
            },
            &Value::Object(_)               => -1,
            &Value::String(_)               => -1
        }
    }

    fn compare_object(obj: &Map<String, Value>, right: &Value) -> i32 {
        match right {
            &Value::Array(_)                => 1,
            &Value::Bool(_)                 => 1,
            &Value::Null                    => 1,
            &Value::Number(_)               => 1,
            &Value::Object(ref right_obj)   => {
                let left_keys: Vec<&String>     = obj.keys().collect();
                let right_keys: Vec<&String>    = right_obj.keys().collect();

                if left_keys.len() < right_keys.len() {
                    -1
                } else if left_keys.len() > right_keys.len() {
                    1
                } else {
                    for index in 0..left_keys.len() {
                        if left_keys[index] < right_keys[index] {
                            return -1;
                        } else if left_keys[index] > right_keys[index] {
                            return 1;
                        } else {
                            let compare = CompareTool::compare_values(&obj[left_keys[index]], &right_obj[left_keys[index]]);
                            if compare != 0 {
                                return compare;
                            }
                        }
                    }

                    0
                }
            },
            &Value::String(_)               => -1
        }
    }

    fn compare_string(s: &String, right: &Value) -> i32 {
        match right {
            &Value::Array(_)            => 1,
            &Value::Bool(_)             => 1,
            &Value::Null                => 1,
            &Value::Number(_)           => 1,
            &Value::Object(_)           => 1,
            &Value::String(ref right_s) => {
                if s < right_s {
                    -1
                } else if s > right_s {
                    1
                } else {
                    0
                }
            }
        }
    }

    ///
    /// Compares two JSON values
    ///
    fn compare_values(left: &Value, right: &Value) -> i32 {
        match left {
            &Value::Array(ref array_values) => CompareTool::compare_array(array_values, right),
            &Value::Bool(val)               => CompareTool::compare_bool(val, right),
            &Value::Null                    => CompareTool::compare_null(right),
            &Value::Number(ref num)         => CompareTool::compare_number(num, right),
            &Value::Object(ref obj)         => CompareTool::compare_object(obj, right),
            &Value::String(ref s)           => CompareTool::compare_string(s, right)
        }
    }
}

impl Tool for CompareTool {
    fn invoke_json(&self, input: Value, _environment: &Environment) -> Result<Value, Value> {
        match input {
            Value::Array(ref values) => {
                if values.len() == 2 {
                    // Compare the values
                    let result = CompareTool::compare_values(&values[0], &values[1]);
                    Ok(json![ result ])
                } else {
                    // Incorrect number of values
                    Err(json![ {
                        "error": "Compare must be called with two values"
                    } ])
                }
            },

            _ => {
                // Parameter must be an array
                Err(json![ {
                    "error": "Compare must be called with an array"
                } ])
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use silkthread_base::basic::*;

    #[test]
    fn can_compare_numbers_i64_lt() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke((1, 2), &env) == Ok(-1));
    }

    #[test]
    fn can_compare_numbers_i64_gt() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke((2, 1), &env) == Ok(1));
    }

    #[test]
    fn can_compare_numbers_i64_eq() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke((2, 2), &env) == Ok(0));
    }

    #[test]
    fn can_compare_numbers_f64_lt() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke((1, 1.1), &env) == Ok(-1));
    }

    #[test]
    fn can_compare_numbers_f64_gt() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke((1.2, 1.1), &env) == Ok(1));
    }

    #[test]
    fn can_compare_strings_lt() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke(("aardvark", "zebra"), &env) == Ok(-1));
    }

    #[test]
    fn can_compare_strings_gt() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke(("zebra", "aardvark"), &env) == Ok(1));
    }

    #[test]
    fn can_compare_strings_eq() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke(("ocelot", "ocelot"), &env) == Ok(0));
    }

    #[test]
    fn can_compare_nulls_eq() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke(((), ()), &env) == Ok(0));
    }

    #[test]
    fn can_compare_bools_lt() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke((false, true), &env) == Ok(-1));
    }

    #[test]
    fn can_compare_bools_gt() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke((true, false), &env) == Ok(1));
    }

    #[test]
    fn can_compare_bools_eq() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke((true, true), &env) == Ok(0));
    }

    #[test]
    fn can_compare_arrays_eq() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke((vec![ 1, 2, 3 ], vec![ 1, 2, 3 ]), &env) == Ok(0));
    }

    #[test]
    fn can_compare_arrays_lt() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke((vec![ 1, 2, 3 ], vec![ 1, 2, 4 ]), &env) == Ok(-1));
    }

    #[test]
    fn can_compare_arrays_gt() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke((vec![ 1, 2, 4 ], vec![ 1, 2, 3 ]), &env) == Ok(1));
    }

    #[test]
    fn can_compare_arrays_gt_len() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke((vec![ 1, 2, 3, 4 ], vec![ 1, 2, 3 ]), &env) == Ok(1));
    }

    #[test]
    fn can_compare_arrays_lt_len() {
        let env     = EmptyEnvironment::new();
        let compare = TypedTool::from(Box::new(CompareTool::new()));
        assert!(compare.invoke((vec![ 1, 2, 3 ], vec![ 1, 2, 3, 4 ]), &env) == Ok(-1));
    }

    #[test]
    fn can_compare_objects_eq() {
        let env     = EmptyEnvironment::new();
        let compare = CompareTool::new();
        assert!(compare.invoke_json(json![ vec![ json![{
            "foo": "bar",
            "bar": "foo"
        }], json![{
            "bar": "foo",
            "foo": "bar"
        }] ] ], &env) == Ok(json![0]));
    }

    #[test]
    fn can_compare_objects_lt() {
        let env     = EmptyEnvironment::new();
        let compare = CompareTool::new();
        assert!(compare.invoke_json(json![ vec![ json![{
            "foo": "aar",
            "bar": "foo"
        }], json![{
            "bar": "foo",
            "foo": "bar"
        }] ] ], &env) == Ok(json![-1]));
    }

    #[test]
    fn can_compare_objects_gt() {
        let env     = EmptyEnvironment::new();
        let compare = CompareTool::new();
        assert!(compare.invoke_json(json![ vec![ json![{
            "foo": "bar",
            "bar": "foo"
        }], json![{
            "bar": "foo",
            "foo": "aar"
        }] ] ], &env) == Ok(json![1]));
    }

    #[test]
    fn can_compare_objects_lt_len() {
        let env     = EmptyEnvironment::new();
        let compare = CompareTool::new();
        assert!(compare.invoke_json(json![ vec![ json![{
            "foo": "bar",
            "bar": "foo"
        }], json![{
            "bar": "foo",
            "foo": "bar",
            "quux": "plugh"
        }] ] ], &env) == Ok(json![-1]));
    }

    #[test]
    fn can_compare_objects_gt_len() {
        let env     = EmptyEnvironment::new();
        let compare = CompareTool::new();
        assert!(compare.invoke_json(json![ vec![ json![{
            "foo": "bar",
            "bar": "foo",
            "quux": "plugh"
        }], json![{
            "bar": "foo",
            "foo": "bar"
        }] ] ], &env) == Ok(json![1]));
    }
}
