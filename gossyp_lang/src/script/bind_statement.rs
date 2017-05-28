use std::result::Result;

use serde_json::*;

use super::script::*;
use super::bound_script::*;
use super::bind_expression::*;
use super::binding_environment::*;
use super::script_interpreter::*;

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
fn bind_sequence(sequence: &Vec<Script>, binding_environment: &mut BindingEnvironment) -> Result<Vec<BoundScript>, Value> {
    let mut result = vec![];

    for statement in sequence {
        result.push(bind_statement(statement, binding_environment)?);
    }

    Ok(result)
}

///
/// Binds a new variable name
///
fn bind_variable_name(name: &ScriptToken, script: &Script, binding_environment: &mut BindingEnvironment) -> Result<u32, Value> {
    let binding = binding_environment.allocate_variable(&name.matched);
    
    match binding {
        Ok(value)                       => Ok(value),
        Err(BindingError::AlreadyInUse) => Err(generate_statement_error(ScriptEvaluationError::VariableNameAlreadyInUse, script))
    }
}

///
/// Binds a statement to an environment
///
pub fn bind_statement(script: &Script, binding_environment: &mut BindingEnvironment) -> Result<BoundScript, Value> {
    use self::BoundScript::*;

    match *script {
        Script::RunCommand(ref expr)    => Ok(RunCommand(bind_expression(expr, binding_environment)?)),
        Script::Sequence(ref parts)     => Ok(Sequence(bind_sequence(parts, binding_environment)?)),
        Script::Var(ref name, ref expr) => Ok(Var(bind_variable_name(name, script, binding_environment)?, bind_expression(expr, binding_environment)?, name.clone())),

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
        let sequence_statement  = Script::Sequence(vec![Script::RunCommand(Expression::string("\"Foo\""))]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = BindingEnvironment::new(&empty_environment);

        let bound               = bind_statement(&sequence_statement, &mut *env);

        assert!(match bound { Ok(BoundScript::Sequence(_)) => true, _ => false });
    }
    
    #[test]
    fn can_bind_var_expression() {
        let var_statement       = Script::Var(ScriptToken::identifier("test"), Expression::number("42"));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = BindingEnvironment::new(&empty_environment);

        let bound               = bind_statement(&var_statement, &mut *env);

        assert!(match bound { Ok(BoundScript::Var(0, BoundExpression::Value(_, _), _)) => true, _ => false });
    }
}
