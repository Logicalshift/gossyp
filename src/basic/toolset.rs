//!
//! A toolset is a factory that can create a number of tools for a particular environment. It's convenient for
//! initialising new environments.
//!

use std::result::Result;
use serde_json::*;

use super::super::tool::*;
use super::super::environment::*;

///
/// Tool with a name for an environment
///
pub trait NamedTool : Tool {
    ///
    /// Retrieves the name of this tool
    ///
    fn get_name<'a>(&'a self) -> &'a str;
}

///
/// Represents a factory for a set of tools
///
pub trait ToolSet {
    ///
    /// Creates the tools in this toolset
    ///
    fn create_tools(&self, environment: &Environment) -> Vec<Box<NamedTool>>;
}

impl<T: Tool> Tool for (String, T) {
    #[inline]
    fn invoke_json(&self, input: Value, environment: &Environment) -> Result<Value, Value> {
        let (_, ref tool) = *self;
        tool.invoke_json(input, environment)
    }
}

impl<T: Tool> NamedTool for (String, T) {
    #[inline]
    fn get_name<'a>(&'a self) -> &'a str {
        let (ref name, _) = *self;
        &**name
    }
}

#[cfg(test)]
mod test {
    use std::result::Result;
    use serde_json::*;

    use super::*;
    use super::super::functional_tool::*;
    use super::super::empty_environment::*;

    #[test]
    fn can_invoke_named_tool() {
        let empty       = EmptyEnvironment::new();
        let base_tool   = make_tool(|x: i32| -> Result<i32, ()> { Ok(x+1) });
        let named_tool  = (String::from("name"), base_tool);

        let result = named_tool.invoke_json(json![ 2 ], &empty);
        assert!(result == Ok(json![ 3 ]));
    }

    #[test]
    fn can_get_tool_name() {
        let base_tool   = make_tool(|x: i32| -> Result<i32, ()> { Ok(x+1) });
        let named_tool  = (String::from("name"), base_tool);

        let name = named_tool.get_name();
        assert!(name == "name");
    }
}
