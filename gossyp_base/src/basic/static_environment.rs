//!
//! A static environment is used to store a fixed set of tools.
//!

use std::sync::Arc;
use std::collections::HashMap;
use std::result::Result;
use serde_json::*;

use super::super::tool::*;
use super::super::environment::*;
use super::toolset::*;

///
/// A static environment just contains a fixed set of tools
///
pub struct StaticEnvironment {
    /// The tools in this environment
    tools: HashMap<String, Arc<Box<Tool>>>
}

///
/// Wrapper for a tool from a static environment
///
struct StaticEnvironmentTool {
    /// Reference to the tool within the environment
    tool: Arc<Box<Tool>>
}

impl Tool for StaticEnvironmentTool {
    fn invoke_json(&self, input: Value, environment: &Environment) -> Result<Value, Value> {
        (**self.tool).invoke_json(input, environment)
    }
}

impl Environment for StaticEnvironment {
    ///
    /// Retrieves a tool using a JSON interface by name
    ///
    fn get_json_tool(&self, name: &str) -> Result<Box<Tool>, RetrieveToolError> {
        let tool = self.tools.get(&String::from(name));

        match tool {
            Some(tool)  => Ok(Box::new(StaticEnvironmentTool { tool: tool.clone() })),
            None        => Err(RetrieveToolError::not_found())
        }
    }
}

impl StaticEnvironment {
    ///
    /// Creates a new static environment from a toolset
    ///
    pub fn from_toolset<T: ToolSet>(set: T, environment: &Environment) -> StaticEnvironment {
        let tools           = set.create_tools(environment);
        let mut tool_hash   = HashMap::new();

        for tool_and_name in tools {
            let (name, tool) = tool_and_name;
            
            tool_hash.insert(name, Arc::new(tool));
        }

        StaticEnvironment { tools: tool_hash }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::functional_tool::*;
    use super::super::empty_environment::*;
    use super::super::basic_toolset::*;

    #[test]
    fn can_get_tool_by_name() {
        let toolset = BasicToolSet::from(vec![
            ("add-1", make_pure_tool(|x: i32| { x+1 })),
            ("add-2", make_pure_tool(|x: i32| { x+2 }))
        ]);
        let environment = StaticEnvironment::from_toolset(toolset, &EmptyEnvironment::new());

        assert!(environment.get_json_tool("add-1").is_ok());
        assert!(environment.get_json_tool("add-2").is_ok());
    }

    #[test]
    fn error_for_missing_tool() {
        let toolset = BasicToolSet::from(vec![
            ("add-1", make_pure_tool(|x: i32| { x+1 })),
            ("add-2", make_pure_tool(|x: i32| { x+2 }))
        ]);
        let environment = StaticEnvironment::from_toolset(toolset, &EmptyEnvironment::new());

        assert!(environment.get_json_tool("add-3").is_err());
    }

    #[test]
    fn tools_are_right() {
        let toolset = BasicToolSet::from(vec![
            ("add-1", make_pure_tool(|x: i32| { x+1 })),
            ("add-2", make_pure_tool(|x: i32| { x+2 }))
        ]);
        let environment = StaticEnvironment::from_toolset(toolset, &EmptyEnvironment::new());

        let add1 = environment.get_json_tool("add-1").unwrap();
        let add2 = environment.get_json_tool("add-2").unwrap();

        assert!(add1.invoke_json(json![ 2 ], &environment) == Ok(json![ 3 ]));
        assert!(add2.invoke_json(json![ 2 ], &environment) == Ok(json![ 4 ]));
    }
}
