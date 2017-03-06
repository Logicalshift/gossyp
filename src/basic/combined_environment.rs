//!
//! Combined environment
//!
//! This is used to merge several separate environments into one.
//!

use std::result::Result;
use std::sync::*;
use serde_json::*;

use super::super::tool::*;
use super::super::environment::*;
use super::list_tools::*;
use super::empty_environment::*;
use super::functional_tool::*;

struct EnvironmentCollection {
    environments: Vec<Box<Environment>>
}

impl EnvironmentCollection {
    fn new(environments: Vec<Box<Environment>>) -> EnvironmentCollection {
        EnvironmentCollection { environments: environments }
    }
}

#[derive(Clone)]
pub struct CombinedEnvironment {
    collection: Arc<Mutex<EnvironmentCollection>>
}

impl CombinedEnvironment {
    ///
    /// Creates a new combined environment
    ///
    /// The first environment to define a tool is the one that is used in the event that more than
    /// one environment defines one. The `list-tools` tool will be changed to one that combines the
    /// results across all environments.
    ///
    pub fn from_environments(environments: Vec<Box<Environment>>) -> CombinedEnvironment {
        CombinedEnvironment { collection: Arc::new(Mutex::new(EnvironmentCollection::new(environments))) }
    }

    ///
    /// Combines the results of listing the tools across all of the environments
    ///
    pub fn list_tools(&self) -> ListToolsResult {
        let collection = self.collection.lock().unwrap();

        // List all of the tools in all the environments
        let tools = collection.environments.iter()
            .map(|env| env.get_json_tool(super::tool_name::LIST_TOOLS))
            .filter(|tool| tool.is_ok())
            .map(|tool| tool.unwrap());
        
        let results = tools.map(|tool| tool.invoke_json(Value::Null, &EmptyEnvironment::new()))
            .filter(|result| result.is_ok())
            .map(|result| result.unwrap());

        let decoded = results.map(|result| from_value::<ListToolsResult>(result).unwrap_or(ListToolsResult::with_names(vec![])).names);

        // Concatentate the results
        let mut final_result = vec![];
        for list in decoded {
            for name in list {
                final_result.push(name.clone());
            }
        }

        final_result.sort();
        final_result.dedup();

        ListToolsResult::with_name_strings(final_result)
    }
}

impl Environment for CombinedEnvironment {
    fn get_json_tool(&self, name: &str) -> Result<Box<Tool>, RetrieveToolError> {
        if name == super::tool_name::LIST_TOOLS {
            // Generate a list-tools implementation (we use a copy of this object, which will reference
            // our environments as the collection is reference counted)
            let list_environment    = self.clone();
            let list_tools          = make_pure_tool(move |_: ()| list_environment.list_tools());

            Ok(Box::new(list_tools))
        } else {
            // Return the first item in the collection that implements the specified tool name
            let collection  = self.collection.lock().unwrap();
            let item        = collection.environments.iter()
                .map(|env| env.get_json_tool(name).ok())
                .find(|env| env.is_some())
                .map(|env| env.unwrap());

            item.ok_or(RetrieveToolError::not_found())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::dynamic_environment::*;

    #[test]
    fn can_find_tool_in_first_environment() {
        let first   = DynamicEnvironment::new();
        let second  = DynamicEnvironment::new();

        first.define("first-tool", Box::new(make_pure_tool(|x:i32| x+1)));
        second.define("second-tool", Box::new(make_pure_tool(|x:i32| x+2)));

        let combined = CombinedEnvironment::from_environments(vec![ Box::new(first), Box::new(second) ]);

        assert!(combined.get_json_tool("first-tool").is_ok());
        assert!(combined.get_typed_tool("first-tool").unwrap().invoke(2, &combined) == Ok(3));
    }

    #[test]
    fn can_find_tool_in_second_environment() {
        let first   = DynamicEnvironment::new();
        let second  = DynamicEnvironment::new();

        first.define("first-tool", Box::new(make_pure_tool(|x:i32| x+1)));
        second.define("second-tool", Box::new(make_pure_tool(|x:i32| x+2)));

        let combined = CombinedEnvironment::from_environments(vec![ Box::new(first), Box::new(second) ]);

        assert!(combined.get_json_tool("second-tool").is_ok());
        assert!(combined.get_typed_tool("second-tool").unwrap().invoke(2, &combined) == Ok(4));
    }

    #[test]
    fn first_environment_overrides_second() {
        let first   = DynamicEnvironment::new();
        let second  = DynamicEnvironment::new();

        first.define("tool", Box::new(make_pure_tool(|x:i32| x+1)));
        second.define("tool", Box::new(make_pure_tool(|x:i32| x+2)));

        let combined = CombinedEnvironment::from_environments(vec![ Box::new(first), Box::new(second) ]);

        assert!(combined.get_json_tool("tool").is_ok());
        assert!(combined.get_typed_tool("tool").unwrap().invoke(2, &combined) == Ok(3));
    }

    #[test]
    fn can_list_tools() {
        let first   = DynamicEnvironment::new();
        let second  = DynamicEnvironment::new();

        first.define("first-tool", Box::new(make_pure_tool(|x:i32| x+1)));
        second.define("second-tool", Box::new(make_pure_tool(|x:i32| x+2)));

        let combined = CombinedEnvironment::from_environments(vec![ Box::new(first), Box::new(second) ]);

        assert!(combined.get_json_tool("list-tools").is_ok());
        assert!(combined.get_typed_tool("list-tools").unwrap().invoke((), &combined) == Ok(ListToolsResult::with_names(vec![ "define-tool", "first-tool", "list-tools", "second-tool", "undefine-tool" ])));
    }

    #[test]
    fn list_tools_does_not_generate_duplicates() {
        let first   = DynamicEnvironment::new();
        let second  = DynamicEnvironment::new();

        first.define("tool", Box::new(make_pure_tool(|x:i32| x+1)));
        second.define("tool", Box::new(make_pure_tool(|x:i32| x+2)));

        let combined = CombinedEnvironment::from_environments(vec![ Box::new(first), Box::new(second) ]);

        assert!(combined.get_json_tool("list-tools").is_ok());
        assert!(combined.get_typed_tool("list-tools").unwrap().invoke((), &combined) == Ok(ListToolsResult::with_names(vec![ "define-tool", "list-tools", "tool", "undefine-tool" ])));
    }
}
