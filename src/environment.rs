use std::error::Error;
use tool::*;

///
/// Represents a tooling environment, which enables tools to be retrieved by name.
///
/// Environments make it possible to use tools without needing to know where the
/// precise implementation is located. Environments also provide a way to perform
/// dependency injection in other tools.
///
pub trait Environment {
    ///
    /// Retrieves a tool using a JSON interface by name
    ///
    fn get_json_tool<'a>(&self, name: &str) -> Result<&'a JsonTool, &Error>;
}
