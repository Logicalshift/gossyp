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
/// Binds an expression to an environment
///
pub fn bind_expression(expr: &Expression, execution_environment: &mut ScriptExecutionEnvironment) -> Result<BoundExpression, Value> {
    match expr {
        &Expression::String(ref s)          => Ok(BoundExpression::Value(Value::String(unquote_string(&s.matched)), s.clone())),
        &Expression::Number(ref n)          => Ok(BoundExpression::Value(parse_number(&n.matched), n.clone())),

        _ => unimplemented!()
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
}
