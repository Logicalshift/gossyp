/// Tool that returns an array of all the tools in an environment
pub const LIST_TOOLS: &'static str = "list-tools";

/// Tool that defines another tool from the execution environment (into its source environment)
pub const DEFINE_TOOL: &'static str = "define-tool";

/// Tool that removes a tool from the source environment
pub const UNDEFINE_TOOL: &'static str = "undefine-tool";
