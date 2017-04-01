//!
//! The script interpreter runs scripts directly from their parse tree. This is
//! a fairly slow way to run scripts but comparatively simple to implement.
//!

use std::result::Result;
use serde_json::*;

use silkthread_base::*;
use silkthread_base::basic::*;

use super::script::*;

///
/// A tool representing a script that will be interepreted
///
pub struct InterpretedScriptTool {
    statements: Vec<Script>
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
/// Script evaluation error
///
#[derive(Serialize, Deserialize)]
pub enum ScriptEvaluationError {
    /// Tried to evaluate an expression type that's not implemented yet
    ExpressionNotImplemented,

    /// Tried to evaluate a statement type that's not implemented yet
    StatementNotImplemented,

    /// Tried to look up a tool and it couldn't be found
    ToolNameNotFound,

    /// Found an expression that can't be treated as a tool where a tool name was expected
    ExpressionDoesNotEvaluateToTool,

    /// Expressions used as keys in a map must evaluate to a string
    MapKeysMustEvaluateToAString,

    /// In index expression like foo[bar], foo must be either an array, a string or a map
    IndexMustApplyToAnArrayOrAMap,

    /// When indexing an array or a string, the index must be a number
    ArrayIndexMustBeANumber,

    /// When indexing a map, the index must be a string
    MapIndexMustBeAString,

    /// When indexing an array, the index must be in the array bounds
    IndexOutOfBounds,

    /// Object value is not present in a map
    ObjectValueNotPresent
}

///
/// Creates an execution error
///
fn generate_script_error(error: ScriptEvaluationError, script: &Script) -> Value {
    json![{
        "error":                error,
        "failed-statement":     script
    }]
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

impl InterpretedScriptTool {
    ///
    /// Creates a tool that can evaluate a script
    ///
    pub fn new_script_eval_tool() -> Box<Tool> {
        Box::new(make_dynamic_tool(|script: Vec<Script>, environment: &Environment| {
            let script_tool = InterpretedScriptTool::from_statements(script);

            script_tool.invoke_json(Value::Null, environment)
        }))
    }

    ///
    /// Creates a new interpreted script tool from a set of statements
    ///
    pub fn from_statements(statements: Vec<Script>) -> InterpretedScriptTool {
        InterpretedScriptTool { statements: statements }
    }

    ///
    /// Attempts to evaluate an expression to a tool
    ///
    pub fn evaluate_expression_to_tool(expression: &Expression, environment: &mut ScriptExecutionEnvironment) -> Result<Box<Tool>, Value> {
        match expression {
            &Expression::Identifier(ref name)   => environment.parent_environment.get_json_tool(&name.matched).map_err(|_| generate_expression_error(ScriptEvaluationError::ToolNameNotFound, expression)),
            _                                   => Err(generate_expression_error(ScriptEvaluationError::ExpressionDoesNotEvaluateToTool, expression))
        }
    }

    ///
    /// Calls an expression representing a tool and calls it with the specified parameters
    ///
    pub fn call_tool(tool_name: &Expression, parameters: Value, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
        InterpretedScriptTool::evaluate_expression_to_tool(tool_name, environment).and_then(|tool| {
            // TODO: create environment for the tool to run in

            // Invoke the tool and generate the final result
            tool.invoke_json(parameters, environment.parent_environment)
        })
    }

    ///
    /// Evaluates an 'apply' expression
    ///
    pub fn apply(&(ref tool, ref parameters): &(Expression, Expression), environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
        InterpretedScriptTool::evaluate_expression(&parameters, environment).and_then(|parameters| {
            InterpretedScriptTool::call_tool(&tool, parameters, environment)
        })
    }
    
    ///
    /// Evaluates a series of expressions into an array
    ///
    pub fn evaluate_array(exprs: &Vec<Expression>, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
        let mut result = vec![];

        for expr in exprs.iter() {
            match InterpretedScriptTool::evaluate_expression(expr, environment) {
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
            let key = match InterpretedScriptTool::evaluate_expression(key_expr, environment) {
                Ok(Value::String(key))  => key,
                Ok(_)                   => return Err(generate_expression_error(ScriptEvaluationError::MapKeysMustEvaluateToAString, key_expr)),
                Err(erm)                => return Err(erm)
            };

            let value = match InterpretedScriptTool::evaluate_expression(value_expr, environment) {
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
        InterpretedScriptTool::evaluate_expression(lhs, environment)
            .and_then(|lhs_res| InterpretedScriptTool::evaluate_expression(rhs, environment).map(|rhs_res| (lhs_res, rhs_res)))
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

            &Expression::Identifier(_)          => InterpretedScriptTool::call_tool(expression, Value::Null, environment),
            &Expression::Apply(ref expr)        => InterpretedScriptTool::apply(&*expr, environment),

            &Expression::Array(ref exprs)       => InterpretedScriptTool::evaluate_array(exprs, environment),
            &Expression::Tuple(ref exprs)       => InterpretedScriptTool::evaluate_array(exprs, environment),
            &Expression::Map(ref exprs)         => InterpretedScriptTool::evaluate_map(exprs, environment),

            &Expression::Index(ref boxed_exprs) => {
                let (ref lhs, ref rhs) = **boxed_exprs;
                InterpretedScriptTool::evaluate_index(lhs, rhs, environment)
            },

            _                                   => Err(generate_expression_error(ScriptEvaluationError::ExpressionNotImplemented, expression))
        }
    }

    ///
    /// Evaluates the result of executing a single statement
    ///
    pub fn evaluate_statement(statement: &Script, environment: &mut ScriptExecutionEnvironment) -> Result<Value, Value> {
        match statement {
            &Script::RunCommand(ref expr)   => InterpretedScriptTool::evaluate_expression(expr, environment),

            _                               => Err(generate_script_error(ScriptEvaluationError::StatementNotImplemented, statement))
        }
    }
}

impl Tool for InterpretedScriptTool {
    fn invoke_json(&self, _input: Value, environment: &Environment) -> Result<Value, Value> {
        // Make the environment that this script will run in
        let mut script_environment = ScriptExecutionEnvironment::new(environment);

        // Execute the script
        let mut result = vec![];
        for statement in self.statements.iter() {
            // Evaluate the next statement
            let next_result = match Self::evaluate_statement(statement, &mut script_environment) {
                Ok(result) => result,

                // Fail immediately if any statement generates an error
                Err(fail) => return Err(fail)
            };

            // The script result is built up from the result of each statement
            // TODO: unless there's something like a return statement?
            result.push(next_result);
        }

        // Script is done
        Ok(Value::Array(result))
    }
}

///
/// Represents an execution environment for a running script
///
pub struct ScriptExecutionEnvironment<'a> {
    /// The environment where tools are drawn from
    parent_environment: &'a Environment
}

impl<'a> ScriptExecutionEnvironment<'a> {
    ///
    /// Creates a new script execution environment
    ///
    pub fn new(parent_environment: &'a Environment) -> ScriptExecutionEnvironment<'a> {
        ScriptExecutionEnvironment { parent_environment: parent_environment }
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
        let result              = InterpretedScriptTool::evaluate_expression(&string_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Foo"))));
    }

    #[test]
    fn can_evaluate_string_with_newline() {
        let string_expr         = Expression::string("\"Foo\\n\"");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&string_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Foo\n"))));
    }

    #[test]
    fn can_evaluate_string_with_quote() {
        let string_expr         = Expression::string("\"\\\"\"");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&string_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("\""))));
    }

    #[test]
    fn can_evaluate_number() {
        let num_expr            = Expression::number("42");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&num_expr, &mut env);

        assert!(result == Ok(json![ 42 ]));
    }

    #[test]
    fn can_evaluate_float_number_1() {
        let num_expr            = Expression::number("42.2");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&num_expr, &mut env);

        assert!(result == Ok(json![ 42.2 ]));
    }

    #[test]
    fn can_evaluate_float_number_2() {
        let num_expr            = Expression::number("42e1");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&num_expr, &mut env);

        assert!(result == Ok(json![ 42e1 ]));
    }

    #[test]
    fn can_evaluate_hex_number() {
        let num_expr            = Expression::number("0xabcd");
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&num_expr, &mut env);

        assert!(result == Ok(json![ 0xabcd ]));
    }

    #[test]
    fn can_evaluate_array() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&array_expr, &mut env);

        assert!(result == Ok(json![ [ 1,2,3 ] ]));
    }

    #[test]
    fn can_lookup_array_index() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let lookup_expr         = Expression::Index(Box::new((array_expr, Expression::number("1"))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&lookup_expr, &mut env);

        assert!(result == Ok(json![ 2 ]));
    }

    #[test]
    fn can_lookup_string_index() {
        let string_expr         = Expression::string("\"Abcd\"");
        let lookup_expr         = Expression::Index(Box::new((string_expr, Expression::number("2"))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&lookup_expr, &mut env);

        assert!(result == Ok(json![ "c" ]));
    }

    #[test]
    fn positve_index_can_be_out_of_range() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let lookup_expr         = Expression::Index(Box::new((array_expr, Expression::number("100"))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&lookup_expr, &mut env);

        assert!(result.is_err());
    }

    #[test]
    fn negative_index_is_out_of_range() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let lookup_expr         = Expression::Index(Box::new((array_expr, Expression::number("-1"))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&lookup_expr, &mut env);

        assert!(result.is_err());
    }

    #[test]
    fn cannot_index_array_with_string() {
        let array_expr          = Expression::Array(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let lookup_expr         = Expression::Index(Box::new((array_expr, Expression::string("\"1\""))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&lookup_expr, &mut env);

        assert!(result.is_err());
    }

    #[test]
    fn can_evaluate_tuple() {
        let tuple_expr          = Expression::Tuple(vec![Expression::number("1"), Expression::number("2"), Expression::number("3")]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&tuple_expr, &mut env);

        assert!(result == Ok(json![ [ 1,2,3 ] ]));
    }

    #[test]
    fn can_evaluate_map() {
        let map_expr            = Expression::Map(vec![ (Expression::string("\"Foo\""), Expression::number("1")), (Expression::string("\"Bar\""), Expression::number("2")) ]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&map_expr, &mut env);

        assert!(result == Ok(json![ { "Foo": 1, "Bar": 2 } ]));
    }

    #[test]
    fn can_index_map() {
        let map_expr            = Expression::Map(vec![ (Expression::string("\"Foo\""), Expression::number("1")), (Expression::string("\"Bar\""), Expression::number("2")) ]);
        let lookup_expr         = Expression::Index(Box::new((map_expr, Expression::string("\"Bar\""))));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&lookup_expr, &mut env);

        assert!(result == Ok(json![ 2 ]));
    }

    #[test]
    fn can_evaluate_map_with_duplicate_keys() {
        let map_expr            = Expression::Map(vec![ (Expression::string("\"Foo\""), Expression::number("1")), (Expression::string("\"Foo\""), Expression::number("2")) ]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = ScriptExecutionEnvironment::new(&empty_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&map_expr, &mut env);

        assert!(result == Ok(json![ { "Foo": 2 } ]));
    }

    #[test]
    fn can_call_tool() {
        let tool_expr           = Expression::identifier("test");
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = InterpretedScriptTool::call_tool(&tool_expr, Value::Null, &mut env);

        assert!(result == Ok(Value::String(String::from("Success"))));
    }

    #[test]
    fn can_evaluate_tool_call() {
        let tool_expr           = Expression::identifier("test");
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&tool_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Success"))));
    }

    #[test]
    fn can_execute_run_command() {
        let tool_expr           = Script::RunCommand(Expression::identifier("test"));
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = InterpretedScriptTool::evaluate_statement(&tool_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Success"))));
    }

    #[test]
    fn can_evaluate_apply_expression() {
        let tool_expr           = Expression::Apply(Box::new((Expression::identifier("test"), Expression::string("\"Success\""))));
        let tool_environment    = DynamicEnvironment::new();

        tool_environment.define("test", Box::new(make_pure_tool(|s: String| s)));

        let mut env             = ScriptExecutionEnvironment::new(&tool_environment);
        let result              = InterpretedScriptTool::evaluate_expression(&tool_expr, &mut env);

        assert!(result == Ok(Value::String(String::from("Success"))));
    }
}
