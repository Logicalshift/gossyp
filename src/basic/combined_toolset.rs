//!
//! Combined toolsets make it possible to merge many toolsets into one
//!

use super::toolset::*;
use super::super::tool::*;
use super::super::environment::*;

///
/// Represents a combined toolset
///
pub struct CombinedToolSet<First: ToolSet, Second: ToolSet> {
    toolsets: (First, Second)
}

///
/// Trait implemented by toolsets that can be combined
///
pub trait CombineToolSet<CombineWith: ToolSet> where Self: ToolSet+Sized {
    ///
    /// Returns a new toolset with both the tools from this toolset and another
    ///
    fn combine(self, combine_with: CombineWith) -> CombinedToolSet<Self, CombineWith>;
}

impl<Source: ToolSet+'static, Target: ToolSet+'static> CombineToolSet<Target> for Source {
    fn combine(self, combine_with: Target) -> CombinedToolSet<Source, Target> {
        CombinedToolSet { toolsets: (self, combine_with) }
    }
}

impl<Source: ToolSet, Target: ToolSet> ToolSet for CombinedToolSet<Source, Target> {
    fn create_tools(self, environment: &Environment) -> Vec<(String, Box<Tool>)> {
        let (first, second) = self.toolsets;

        let mut first_tools = first.create_tools(environment);
        let second_tools    = second.create_tools(environment);

        first_tools.extend(second_tools);
        first_tools
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::basic_toolset::*;
    use super::super::functional_tool::*;
    use super::super::empty_environment::*;

    #[test]
    fn can_combine_toolsets() {
        let first       = BasicToolSet::from(vec![("test1", make_pure_tool(|x:i32| x+1))]);
        let second      = BasicToolSet::from(vec![("test2", make_pure_tool(|x:i32| x+1))]);
        let combined    = first.combine(second);

        let tools           = combined.create_tools(&EmptyEnvironment::new());
        let mut tool_names  = tools.iter().map(|&(ref name, _)| name.clone());

        assert!(tool_names.next() == Some(String::from("test1")));
        assert!(tool_names.next() == Some(String::from("test2")));
        assert!(tool_names.next() == None);
    }
}
