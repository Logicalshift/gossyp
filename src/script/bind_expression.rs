use std::rc::*;
use std::result::Result;
use serde_json::*;

use super::script::*;
use super::bound_script::*;
use super::script_interpreter::*;

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
/// Creates an execution error relating to an expression
///
fn generate_expression_error(error: ScriptEvaluationError, expr: &Expression) -> Value {
    json![{
        "error":                error,
        "failed-expression":    expr
    }]
}

///
/// Generates a tool binding
///
pub fn bind_tool(tool_name: &ScriptToken, expr: &Expression, execution_environment: &ScriptExecutionEnvironment) -> Result<BoundExpression, Value> {
    if let Ok(tool) = execution_environment.get_json_tool(&tool_name.matched) {
        Ok(BoundExpression::Tool(Rc::new(tool), tool_name.clone()))
    } else {
        Err(generate_expression_error(ScriptEvaluationError::ExpressionDoesNotEvaluateToTool, expr))
    }
}

///
/// Binds a sequence of elements
///
fn bind_sequence(items: &Vec<Expression>, execution_environment: &ScriptExecutionEnvironment) -> Result<Vec<BoundExpression>, Value> {
    let mut result = vec![];

    for expr in items {
        result.push(bind_expression(expr, execution_environment)?);
    }

    Ok(result)
}

///
/// Generates an array binding
///
pub fn bind_array(items: &Vec<Expression>, execution_environment: &ScriptExecutionEnvironment) -> Result<BoundExpression, Value> {
    bind_sequence(items, execution_environment)
        .map(|array_items| BoundExpression::Array(array_items))
}

///
/// Generates a tuple binding
///
pub fn bind_tuple(items: &Vec<Expression>, execution_environment: &ScriptExecutionEnvironment) -> Result<BoundExpression, Value> {
    bind_sequence(items, execution_environment)
        .map(|tuple_items| BoundExpression::Tuple(tuple_items))
}

///
/// Generates a map binding
///
pub fn bind_map(items: &Vec<(Expression, Expression)>, execution_environment: &ScriptExecutionEnvironment) -> Result<BoundExpression, Value> {
    let mut result = vec![];

    for &(ref lexpr, ref rexpr) in items {
        let lbound = bind_expression(lexpr, execution_environment)?;
        let rbound = bind_expression(rexpr, execution_environment)?;

        result.push((lbound, rbound));
    }

    Ok(BoundExpression::Map(result))
}

///
/// Binds an index expression (a[b])
///
pub fn bind_index(index: &Box<(Expression, Expression)>, execution_environment: &ScriptExecutionEnvironment) -> Result<BoundExpression, Value> {
    let (ref tool, ref indexer) = **index;

    let bound_tool      = bind_expression(tool, execution_environment)?;
    let bound_indexer   = bind_expression(indexer, execution_environment)?;

    Ok(BoundExpression::Index(Box::new((bound_tool, bound_indexer))))
}

///
/// Binds a field access expression (a.b)
///
pub fn bind_field_access(field_access: &Box<(Expression, Expression)>, execution_environment: &ScriptExecutionEnvironment) -> Result<BoundExpression, Value> {
    unimplemented!();
}

///
/// Binds an apply expression (a(parameters))
///
pub fn bind_apply(apply: &Box<(Expression, Expression)>, execution_environment: &ScriptExecutionEnvironment) -> Result<BoundExpression, Value> {
    let (ref tool, ref parameters) = **apply;

    let bound_tool          = bind_expression(tool, execution_environment)?;
    let bound_parameters    = bind_expression(parameters, execution_environment)?;

    Ok(BoundExpression::Apply(Box::new((bound_tool, bound_parameters))))
}

///
/// Binds an expression to an environment
///
pub fn bind_expression(expr: &Expression, execution_environment: &ScriptExecutionEnvironment) -> Result<BoundExpression, Value> {
    match expr {
        &Expression::String(ref s)              => Ok(BoundExpression::Value(Value::String(unquote_string(&s.matched)), s.clone())),
        &Expression::Number(ref n)              => Ok(BoundExpression::Value(parse_number(&n.matched), n.clone())),

        &Expression::Array(ref items)           => bind_array(items, execution_environment),
        &Expression::Tuple(ref items)           => bind_tuple(items, execution_environment),
        &Expression::Map(ref items)             => bind_map(items, execution_environment),

        &Expression::Identifier(ref id)         => bind_tool(id, expr, execution_environment),
        &Expression::Index(ref indexer)         => bind_index(indexer, execution_environment),
        &Expression::FieldAccess(ref accessor)  => bind_field_access(accessor, execution_environment),
        &Expression::Apply(ref application)     => bind_apply(application, execution_environment),
    }
}

#[cfg(test)]
mod test {
    use gossyp_base::basic::*;
    use super::*;
    
    #[test]
    fn can_bind_string() {
        let string_expr         = Expression::string("\"Foo\"");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);

        assert!(match bind_expression(&string_expr, &mut env) { Ok(BoundExpression::Value(Value::String(s), _)) => s == "Foo", _ => false }); 
    }
    
    #[test]
    fn can_bind_number() {
        let string_expr         = Expression::number("42");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);

        assert!(match bind_expression(&string_expr, &mut env) { Ok(BoundExpression::Value(num, _)) => num == json![ 42 ], _ => false }); 
    }

    #[test]
    fn can_bind_tool() {
        let tool_expr           = Expression::identifier("test");
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = bind_expression(&tool_expr, &mut env);

        assert!(match result { Ok(BoundExpression::Tool(_, _)) => true, _ => false });
    }

    #[test]
    fn can_bind_array() {
        let array_expr          = Expression::Array(vec![Expression::identifier("test"), Expression::number("1")]);
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = bind_expression(&array_expr, &mut env);

        assert!(match result { Ok(BoundExpression::Array(_)) => true, _ => false });

        let array = match result { Ok(BoundExpression::Array(x)) => x, _ => vec![] };
        assert!(match array[0] { BoundExpression::Tool(_, _) => true, _ => false });
        assert!(match array[1] { BoundExpression::Value(_, _) => true, _ => false });
    }

    #[test]
    fn can_bind_tuple() {
        let tuple_expr          = Expression::Tuple(vec![Expression::identifier("test"), Expression::number("1")]);
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = bind_expression(&tuple_expr, &mut env);

        assert!(match result { Ok(BoundExpression::Tuple(_)) => true, _ => false });

        let tuple = match result { Ok(BoundExpression::Tuple(x)) => x, _ => vec![] };
        assert!(match tuple[0] { BoundExpression::Tool(_, _) => true, _ => false });
        assert!(match tuple[1] { BoundExpression::Value(_, _) => true, _ => false });
    }

    #[test]
    fn can_bind_map() {
        let map_expr            = Expression::Map(vec![(Expression::string("\"test\""), Expression::identifier("test"))]);
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = bind_expression(&map_expr, &mut env);

        assert!(match result { Ok(BoundExpression::Map(_)) => true, _ => false });

        let map = match result { Ok(BoundExpression::Map(x)) => x, _ => vec![] };
        assert!(match map[0] { (BoundExpression::Value(_, _), BoundExpression::Tool(_, _)) => true, _ => false });
    }

    #[test]
    fn can_bind_apply() {
        let apply_expr          = Expression::Apply(Box::new((Expression::string("\"test\""), Expression::identifier("test"))));
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = bind_expression(&apply_expr, &mut env);

        assert!(match result { Ok(BoundExpression::Apply(_)) => true, _ => false });
    }

    #[test]
    fn can_bind_index() {
        let index_expr          = Expression::Index(Box::new((Expression::string("\"test\""), Expression::identifier("test"))));
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = bind_expression(&index_expr, &mut env);

        assert!(match result { Ok(BoundExpression::Index(_)) => true, _ => false });
    }
}
