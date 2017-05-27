use super::toolset::*;
use super::super::tool::*;
use super::super::environment::*;

///
/// Represents a simple toolset that just returns a constant set of tools
///
pub struct BasicToolSet {
    tools: Vec<(String, Box<Tool>)>
}

impl BasicToolSet {
    pub fn from<T: NamedTool+'static>(source: Vec<T>) -> BasicToolSet {
        let mut result: Vec<(String, Box<Tool>)> = vec![];

        for item in source {
            result.push((String::from(item.get_name()), Box::new(item)));
        }

        BasicToolSet { tools: result }
    }
}

impl ToolSet for BasicToolSet {
    fn create_tools(self, _environment: &Environment) -> Vec<(String, Box<Tool>)> {
        self.tools
    }
}
