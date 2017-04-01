use std::result::Result;

use serde_json::*;

use silkthread_base::*;
use super::script::*;
use super::script_interpreter::*;

///
/// Creates an execution error relating to an expression
///
fn generate_expression_error(error: ScriptEvaluationError, expr: &Expression) -> Value {
    json![{
        "error":                error,
        "failed-expression":    expr
    }]
}

///
/// Creates an unquoted version of a string
///
fn unquote_string(string: &str) -> String {
    let chars: Vec<char>    = string.chars().collect();
    let mut result          = String::new();
    let mut index           = 1;
    while index < chars.len()-1 {
        // Push character
        let chr = chars[index];

        match chr {
            '\\' => { 
                let quoted = chars[index+1];
                index += 1;
                match quoted {
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    quoted => result.push(quoted)
                }
            },
            chr => result.push(chr)
        }

        // Next character
        index += 1;
    }

    result
}

///
/// Parses a number string
///
fn parse_number(number: &str) -> Value {
    if number.contains('.') || number.contains('e') || number.contains('E') {
        json![ number.parse::<f64>().unwrap() ]
    } else if number.starts_with("0x") {
        json![ i64::from_str_radix(&number[2..], 16).unwrap() ]
    } else {
        json![ number.parse::<i64>().unwrap() ]
    }
}

///
/// Attempts to evaluate an expression to a tool
///
pub fn evaluate_expression_to_tool(expression: &Expression, environment: &mut ScriptExecutionEnvironment) -> Result<Box<Tool>, Value> {
    match expression {
        &Expression::Identifier(ref name)   => environment.get_json_tool(&name.matched).map_err(|_| generate_expression_error(ScriptEvaluationError::ToolNameNotFound, expression)),
        _                                   => Err(generate_expression_error(ScriptEvaluationError::ExpressionDoesNotEvaluateToTool, expression))
    }
}

///
/// Calls an expression representing a tool and calls it with the specified parameters
///
pub fn call_tool(tool_name: &Expression, parameters: Value, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    evaluate_expression_to_tool(tool_name, environment).and_then(|tool| {
        // TODO: create environment for the tool to run in

        // Invoke the tool and generate the final result
        environment.invoke_tool(&tool, parameters)
    })
}

///
/// Evaluates an 'apply' expression
///
pub fn apply(&(ref tool, ref parameters): &(Expression, Expression), environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    evaluate_expression(&parameters, environment).and_then(|parameters| {
        call_tool(&tool, parameters, environment)
    })
}

///
/// Evaluates a series of expressions into an array
///
pub fn evaluate_array(exprs: &Vec<Expression>, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    let mut result = vec![];

    for expr in exprs.iter() {
        match evaluate_expression(expr, environment) {
            Ok(next) => result.push(next),
            Err(erm) => return Err(erm)
        }
    }

    Ok(Value::Array(result))
}

///
/// Evaluates a series of expressions into an array
///
pub fn evaluate_map(exprs: &Vec<(Expression, Expression)>, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    let mut result = Map::new();

    for &(ref key_expr, ref value_expr) in exprs.iter() {
        let key = match evaluate_expression(key_expr, environment) {
            Ok(Value::String(key))  => key,
            Ok(_)                   => return Err(generate_expression_error(ScriptEvaluationError::MapKeysMustEvaluateToAString, key_expr)),
            Err(erm)                => return Err(erm)
        };

        let value = match evaluate_expression(value_expr, environment) {
            Ok(value)   => value,
            Err(erm)    => return Err(erm)
        };

        result.insert(key, value);
    }

    Ok(Value::Object(result))
}

///
/// Evaluates an index expression
///
pub fn evaluate_index(lhs: &Expression, rhs: &Expression, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    // Evaluate the left-hand and right-hand sides of the expression
    evaluate_expression(lhs, environment)
        .and_then(|lhs_res| evaluate_expression(rhs, environment).map(|rhs_res| (lhs_res, rhs_res)))
        .and_then(|(lhs_res, rhs_res)| {
            match lhs_res {
                Value::Array(ref array) => {
                    // Array[n] indexing: n must be a number
                    match rhs_res {
                        Value::Number(index) => {
                            index.as_u64()
                                .and_then(|index|       array.get(index as usize))
                                .map(|indexed_value|    indexed_value.clone())
                                .ok_or_else(||          generate_expression_error(ScriptEvaluationError::IndexOutOfBounds, rhs))
                        },

                        _ => Err(generate_expression_error(ScriptEvaluationError::ArrayIndexMustBeANumber, rhs))
                    }
                },

                Value::String(string) => {
                    // String[n] indexing: n must be a number
                    match rhs_res {
                        Value::Number(index) => {
                            index.as_u64()
                                .and_then(|index|       string.chars().nth(index as usize))
                                .map(|indexed_value|    Value::String(indexed_value.to_string()))
                                .ok_or_else(||          generate_expression_error(ScriptEvaluationError::IndexOutOfBounds, rhs))
                        },

                        _ => Err(generate_expression_error(ScriptEvaluationError::ArrayIndexMustBeANumber, rhs))
                    }
                },

                Value::Object(map) => {
                    // Map[n] indexing: n must be a string
                    match rhs_res {
                        Value::String(index) => {
                            map.get(&index)
                                .map(|ref_value|    ref_value.clone())
                                .ok_or_else(||      generate_expression_error(ScriptEvaluationError::ObjectValueNotPresent, rhs))
                        },

                        _ => Err(generate_expression_error(ScriptEvaluationError::MapIndexMustBeAString, rhs))
                    }
                },

                _ => Err(generate_expression_error(ScriptEvaluationError::IndexMustApplyToAnArrayOrAMap, lhs))
            }
        })
}

///
/// Evaluates a single expression
///
pub fn evaluate_expression(expression: &Expression, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    match expression {
        &Expression::Number(ref n)          => Ok(parse_number(&n.matched)),
        &Expression::String(ref s)          => Ok(Value::String(unquote_string(&s.matched))),

        &Expression::Identifier(_)          => call_tool(expression, Value::Null, environment),
        &Expression::Apply(ref expr)        => apply(&*expr, environment),

        &Expression::Array(ref exprs)       => evaluate_array(exprs, environment),
        &Expression::Tuple(ref exprs)       => evaluate_array(exprs, environment),
        &Expression::Map(ref exprs)         => evaluate_map(exprs, environment),

        &Expression::Index(ref boxed_exprs) => {
            let (ref lhs, ref rhs) = **boxed_exprs;
            evaluate_index(lhs, rhs, environment)
        },

        _                                   => Err(generate_expression_error(ScriptEvaluationError::ExpressionNotImplemented, expression))
    }
}

#[cfg(test)]
mod test {
    use silkthread_base::basic::*;
    use super::*;

    #[test]
    fn can_evaluate_string() {
        let string_expr         = Expression::string("\"Foo\"");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&string_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Foo"))));
    }

    #[test]
    fn can_evaluate_string_with_newline() {
        let string_expr         = Expression::string("\"Foo\\n\"");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&string_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Foo\n"))));
    }

    #[test]
    fn can_evaluate_string_with_quote() {
        let string_expr         = Expression::string("\"\\\"\"");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&string_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("\""))));
    }

    #[test]
    fn can_evaluate_number() {
        let num_expr            = Expression::number("42");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&num_expr, &mut env);

        assert!(result == Ok(json![ 42 ]));
    }

    #[test]
    fn can_evaluate_float_number_1() {
        let num_expr            = Expression::number("42.2");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&num_expr, &mut env);

        assert!(result == Ok(json![ 42.2 ]));
    }

    #[test]
    fn can_evaluate_float_number_2() {
        let num_expr            = Expression::number("42e1");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&num_expr, &mut env);

        assert!(result == Ok(json![ 42e1 ]));
    }

    #[test]
    fn can_evaluate_hex_number() {
        let num_expr            = Expression::number("0xabcd");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&num_expr, &mut env);

        assert!(result == Ok(json![ 0xabcd ]));
    }

    #[test]
    fn can_evaluate_array() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&array_expr, &mut env);

        assert!(result == Ok(json![ [ 1,2,3 ] ]));
    }

    #[test]
    fn can_lookup_array_index() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let lookup_expr         = Expression::Index(Box::new((array_expr, Expression::number("1"))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&lookup_expr, &mut env);

        assert!(result == Ok(json![ 2 ]));
    }

    #[test]
    fn can_lookup_string_index() {
        let string_expr         = Expression::string("\"Abcd\"");
        let lookup_expr         = Expression::Index(Box::new((string_expr, Expression::number("2"))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&lookup_expr, &mut env);

        assert!(result == Ok(json![ "c" ]));
    }

    #[test]
    fn positve_index_can_be_out_of_range() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let lookup_expr         = Expression::Index(Box::new((array_expr, Expression::number("100"))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&lookup_expr, &mut env);

        assert!(result.is_err());
    }

    #[test]
    fn negative_index_is_out_of_range() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let lookup_expr         = Expression::Index(Box::new((array_expr, Expression::number("-1"))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&lookup_expr, &mut env);

        assert!(result.is_err());
    }

    #[test]
    fn cannot_index_array_with_string() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let lookup_expr         = Expression::Index(Box::new((array_expr, Expression::string("\"1\""))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&lookup_expr, &mut env);

        assert!(result.is_err());
    }

    #[test]
    fn can_evaluate_tuple() {
        let tuple_expr          = Expression::Tuple(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&tuple_expr, &mut env);

        assert!(result == Ok(json![ [ 1,2,3 ] ]));
    }

    #[test]
    fn can_evaluate_map() {
        let map_expr            = Expression::Map(vec![ (Expression::string("\"Foo\""), Expression::number("1")), (Expression::string("\"Bar\""), Expression::number("2")) ]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&map_expr, &mut env);

        assert!(result == Ok(json![ { "Foo": 1, "Bar": 2 } ]));
    }

    #[test]
    fn can_index_map() {
        let map_expr            = Expression::Map(vec![ (Expression::string("\"Foo\""), Expression::number("1")), (Expression::string("\"Bar\""), Expression::number("2")) ]);
        let lookup_expr         = Expression::Index(Box::new((map_expr, Expression::string("\"Bar\""))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&lookup_expr, &mut env);

        assert!(result == Ok(json![ 2 ]));
    }

    #[test]
    fn can_evaluate_map_with_duplicate_keys() {
        let map_expr            = Expression::Map(vec![ (Expression::string("\"Foo\""), Expression::number("1")), (Expression::string("\"Foo\""), Expression::number("2")) ]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_expression(&map_expr, &mut env);

        assert!(result == Ok(json![ { "Foo": 2 } ]));
    }

    #[test]
    fn can_call_tool() {
        let tool_expr           = Expression::identifier("test");
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = call_tool(&tool_expr, Value::Null, &mut env);

        assert!(result == Ok(Value::String(String::from("Success"))));
    }

    #[test]
    fn can_evaluate_tool_call() {
        let tool_expr           = Expression::identifier("test");
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = evaluate_expression(&tool_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Success"))));
    }

    #[test]
    fn can_evaluate_apply_expression() {
        let tool_expr           = Expression::Apply(Box::new((Expression::identifier("test"), Expression::string("\"Success\""))));
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|s: String| s)));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = evaluate_expression(&tool_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Success"))));
    }
}
