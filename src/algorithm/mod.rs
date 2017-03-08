pub mod compare;
pub mod sort;
pub mod tool;

pub use self::compare::*;
pub use self::sort::*;

use silkthread_base::*;
use silkthread_base::basic::*;

///
/// ToolSet containing the algorithm tools
///
pub struct AlgorithmTools { }

impl AlgorithmTools {
    pub fn new() -> AlgorithmTools {
        AlgorithmTools { }
    }
}

impl<'a> ToolSet for &'a AlgorithmTools {
    fn create_tools(self, _environment: &Environment) -> Vec<(String, Box<Tool>)> {
        vec![
            (String::from(self::tool::COMPARE_VALUES),  Box::new(CompareTool::new())),
            (String::from(self::tool::SORT),            Box::new(SortTool::new()))
        ]
    }
}

impl ToolSet for AlgorithmTools {
    fn create_tools(self, environment: &Environment) -> Vec<(String, Box<Tool>)> {
        (&self).create_tools(environment)
    }
}
