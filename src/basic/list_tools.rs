use super::toolset::*;
use super::functional_tool::*;
use super::super::environment::*;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ListToolsResult {
    pub names: Vec<String>
}

impl ListToolsResult {
    ///
    /// Creates a new list tools result with a particular set of names
    ///
    pub fn with_names(names: Vec<&str>) -> ListToolsResult {
        ListToolsResult {
            names: names.iter().map(|s| String::from(*s)).collect()
        }
    }
}

///
/// Toolset that adds a list-tools tool
///
pub struct ToolSetWithList<TToolSet: ToolSet> {
    source: TToolSet
}

///
/// Adds the list-tools tool to a toolset so we can obtain a list
///
pub fn add_list_to_toolset<TToolSet: ToolSet>(toolset: TToolSet) -> ToolSetWithList<TToolSet> {
    ToolSetWithList { source: toolset }
}

impl<TToolSet: ToolSet> ToolSet for ToolSetWithList<TToolSet> {
    ///
    /// Creates the tools in this toolset
    ///
    fn create_tools(self, environment: &Environment) -> Vec<Box<NamedTool>> {
        // Create the initial set of tools
        let mut result = self.source.create_tools(environment);

        // Get the names of the tools
        let mut names = vec![];
        for tool in result.iter() {
            names.push(String::from(tool.get_name()));
        }

        // Names will include list-tools, and should have no duplicates
        names.push(String::from(super::tool_name::LIST_TOOLS));
        names.sort();
        names.dedup();

        // Create the list-tools tool
        let list_tools = make_pure_tool(move |_: ()| { ListToolsResult { names: names.clone() } });

        result.push(Box::new((super::tool_name::LIST_TOOLS, list_tools)));

        result
    }
}

#[cfg(test)]
mod test {
    use serde_json::*;              // Rust says unused but the json! macro won't work without this
    use super::*;
    use super::super::empty_environment::*;
    use super::super::static_environment::*;
    use super::super::basic_toolset::*;
    use super::super::functional_tool::*;

    #[test]
    fn can_list_tools() {
        let toolset = BasicToolSet::from(vec![
            ("add-1", make_pure_tool(|x: i32| { x+1 })),
            ("add-2", make_pure_tool(|x: i32| { x+2 }))
        ]);
        let withlist    = add_list_to_toolset(toolset);
        let environment = StaticEnvironment::from_toolset(withlist, &EmptyEnvironment::new());

        let list_tool   = environment.get_typed_tool("list-tools").unwrap();
        let list_result = list_tool.invoke((), &environment);

        assert!(list_result == Ok(ListToolsResult::with_names(vec!["add-1", "add-2", "list-tools"])));
    }

    #[test]
    fn will_ignore_duplicates() {
        let toolset = BasicToolSet::from(vec![
            ("add-1", make_pure_tool(|x: i32| { x+1 })),
            ("add-1", make_pure_tool(|x: i32| { x+1 })),
            ("add-2", make_pure_tool(|x: i32| { x+2 }))
        ]);
        let withlist    = add_list_to_toolset(toolset);
        let environment = StaticEnvironment::from_toolset(withlist, &EmptyEnvironment::new());

        let list_tool   = environment.get_typed_tool("list-tools").unwrap();
        let list_result = list_tool.invoke((), &environment);

        assert!(list_result == Ok(ListToolsResult::with_names(vec!["add-1", "add-2", "list-tools"])));
    }
}
