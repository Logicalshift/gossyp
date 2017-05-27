use tool::*;

///
/// Represents a tooling environment, which enables tools to be retrieved by name.
///
/// Environments make it possible to use tools without needing to know where the
/// precise implementation is located. Environments also provide a way to perform
/// dependency injection in other tools.
///
pub trait Environment : Send {
    ///
    /// Retrieves a tool using a JSON interface by name
    ///
    fn get_json_tool(&self, name: &str) -> Result<Box<Tool>, RetrieveToolError>;
}

///
/// The reason an environment retrieve failed
///
#[derive(Copy, Clone, Debug)]
pub enum RetrieveFailReason {
    /// Reason not listed in this enum
    Generic,

    /// A tool could not be found
    NotFound,
}

///
/// Represents an error that occurred during a request to retrieve a tool
///
#[derive(Debug)]
pub struct RetrieveToolError {
    /// The reason the failure occurred
    reason: RetrieveFailReason,

    /// A human-readable message associated with this error
    msg: String
}

impl RetrieveToolError {
    ///
    /// Creates a new error
    ///
    pub fn new(message: &str) -> RetrieveToolError {
        RetrieveToolError { reason: RetrieveFailReason::Generic, msg: String::from(message) }
    }

    ///
    /// Creates a 'tool not found' error
    ///
    pub fn not_found() -> RetrieveToolError {
        RetrieveToolError { reason: RetrieveFailReason::NotFound, msg: String::from("Tool not found") }
    }

    ///
    /// Retrieves the message for an error
    ///
    pub fn message<'a>(&'a self) -> &'a str {
        &self.msg
    }

    ///
    /// Retrieves the reason attached to this failure
    ///
    pub fn reason(&self) -> RetrieveFailReason {
        self.reason
    }
}
