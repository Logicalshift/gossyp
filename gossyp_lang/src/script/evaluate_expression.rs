use std::result::Result;

use serde_json::*;

use gossyp_base::*;
use super::script::*;
use super::bound_script::*;
use super::bind_expression::*;
use super::script_interpreter::*;

///
/// Creates an execution error relating to an expression
///
fn generate_bound_expression_error(error: ScriptEvaluationError, _expr: &BoundExpression) -> Value {
    json![{
        "error":    error
    }]
}

///
/// Attempts to evaluate an expression to a tool
///
pub fn evaluate_expression_to_tool<'a>(expression: &'a BoundExpression) -> Result<&'a Box<Tool>, Value> {
    match expression {
        &BoundExpression::Tool(ref tool, ref _token)    => Ok(&*tool),
        _                                               => Err(generate_bound_expression_error(ScriptEvaluationError::ExpressionDoesNotEvaluateToTool, expression))
    }
}

///
/// Calls an expression representing a tool and calls it with the specified parameters
///
pub fn call_tool(tool: &Box<Tool>, parameters: Value, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    environment.invoke_tool(tool, parameters)
}

///
/// Evaluates an 'apply' expression
///
pub fn apply(&(ref tool, ref parameters): &(BoundExpression, BoundExpression), environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    let parameters_value    = evaluate_expression(parameters, environment)?;
    let applies_to          = evaluate_expression_to_tool(tool)?;

    call_tool(applies_to, parameters_value, environment)
}

///
/// Evaluates a series of expressions into an array
///
pub fn evaluate_array(exprs: &Vec<BoundExpression>, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    let mut result = vec![];

    for expr in exprs.iter() {
        result.push(evaluate_expression(expr, environment)?)
    }

    Ok(Value::Array(result))
}

///
/// Evaluates a series of expressions into an array
///
pub fn evaluate_map(exprs: &Vec<(BoundExpression, BoundExpression)>, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    let mut result = Map::new();

    for &(ref key_expr, ref value_expr) in exprs.iter() {
        let key = match evaluate_expression(key_expr, environment) {
            Ok(Value::String(key))  => key,
            Ok(_)                   => return Err(generate_bound_expression_error(ScriptEvaluationError::MapKeysMustEvaluateToAString, key_expr)),
            Err(erm)                => return Err(erm)
        };

        let value = evaluate_expression(value_expr, environment)?;

        result.insert(key, value);
    }

    Ok(Value::Object(result))
}

///
/// Evaluates an index expression
///
pub fn evaluate_index(lhs: &BoundExpression, rhs: &BoundExpression, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
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
                                .ok_or_else(||          generate_bound_expression_error(ScriptEvaluationError::IndexOutOfBounds, rhs))
                        },

                        _ => Err(generate_bound_expression_error(ScriptEvaluationError::ArrayIndexMustBeANumber, rhs))
                    }
                },

                Value::String(string) => {
                    // String[n] indexing: n must be a number
                    match rhs_res {
                        Value::Number(index) => {
                            index.as_u64()
                                .and_then(|index|       string.chars().nth(index as usize))
                                .map(|indexed_value|    Value::String(indexed_value.to_string()))
                                .ok_or_else(||          generate_bound_expression_error(ScriptEvaluationError::IndexOutOfBounds, rhs))
                        },

                        _ => Err(generate_bound_expression_error(ScriptEvaluationError::ArrayIndexMustBeANumber, rhs))
                    }
                },

                Value::Object(map) => {
                    // Map[n] indexing: n must be a string
                    match rhs_res {
                        Value::String(index) => {
                            map.get(&index)
                                .map(|ref_value|    ref_value.clone())
                                .ok_or_else(||      generate_bound_expression_error(ScriptEvaluationError::ObjectValueNotPresent, rhs))
                        },

                        _ => Err(generate_bound_expression_error(ScriptEvaluationError::MapIndexMustBeAString, rhs))
                    }
                },

                _ => Err(generate_bound_expression_error(ScriptEvaluationError::IndexMustApplyToAnArrayOrAMap, lhs))
            }
        })
}

///
/// Evaluates a single expression
///
pub fn evaluate_expression(expression: &BoundExpression, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    match expression {
        &BoundExpression::Value(ref value, ref _token)          => Ok(value.clone()),

        &BoundExpression::Tool(ref tool, ref _token)            => call_tool(tool, Value::Null, environment),
        &BoundExpression::Variable(ref _var_num, ref _token)    => unimplemented!(),
        &BoundExpression::Field(ref _field_name, ref _token)    => unimplemented!(),
        
        &BoundExpression::Array(ref values)                     => evaluate_array(values, environment),
        &BoundExpression::Tuple(ref values)                     => evaluate_array(values, environment),
        &BoundExpression::Map(ref values)                       => evaluate_map(values, environment),

        &BoundExpression::FieldAccess(ref _accessor)            => unimplemented!(),
        &BoundExpression::Apply(ref application)                => apply(&*application, environment),

        &BoundExpression::Index(ref index)                      => {
            let (ref lhs, ref rhs) = **index;
            evaluate_index(lhs, rhs, environment)
        },
    }
}

///
/// Evaluates a single expression
///
pub fn evaluate_unbound_expression(expression: &Expression, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
    let bound = bind_expression(expression, environment)?;

    evaluate_expression(&bound, environment)
}

#[cfg(test)]
mod test {
    use gossyp_base::basic::*;
    use super::*;

    #[test]
    fn can_evaluate_string() {
        let string_expr         = Expression::string("\"Foo\"");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&string_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Foo"))));
    }

    #[test]
    fn can_evaluate_string_with_newline() {
        let string_expr         = Expression::string("\"Foo\\n\"");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&string_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Foo\n"))));
    }

    #[test]
    fn can_evaluate_string_with_quote() {
        let string_expr         = Expression::string("\"\\\"\"");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&string_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("\""))));
    }

    #[test]
    fn can_evaluate_number() {
        let num_expr            = Expression::number("42");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&num_expr, &mut env);

        assert!(result == Ok(json![ 42 ]));
    }

    #[test]
    fn can_evaluate_float_number_1() {
        let num_expr            = Expression::number("42.2");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&num_expr, &mut env);

        assert!(result == Ok(json![ 42.2 ]));
    }

    #[test]
    fn can_evaluate_float_number_2() {
        let num_expr            = Expression::number("42e1");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&num_expr, &mut env);

        assert!(result == Ok(json![ 42e1 ]));
    }

    #[test]
    fn can_evaluate_hex_number() {
        let num_expr            = Expression::number("0xabcd");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&num_expr, &mut env);

        assert!(result == Ok(json![ 0xabcd ]));
    }

    #[test]
    fn can_evaluate_array() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&array_expr, &mut env);

        assert!(result == Ok(json![ [ 1,2,3 ] ]));
    }

    #[test]
    fn can_lookup_array_index() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let lookup_expr         = Expression::Index(Box::new((array_expr, Expression::number("1"))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&lookup_expr, &mut env);

        assert!(result == Ok(json![ 2 ]));
    }

    #[test]
    fn can_lookup_string_index() {
        let string_expr         = Expression::string("\"Abcd\"");
        let lookup_expr         = Expression::Index(Box::new((string_expr, Expression::number("2"))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&lookup_expr, &mut env);

        assert!(result == Ok(json![ "c" ]));
    }

    #[test]
    fn positve_index_can_be_out_of_range() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let lookup_expr         = Expression::Index(Box::new((array_expr, Expression::number("100"))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&lookup_expr, &mut env);

        assert!(result.is_err());
    }

    #[test]
    fn negative_index_is_out_of_range() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let lookup_expr         = Expression::Index(Box::new((array_expr, Expression::number("-1"))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&lookup_expr, &mut env);

        assert!(result.is_err());
    }

    #[test]
    fn cannot_index_array_with_string() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let lookup_expr         = Expression::Index(Box::new((array_expr, Expression::string("\"1\""))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&lookup_expr, &mut env);

        assert!(result.is_err());
    }

    #[test]
    fn can_evaluate_tuple() {
        let tuple_expr          = Expression::Tuple(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&tuple_expr, &mut env);

        assert!(result == Ok(json![ [ 1,2,3 ] ]));
    }

    #[test]
    fn can_evaluate_map() {
        let map_expr            = Expression::Map(vec![ (Expression::string("\"Foo\""), Expression::number("1")), (Expression::string("\"Bar\""), Expression::number("2")) ]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&map_expr, &mut env);

        assert!(result == Ok(json![ { "Foo": 1, "Bar": 2 } ]));
    }

    #[test]
    fn can_index_map() {
        let map_expr            = Expression::Map(vec![ (Expression::string("\"Foo\""), Expression::number("1")), (Expression::string("\"Bar\""), Expression::number("2")) ]);
        let lookup_expr         = Expression::Index(Box::new((map_expr, Expression::string("\"Bar\""))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&lookup_expr, &mut env);

        assert!(result == Ok(json![ 2 ]));
    }

    #[test]
    fn can_evaluate_map_with_duplicate_keys() {
        let map_expr            = Expression::Map(vec![ (Expression::string("\"Foo\""), Expression::number("1")), (Expression::string("\"Foo\""), Expression::number("2")) ]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = evaluate_unbound_expression(&map_expr, &mut env);

        assert!(result == Ok(json![ { "Foo": 2 } ]));
    }

    #[test]
    fn can_evaluate_tool_call() {
        let tool_expr           = Expression::identifier("test");
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = evaluate_unbound_expression(&tool_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Success"))));
    }

    #[test]
    fn can_evaluate_apply_expression() {
        let tool_expr           = Expression::Apply(Box::new((Expression::identifier("test"), Expression::string("\"Success\""))));
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|s: String| s)));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = evaluate_unbound_expression(&tool_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Success"))));
    }
}