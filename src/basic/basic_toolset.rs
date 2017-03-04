use super::toolset::*;
use super::super::environment::*;

///
/// Represents a simple toolset that just returns a constant set of tools
///
pub struct BasicToolSet {
    tools: Vec<Box<NamedTool>>
}

impl BasicToolSet {
    pub fn from<T: NamedTool+'static>(source: Vec<T>) -> BasicToolSet {
        let mut result: Vec<Box<NamedTool>> = vec![];

        for item in source {
            result.push(Box::new(item));
        }

        BasicToolSet { tools: result }
    }
}

impl ToolSet for BasicToolSet {
    fn create_tools(self, _environment: &Environment) -> Vec<Box<NamedTool>> {
        self.tools
    }
}
