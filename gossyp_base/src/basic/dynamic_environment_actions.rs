use std::result::Result;

use serde::*;
use serde_json::*;

use super::super::tool::*;
use super::super::environment::*;
use super::tool_name::*;
use super::functional_tool::*;
use super::dynamic_environment::*;
use super::static_environment::*;

///
/// Retrieves a tool, or returns a JSON error indicating it doesn't exist
///
#[inline]
fn get_tool(env: &Environment, tool_name: &str) -> Result<Box<Tool>, Value> {
    match env.get_json_tool(tool_name) {
        Ok(tool)            => Ok(tool),
        Err(retrieve_error) => Err(json![{
            "error":        "Tool not found",
            "tool_name":    tool_name,
            "description":  retrieve_error.message()
        }])
    }
}

///
/// Calls the define action for a tool in this environment
///
pub fn define_new_tool(environment: &Environment, new_tool_name: &str, tool: Box<Tool>) -> Result<(), Value> {
    // Fetch the define tool for the currnet environment
    let define_tool         = TypedTool::<DefineToolInput, ()>::from(get_tool(environment, DEFINE_TOOL)?);

    // Put the tool we want to define in its own environment so we can pass it to the define tool
    let source_environment  = StaticEnvironment::from_tool(new_tool_name, tool);

    // Define this tool
    define_tool.invoke(DefineToolInput::new(new_tool_name, Some(new_tool_name)), &source_environment)?;

    Ok(())
}

/// 
/// Redefines a tool in this environment
///
pub fn alias_tool(environment: &Environment, old_tool_name: &str, new_tool_name: &str) -> Result<(), Value> {
    // Fetch the define tool for the currnet environment
    let define_tool = TypedTool::<DefineToolInput, ()>::from(get_tool(environment, DEFINE_TOOL)?);

    // Perform aliasing
    define_tool.invoke(DefineToolInput::new(old_tool_name, Some(new_tool_name)), environment)?;

    Ok(())
}

///
/// Undefines a tool in this environment
///
pub fn undefine_tool(environment: &Environment, old_tool_name: &str) -> Result<bool, Value> {
    // Fetch the define tool for the currnet environment
    let undefine_tool = TypedTool::<UndefineToolInput, bool>::from(get_tool(environment, UNDEFINE_TOOL)?);

    // Remove the tool
    Ok(undefine_tool.invoke(UndefineToolInput::new(old_tool_name), environment)?)
}

///
/// Defines a new tool from a function that doesn't produce errors in this environment
///
pub fn define_pure_tool<TIn: 'static, TOut: 'static+Serialize, F: 'static+Send+Sync+Fn(TIn) -> TOut>(environment: &Environment, new_tool_name: &str, function: F) -> Result<(), Value> 
where for<'de> TIn: Deserialize<'de> {
    define_new_tool(environment, new_tool_name, Box::new(make_pure_tool(function)))
}

///
/// Defines a new tool from a function that can produce errors in this environment
///
pub fn define_dynamic_tool<TIn: 'static, TOut: 'static+Serialize, TErr: 'static+Serialize, F: 'static+Send+Sync+Fn(TIn, &Environment) -> Result<TOut, TErr>>(environment: &Environment, new_tool_name: &str, function: F) -> Result<(), Value>
where for<'de> TIn: Deserialize<'de> {
    define_new_tool(environment, new_tool_name, Box::new(make_dynamic_tool(function)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_define_new_tool_using_convenience_function() {
        let new_tool = make_pure_tool(|x: i32| x+1);
        let env      = DynamicEnvironment::new();

        assert!(env.get_json_tool("test").is_err());
        assert!(define_new_tool(&env, "test", Box::new(new_tool)).is_ok());
        assert!(env.get_json_tool("test").is_ok());
    }

    #[test]
    fn can_define_pure_tool_using_convenience_function() {
        let env      = DynamicEnvironment::new();

        assert!(env.get_json_tool("test").is_err());
        assert!(define_pure_tool(&env, "test", |x: i32| x+1).is_ok());
        assert!(env.get_json_tool("test").is_ok());
    }

    #[test]
    fn can_define_dynamic_tool_using_convenience_function() {
        let env      = DynamicEnvironment::new();

        assert!(env.get_json_tool("test").is_err());
        assert!(define_dynamic_tool(&env, "test", |x: i32, _env| if x > 0 { Ok(x-1) } else { Err(false) }).is_ok());
        assert!(env.get_json_tool("test").is_ok());
    }

    #[test]
    fn can_alias_tool_using_convenience_function() {
        let new_tool = make_pure_tool(|x: i32| x+1);
        let env      = DynamicEnvironment::new();

        env.define("test", Box::new(new_tool));

        assert!(env.get_json_tool("test").is_ok());
        assert!(env.get_json_tool("test2").is_err());
        assert!(alias_tool(&env, "test", "test2").is_ok());
        assert!(env.get_json_tool("test2").is_ok());
    }

    #[test]
    fn can_undefine_tool_using_convenience_function() {
        let new_tool = make_pure_tool(|x: i32| x+1);
        let env      = DynamicEnvironment::new();

        env.define("test", Box::new(new_tool));

        assert!(env.get_json_tool("test").is_ok());
        assert!(undefine_tool(&env, "test") == Ok(true));
        assert!(env.get_json_tool("test").is_err());
    }
}
