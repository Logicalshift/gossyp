//!
//! A dynamic environment is used when we want to be able define more tools later on.
//!
//! A dynamic environment is modified by a tool: `define-tool` will take the name of a tool in its
//! execution environment and define it with a new name in the dynamic environment it belongs to.
//!

use std::sync::*;
use std::collections::*;
use std::result::Result;
use serde_json::*;

use super::super::tool::*;
use super::super::environment::*;
use super::functional_tool::*;
use super::list_tools::*;
use super::toolset::*;

///
/// Input to the `define-tool` tool
///
#[derive(Serialize, Deserialize)]
pub struct DefineToolInput {
    /// Name of the tool in the execution environment to copy to the dynamic environment
    pub source_name: String,

    /// Name that should be given to the tool in the dynamic environment (or None if the name should be left the same)
    pub target_name: Option<String>
}

///
/// Input to the `undefine-tool` tool
///
#[derive(Serialize, Deserialize)]
pub struct UndefineToolInput {
    pub name: String
}

impl DefineToolInput {
    pub fn new(source_name: &str, target_name: Option<&str>) -> DefineToolInput {
        DefineToolInput { 
            source_name: String::from(source_name),
            target_name: target_name.map(|n| String::from(n)) 
        }
    }
}

impl UndefineToolInput {
    pub fn new(name: &str) -> UndefineToolInput {
        UndefineToolInput {
            name: String::from(name)
        }
    }
}

///
/// Tool from a dynamic environment
///
#[derive(Clone)]
struct DynamicTool {
    tool: Arc<Box<Tool>>
}

impl Tool for DynamicTool {
    fn invoke_json(&self, input: Value, environment: &Environment) -> Result<Value, Value> {
        self.tool.invoke_json(input, environment)
    }
}

impl DynamicTool {
    fn new(tool: Box<Tool>) -> DynamicTool {
        DynamicTool { tool: Arc::new(tool) }
    }
}

///
/// Structure used to store the tools in a dynamic environment
///
struct DynamicToolMap {
    tools: HashMap<String, DynamicTool>,

    // Whether or not the three built-in tools have been flagged as undefined
    undefined_list:     bool,
    undefined_define:   bool,
    undefined_undefine: bool
}

impl DynamicToolMap {
    fn new() -> DynamicToolMap {
        DynamicToolMap { 
            tools:              HashMap::new(),
            undefined_list:     false,
            undefined_define:   false,
            undefined_undefine: false         
        }
    }
}

#[derive(Clone)]
pub struct DynamicEnvironment {
    /// The tools defined in this environment
    tools: Arc<Mutex<DynamicToolMap>>
}

impl DynamicEnvironment {
    pub fn new() -> DynamicEnvironment {
        DynamicEnvironment { 
            tools: Arc::new(Mutex::new(DynamicToolMap::new()))
        }
    }
}

impl DynamicEnvironment {
    ///
    /// Defines a new tool in this environment
    ///
    pub fn define(&self, name: &str, tool: Box<Tool>) {
        let mut map = self.tools.lock().unwrap();
        map.tools.insert(String::from(name), DynamicTool::new(tool));
    }

    ///
    /// Imports a ToolSet into this environment
    ///
    pub fn import<TToolSet: ToolSet>(&self, toolset: TToolSet) {
        for tool_and_name in toolset.create_tools(self) {
            let (name, tool) = tool_and_name;

            self.define(&name, tool);
        }
    }

    ///
    /// Undefines a tool and returns whether or not it was present in the map
    ///
    pub fn undefine(&self, name: &str) -> bool {
        // Remove from the map
        let mut map     = self.tools.lock().unwrap();
        let last_value  = map.tools.remove(&String::from(name));

        let mut removed = last_value.is_some();

        // Undefine the 'internal' tools
        match name {
            super::tool_name::DEFINE_TOOL => {
                removed = removed || !map.undefined_define;
                map.undefined_define = true;
            },

            super::tool_name::UNDEFINE_TOOL => {
                removed = removed || !map.undefined_undefine;
                map.undefined_undefine = true;
            },

            super::tool_name::LIST_TOOLS => {
                removed = removed || !map.undefined_list;
                map.undefined_list = true;
            },

            _ => ()
        }
        
        removed
    }

    ///
    /// Copies a tool from a source environment into this dynamic environment
    ///
    pub fn define_tool(&self, source_name: &str, target_name: &str, source_environment: &Environment) -> Result<(), Value> {
        let source_tool = source_environment.get_json_tool(source_name);

        match source_tool {
            Ok(source_tool) => {
                self.define(target_name, source_tool);

                Ok(())
            },

            Err(erm) => {
                Err(json![{
                    "error":        "Could not find source tool",
                    "description":  erm.message()
                }])
            }
        }
    }

    ///
    /// Lists the tools in this environment
    ///
    pub fn list_tools(&self) -> ListToolsResult {
        // Collect the names from the map
        let map = self.tools.lock().unwrap();
        let mut defined_names: Vec<String> = map.tools.keys().map(|s| s.clone()).collect();

        // We also define define-tool, undefine-tool and list-tools in a dynamic environment
        if !map.undefined_define    { defined_names.push(String::from(super::tool_name::DEFINE_TOOL)); }
        if !map.undefined_undefine  { defined_names.push(String::from(super::tool_name::UNDEFINE_TOOL)); }
        if !map.undefined_list      { defined_names.push(String::from(super::tool_name::LIST_TOOLS)); }

        // Remove duplicates
        defined_names.sort();
        defined_names.dedup();

        ListToolsResult::with_name_strings(defined_names)
    }
}

impl Environment for DynamicEnvironment {
    fn get_json_tool(&self, name: &str) -> Result<Box<Tool>, RetrieveToolError> {
        // Try to get the tool from the map
        let map = self.tools.lock().unwrap();
        let tool = map.tools.get(name);

        // Always use the mapped tool if available (so it's possible to redefine define-tool and list-tools if we want)
        match tool {
            Some(tool) => Ok(Box::new(tool.clone())),

            None => {
                match name {
                    super::tool_name::DEFINE_TOOL => {
                        if !map.undefined_define {
                            // Cloning the environment creates a new reference to the map that we can use in the tool
                            let target_environment = self.clone();

                            // Generate a define-tool tool when this is requested (calls through to define_tool)
                            Ok(Box::new(make_dynamic_tool(move |input: DefineToolInput, source_environment| {
                                target_environment.define_tool(&input.source_name.clone(), &input.target_name.unwrap_or(input.source_name), source_environment)
                            })))
                        } else {
                            Err(RetrieveToolError::not_found())
                        }
                    },

                    super::tool_name::LIST_TOOLS => {
                        if !map.undefined_list {
                            // Cloning the environment creates a new reference to the map that we can use in the tool
                            let target_environment = self.clone();

                            // List the tools on request
                            Ok(Box::new(make_pure_tool(move |_: ()| {
                                target_environment.list_tools()
                            })))
                        } else {
                            Err(RetrieveToolError::not_found())
                        }
                    }

                    super::tool_name::UNDEFINE_TOOL => {
                        if !map.undefined_undefine {
                            // Cloning the environment creates a new reference to the map that we can use in the tool
                            let target_environment = self.clone();

                            // Create an undefine tool
                            Ok(Box::new(make_pure_tool(move |input: UndefineToolInput| {
                                target_environment.undefine(&input.name)
                            })))
                        } else {
                            Err(RetrieveToolError::not_found())
                        }
                    }

                    _ => Err(RetrieveToolError::not_found())
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::empty_environment::*;
    use super::super::static_environment::*;
    use super::super::basic_toolset::*;

    #[test]
    fn can_list_tools() {
        let env         = DynamicEnvironment::new();
        let list_tools  = env.get_typed_tool("list-tools").unwrap();
        let list_result = list_tools.invoke((), &env);

        assert!(list_result == Ok(ListToolsResult::with_names(vec![ "define-tool", "list-tools", "undefine-tool" ])));
    }

    #[test]
    fn can_define_tool() {
        // Create a dynamic environment
        let dynamic_env = DynamicEnvironment::new();

        // Initially there is no tool with this name...
        assert!(dynamic_env.get_json_tool("test").is_err());

        // Define a new tool
        dynamic_env.define("test", Box::new(make_pure_tool(|x: i32| x+1)));

        // Should now exist
        let new_tool = dynamic_env.get_typed_tool("test");
        assert!(new_tool.is_ok());
        assert!(new_tool.unwrap().invoke(2, &dynamic_env) == Ok(3));
    }

    #[test]
    fn can_import_toolset() {
        // Create a dynamic environment
        let dynamic_env = DynamicEnvironment::new();
        let toolset = BasicToolSet::from(vec![
            ("test", make_pure_tool(|x: i32| x+1)),
            ("test2", make_pure_tool(|x: i32| x+2)),
            ("test3", make_pure_tool(|x: i32| x+3)),
        ]);

        // Initially there are no tools
        assert!(dynamic_env.get_json_tool("test").is_err());

        // Import the toolset
        dynamic_env.import(toolset);

        // Should now exist
        let new_tool = dynamic_env.get_typed_tool("test");
        assert!(new_tool.is_ok());
        assert!(new_tool.unwrap().invoke(2, &dynamic_env) == Ok(3));
    }

    #[test]
    fn can_redefine_tool() {
        // Create a dynamic environment
        let dynamic_env = DynamicEnvironment::new();

        // Initially there is no tool with this name...
        assert!(dynamic_env.get_json_tool("test").is_err());

        // Define a new tool, then redefine it
        dynamic_env.define("test", Box::new(make_pure_tool(|x: i32| x+1)));
        dynamic_env.define("test", Box::new(make_pure_tool(|x: i32| x+2)));

        // Should now exist
        let new_tool = dynamic_env.get_typed_tool("test");
        assert!(new_tool.is_ok());
        assert!(new_tool.unwrap().invoke(2, &dynamic_env) == Ok(4));
    }

    #[test]
    fn can_undefine_tool() {
        // Create a dynamic environment
        let dynamic_env = DynamicEnvironment::new();

        // Define a new tool
        dynamic_env.define("test", Box::new(make_pure_tool(|x: i32| x+1)));

        // Should exist
        assert!(dynamic_env.get_json_tool("test").is_ok());

        // Undefine it, check that it no longer exists
        let was_undefined = dynamic_env.undefine("test");
        assert!(was_undefined);
        assert!(dynamic_env.get_json_tool("test").is_err());

        // Should not be able to undefine it again
        let was_undefined_again = dynamic_env.undefine("test");
        assert!(!was_undefined_again);
    }

    #[test]
    fn can_undefine_define_tool() {
        // Create a dynamic environment
        let dynamic_env = DynamicEnvironment::new();

        // Should exist
        assert!(dynamic_env.get_json_tool("define-tool").is_ok());

        // Undefine it, check that it no longer exists
        let was_undefined = dynamic_env.undefine("define-tool");
        assert!(was_undefined);
        assert!(dynamic_env.get_json_tool("define-tool").is_err());

        // Should not be able to undefine it again
        let was_undefined_again = dynamic_env.undefine("define-tool");
        assert!(!was_undefined_again);
    }

    #[test]
    fn can_undefine_list_tools() {
        // Create a dynamic environment
        let dynamic_env = DynamicEnvironment::new();

        // Should exist
        assert!(dynamic_env.get_json_tool("list-tools").is_ok());

        // Undefine it, check that it no longer exists
        let was_undefined = dynamic_env.undefine("list-tools");
        assert!(was_undefined);
        assert!(dynamic_env.get_json_tool("list-tools").is_err());

        // Should not be able to undefine it again
        let was_undefined_again = dynamic_env.undefine("list-tools");
        assert!(!was_undefined_again);
    }

    #[test]
    fn can_undefine_undefine_tool() {
        // Create a dynamic environment
        let dynamic_env = DynamicEnvironment::new();

        // Should exist
        assert!(dynamic_env.get_json_tool("undefine-tool").is_ok());

        // Undefine it, check that it no longer exists
        let was_undefined = dynamic_env.undefine("undefine-tool");
        assert!(was_undefined);
        assert!(dynamic_env.get_json_tool("undefine-tool").is_err());

        // Should not be able to undefine it again
        let was_undefined_again = dynamic_env.undefine("undefine-tool");
        assert!(!was_undefined_again);
    }

    #[test]
    fn can_define_tool_using_tool() {
        // Create a dynamic environment
        let dynamic_env = DynamicEnvironment::new();
        let define_tool = dynamic_env.get_typed_tool("define-tool").unwrap();

        // Then a static environment to copy our tool from
        let new_env     = StaticEnvironment::from_toolset(BasicToolSet::from(vec![
            ("test", make_pure_tool(|x: i32| x+1))
        ]), &EmptyEnvironment::new());

        // Initially there is no tool with this name...
        assert!(dynamic_env.get_json_tool("test").is_err());

        // Define a new tool
        let define_result = define_tool.invoke(DefineToolInput::new("test", None), &new_env);
        assert!(define_result == Ok(()));

        // Should now exist
        let new_tool = dynamic_env.get_typed_tool("test");
        assert!(new_tool.is_ok());
        assert!(new_tool.unwrap().invoke(2, &dynamic_env) == Ok(3));
    }

    #[test]
    fn can_undefine_tool_using_tool() {
        // Create a dynamic environment
        let dynamic_env     = DynamicEnvironment::new();
        let undefine_tool   = dynamic_env.get_typed_tool("undefine-tool").unwrap();

        // Define a new tool
        dynamic_env.define("test", Box::new(make_pure_tool(|x: i32| x+1)));

        // Should exist
        assert!(dynamic_env.get_json_tool("test").is_ok());

        // Undefine it, check that it no longer exists
        let was_undefined = undefine_tool.invoke(UndefineToolInput::new("test"), &dynamic_env);
        assert!(was_undefined == Ok(true));
        assert!(dynamic_env.get_json_tool("test").is_err());

        // Should not be able to undefine it again
        let was_undefined_again = undefine_tool.invoke(UndefineToolInput::new("test"), &dynamic_env);
        assert!(was_undefined_again == Ok(false));
    }

    #[test]
    fn can_define_tool_with_new_name() {
        // Create a dynamic environment
        let dynamic_env = DynamicEnvironment::new();
        let define_tool = dynamic_env.get_typed_tool("define-tool").unwrap();

        // Then a static environment to copy our tool from
        let new_env     = StaticEnvironment::from_toolset(BasicToolSet::from(vec![
            ("test", make_pure_tool(|x: i32| x+1))
        ]), &EmptyEnvironment::new());

        // Initially there is no tool with this name...
        assert!(dynamic_env.get_json_tool("test").is_err());
        assert!(dynamic_env.get_json_tool("new-tool").is_err());

        // Define a new tool
        let define_result = define_tool.invoke(DefineToolInput::new("test", Some("new-tool")), &new_env);
        assert!(define_result == Ok(()));

        // Should now exist
        assert!(dynamic_env.get_json_tool("test").is_err());
        let new_tool = dynamic_env.get_typed_tool("new-tool");
        assert!(new_tool.is_ok());
        assert!(new_tool.unwrap().invoke(2, &dynamic_env) == Ok(3));
    }

    #[test]
    fn can_replace_define_tool() {
        // Create a dynamic environment
        let dynamic_env = DynamicEnvironment::new();
        let define_tool = dynamic_env.get_typed_tool("define-tool").unwrap();

        // Then a static environment to copy our tool from
        let new_env     = StaticEnvironment::from_toolset(BasicToolSet::from(vec![
            ("test", make_pure_tool(|x: i32| x+1))
        ]), &EmptyEnvironment::new());

        // Define a new tool, overwriting the 'define-tool' tool
        let define_result = define_tool.invoke(DefineToolInput::new("test", Some("define-tool")), &new_env);
        assert!(define_result == Ok(()));

        // Define-tool should now be our new tool instead of its default implementation
        let new_tool = dynamic_env.get_typed_tool("define-tool");
        assert!(new_tool.is_ok());
        assert!(new_tool.unwrap().invoke(2, &dynamic_env) == Ok(3));
    }

    #[test]
    fn can_replace_list_tools() {
        // Create a dynamic environment
        let dynamic_env = DynamicEnvironment::new();
        let define_tool = dynamic_env.get_typed_tool("define-tool").unwrap();

        // Then a static environment to copy our tool from
        let new_env     = StaticEnvironment::from_toolset(BasicToolSet::from(vec![
            ("test", make_pure_tool(|x: i32| x+1))
        ]), &EmptyEnvironment::new());

        // Define a new tool, overwriting the 'list-tools' tool
        let define_result = define_tool.invoke(DefineToolInput::new("test", Some("list-tools")), &new_env);
        assert!(define_result == Ok(()));

        // List-tools should now be our new tool instead of its default implementation
        let new_tool = dynamic_env.get_typed_tool("list-tools");
        assert!(new_tool.is_ok());
        assert!(new_tool.unwrap().invoke(2, &dynamic_env) == Ok(3));
    }

    #[test]
    fn new_tools_are_added_to_list() {
        // Create a dynamic environment
        let dynamic_env = DynamicEnvironment::new();
        let define_tool = dynamic_env.get_typed_tool("define-tool").unwrap();
        let list_tools  = dynamic_env.get_typed_tool("list-tools").unwrap();

        // List is initially just the two tools
        let initial_list_result = list_tools.invoke((), &dynamic_env);
        assert!(initial_list_result == Ok(ListToolsResult::with_names(vec![ "define-tool", "list-tools", "undefine-tool" ])));

        // Then a static environment to copy our tool from
        let new_env     = StaticEnvironment::from_toolset(BasicToolSet::from(vec![
            ("test", make_pure_tool(|x: i32| x+1))
        ]), &EmptyEnvironment::new());

        // Initially there is no tool with this name...
        assert!(dynamic_env.get_json_tool("test").is_err());
        assert!(dynamic_env.get_json_tool("new-tool").is_err());

        // Define a new tool
        let define_result = define_tool.invoke(DefineToolInput::new("test", Some("new-tool")), &new_env);
        assert!(define_result == Ok(()));

        // Should now be in the list
        let final_list_result = list_tools.invoke((), &dynamic_env);
        assert!(final_list_result == Ok(ListToolsResult::with_names(vec![ "define-tool", "list-tools", "new-tool", "undefine-tool" ])));
    }

    #[test]
    fn undefining_define_removes_from_the_list() {
        // Create a dynamic environment
        let dynamic_env = DynamicEnvironment::new();

        // Should exist
        assert!(dynamic_env.get_json_tool("define-tool").is_ok());

        // Undefine it, check that it no longer exists
        dynamic_env.undefine("define-tool");
        assert!(dynamic_env.get_json_tool("define-tool").is_err());

        assert!(dynamic_env.list_tools() == ListToolsResult::with_names(vec![ "list-tools", "undefine-tool" ]))
    }
}
