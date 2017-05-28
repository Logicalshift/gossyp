use std::result::Result;

use serde_json::*;

use super::script::*;
use super::bound_script::*;
use super::bind_expression::*;
use super::binding_environment::*;
use super::script_interpreter::*;

use self::BoundScript::*;

///
/// Creates an execution error relating to an script statement
///
fn generate_statement_error(error: ScriptEvaluationError, script: &Script) -> Value {
    json![{
        "error":            error,
        "failed-statement": script
    }]
}

///
/// Binds a sequnce in a script
///
fn bind_sequence(sequence: &Vec<Script>, binding_environment: &BindingEnvironment) -> Result<Vec<BoundScript>, Value> {
    let mut result = vec![];

    for statement in sequence {
        result.push(bind_statement(statement, binding_environment)?);
    }

    Ok(result)
}

///
/// Binds a statement to an environment
///
pub fn bind_statement(script: &Script, binding_environment: &BindingEnvironment) -> Result<BoundScript, Value> {
    match *script {
        Script::RunCommand(ref expr)    => Ok(RunCommand(bind_expression(expr, binding_environment)?)),
        Script::Sequence(ref parts)     => Ok(Sequence(bind_sequence(parts, binding_environment)?)),

        _ => unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use gossyp_base::basic::*;
    use super::*;
    
    #[test]
    fn can_bind_simple_statement() {
        let string_statement    = Script::RunCommand(Expression::string("\"Foo\""));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = BindingEnvironment::new(&empty_environment);

        let bound               = bind_statement(&string_statement, &mut *env);

        assert!(match bound { Ok(BoundScript::RunCommand(BoundExpression::Value(Value::String(s), _))) => s == "Foo", _ => false });
    }
    
    #[test]
    fn can_bind_simple_sequence() {
        let string_statement    = Script::Sequence(vec![Script::RunCommand(Expression::string("\"Foo\""))]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = BindingEnvironment::new(&empty_environment);

        let bound               = bind_statement(&string_statement, &mut *env);

        assert!(match bound { Ok(BoundScript::Sequence(_)) => true, _ => false });
    }
}
