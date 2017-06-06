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
        result.push(bind_statement_without_allocation(statement, binding_environment)?);
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
/// Retrieves an existing variable name
///
fn get_variable_name(name: &ScriptToken, script: &Script, binding_environment: &mut BindingEnvironment) -> Result<u32, Value> {
    let binding = binding_environment.lookup(&name.matched);
    
    match binding {
        BindingResult::Variable(value)  => Ok(value),
        BindingResult::Tool(_)          => Err(generate_statement_error(ScriptEvaluationError::WasExpectingAVariable, script)),
        BindingResult::Error(_)         => Err(generate_statement_error(ScriptEvaluationError::VariableNameNotFound, script))
    }
}

///
/// Binds a statement to an environment (does not allocate space for variables)
///
fn bind_statement_without_allocation(script: &Script, binding_environment: &mut BindingEnvironment) -> Result<BoundScript, Value> {
    use self::BoundScript::*;

    match *script {
        Script::RunCommand(ref expr)        => Ok(RunCommand(bind_expression(expr, binding_environment)?)),
        Script::Sequence(ref parts)         => Ok(Sequence(bind_sequence(parts, binding_environment)?)),
        Script::Var(ref name, ref expr)     => Ok(Var(bind_variable_name(name, script, binding_environment)?, bind_expression(expr, binding_environment)?, name.clone())),
        Script::Assign(ref name, ref expr)  => Ok(Assign(get_variable_name(name, script, binding_environment)?, bind_expression(expr, binding_environment)?, name.clone())),

        _ => unimplemented!()
    }
}

///
/// Binds a statement to an environment
///
pub fn bind_statement(script: &Script, binding_environment: &mut BindingEnvironment) -> Result<BoundScript, Value> {
    use self::BoundScript::*;

    // We store the initial number of variables so we can see if any allocation is required
    let initial_variable_count  = binding_environment.get_number_of_variables();

    // Then bind the statements with no further allocation
    let bound_script            = bind_statement_without_allocation(script, binding_environment)?;

    // Transform to an allocation script if required
    let final_variable_count = binding_environment.get_number_of_variables();
    if initial_variable_count < final_variable_count {
        Ok(AllocateVariables(final_variable_count, Box::new(bound_script)))
    } else {
        Ok(bound_script)
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
        let mut env             = BindingEnvironment::from_environment(&empty_environment);

        let bound               = bind_statement(&string_statement, &mut *env);

        assert!(match bound { Ok(BoundScript::RunCommand(BoundExpression::Value(Value::String(s), _))) => s == "Foo", _ => false });
    }
    
    #[test]
    fn can_bind_simple_sequence() {
        let sequence_statement  = Script::Sequence(vec![Script::RunCommand(Expression::string("\"Foo\""))]);
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = BindingEnvironment::from_environment(&empty_environment);

        let bound               = bind_statement(&sequence_statement, &mut *env);

        assert!(match bound { Ok(BoundScript::Sequence(_)) => true, _ => false });
    }
    
    #[test]
    fn can_bind_var_expression() {
        let var_statement       = Script::Var(ScriptToken::identifier("test"), Expression::number("42"));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = BindingEnvironment::from_environment(&empty_environment);

        let bound               = bind_statement(&var_statement, &mut *env);

        assert!(match bound { Ok(BoundScript::AllocateVariables(1, _)) => true, _ => false });

        if let Ok(BoundScript::AllocateVariables(_, boundvar)) = bound {
            assert!(match *boundvar { BoundScript::Var(0, BoundExpression::Value(_, _), _) => true, _ => false });
        } else {
            assert!(false);
        }
    }
    
    #[test]
    fn can_bind_assign_expression() {
        let assign_statement    = Script::Assign(ScriptToken::identifier("test"), Expression::number("42"));
        let empty_environment   = EmptyEnvironment::new();
        let mut env             = BindingEnvironment::from_environment(&empty_environment);

        assert!(env.allocate_variable("test") == Ok(0));

        let bound               = bind_statement(&assign_statement, &mut *env);

        assert!(bound.is_ok());
        assert!(match bound { Ok(BoundScript::Assign(0, BoundExpression::Value(_, _), _)) => true, _ => false });
    }
}
