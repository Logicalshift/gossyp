use super::super::*;

///
/// Represents an environment containing no tools
///
pub struct EmptyEnvironment { }

impl EmptyEnvironment {
    ///
    /// Creates a new empty environment
    ///
    pub fn new() -> EmptyEnvironment {
        EmptyEnvironment { }
    }
}

impl Environment for EmptyEnvironment {
    fn get_json_tool(&self, _name: &str) -> Result<Box<Tool>, RetrieveToolError> {
        Err(RetrieveToolError::not_found())
    }
}
