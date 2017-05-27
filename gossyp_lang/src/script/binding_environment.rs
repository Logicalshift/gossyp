use std::result::Result;
use std::collections::HashMap;

use gossyp_base::RetrieveToolError;
use gossyp_base::Environment;
use gossyp_base::Tool;

///
/// Errors that can occur when binding a variable
///
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum BindingError {
    /// The requested variable name is already in use
    AlreadyInUse,
}

///
/// Result of binding to an environment
///
pub enum BindingResult {
    /// Name maps to a variable
    Variable(u32),

    /// Name maps to a tool in the external environment
    Tool(Box<Tool>),

    /// Got an error while mapping a tool (binding isn't a tool or a variable)
    Error(RetrieveToolError)
}

///
/// A binding environment can be used to allocate variable values that will be used
/// during execution of a script.
///
pub struct BindingEnvironment<'a> {
    /// The parent of this binding environment (for creating binding hierarchies)
    parent: Option<&'a mut BindingEnvironment<'a>>,

    /// The external environment that contains tools we can bind to
    environment: &'a Environment,

    /// The next variable value that will be alllocated
    next_to_allocate: u32,

    /// The current set of binding allocations
    bindings: HashMap<String, u32>
}

impl<'a> BindingEnvironment<'a> {
    ///
    /// Creates a new binding environment. New variables will be mapped from 0
    ///
    pub fn new(environment: &'a Environment) -> BindingEnvironment<'a> {
        BindingEnvironment { 
            parent:             None, 
            environment:        environment, 
            next_to_allocate:   0, 
            bindings:           HashMap::new() 
        }
    }

    ///
    /// Creates a new sub-environment, where new variable names can be a
    ///
    pub fn create_sub_environment<'b: 'a>(&'b mut self) -> BindingEnvironment<'a> {
        BindingEnvironment { 
            next_to_allocate:   self.next_to_allocate,
            environment:        self.environment,
            parent:             Some(self), 
            bindings:           HashMap::new()
        }
    }

    ///
    /// Allocates a new variable location in this binding (without assigning a name to it)
    ///
    pub fn allocate_location(&mut self) -> u32 {
        if let Some(ref mut parent) = self.parent {
            // Allocate up the chain if this is a chained environment
            let allocation = parent.allocate_location();
            self.next_to_allocate = allocation + 1;

            allocation
        } else {
            // If there's no parent, just allocate directly
            let allocation          = self.next_to_allocate;
            self.next_to_allocate   = allocation + 1;

            allocation
        }
    }

    ///
    /// Allocates a new variable
    ///
    pub fn allocate_variable(&mut self, name: &str) -> Result<u32, BindingError> {
        let name_string = String::from(name);

        if self.bindings.contains_key(&name_string) {
            // Variable name is already taken
            Err(BindingError::AlreadyInUse)
        } else {
            // Can assign this location to value
            let allocation = self.allocate_location();
            self.bindings.insert(name_string, allocation);

            Ok(allocation)
        }
    }

    ///
    /// Looks up a name in this binding environment
    ///
    pub fn lookup(&self, name: &str) -> BindingResult {
        if let Some(variable) = self.bindings.get(&String::from(name)) {
            // Try to retrieve as a variable directly from this environment
            BindingResult::Variable(*variable)
        } else if let Some(ref parent) = self.parent {
            // Try to retrieve from the parent environment if there is one
            parent.lookup(name)
        } else {
            // Try to retrieve from the environment
            let tool_or_error = self.environment.get_json_tool(name);

            match tool_or_error {
                Ok(tool)    => BindingResult::Tool(tool),
                Err(error)  => BindingResult::Error(error)
            }
        }
    }

    ///
    /// Returns the number of variables used in this environment
    ///
    pub fn get_number_of_variables(&self) -> u32 {
        self.next_to_allocate
    }
}

#[cfg(test)]
mod test {
    use gossyp_base::basic::*;
    use super::*;

    #[test]
    fn can_allocate_location() {
        let empty_environment   = EmptyEnvironment::new();
        let mut binding         = BindingEnvironment::new(&empty_environment);

        assert!(binding.allocate_location() == 0);
    }

    #[test]
    fn can_allocate_many_locations() {
        let empty_environment   = EmptyEnvironment::new();
        let mut binding         = BindingEnvironment::new(&empty_environment);

        assert!(binding.allocate_location() == 0);
        assert!(binding.allocate_location() == 1);
        assert!(binding.allocate_location() == 2);
    }

    #[test]
    fn can_get_number_of_variables() {
        let empty_environment   = EmptyEnvironment::new();
        let mut binding         = BindingEnvironment::new(&empty_environment);

        binding.allocate_location();
        binding.allocate_location();
        binding.allocate_location();

        assert!(binding.get_number_of_variables() == 3);

        binding.allocate_location();

        assert!(binding.get_number_of_variables() == 4);
    }

    #[test]
    fn allocating_locations_in_child_environments_also_does_in_parent() {
        let empty_environment   = EmptyEnvironment::new();
        let mut binding         = BindingEnvironment::new(&empty_environment);

        assert!(binding.allocate_location() == 0);

        {
            let mut child_environment = binding.create_sub_environment();

            assert!(child_environment.allocate_location() == 1);
        }

        // TODO! (Need to fix lifetime) assert!(binding.allocate_location() == 2);
    }
    
    #[test]
    fn can_allocate_variable_name() {
        let empty_environment   = EmptyEnvironment::new();
        let mut binding         = BindingEnvironment::new(&empty_environment);

        assert!(binding.allocate_variable("test") == Ok(0));
    }
    
    #[test]
    fn cannot_allocate_variable_name_twice() {
        let empty_environment   = EmptyEnvironment::new();
        let mut binding         = BindingEnvironment::new(&empty_environment);

        assert!(binding.allocate_variable("test") == Ok(0));
        assert!(binding.allocate_variable("test") == Err(BindingError::AlreadyInUse));
    }
    
    #[test]
    fn can_lookup_variable_name() {
        let empty_environment   = EmptyEnvironment::new();
        let mut binding         = BindingEnvironment::new(&empty_environment);

        binding.allocate_variable("test");
        
        assert!(match binding.lookup("test") { BindingResult::Variable(v) => v == 0, _ => false });
    }
    
    #[test]
    fn can_lookup_many_variable_names() {
        let empty_environment   = EmptyEnvironment::new();
        let mut binding         = BindingEnvironment::new(&empty_environment);

        binding.allocate_variable("test1");
        binding.allocate_variable("test2");
        binding.allocate_variable("test3");
        
        assert!(match binding.lookup("test1") { BindingResult::Variable(v) => v == 0, _ => false });
        assert!(match binding.lookup("test2") { BindingResult::Variable(v) => v == 1, _ => false });
        assert!(match binding.lookup("test3") { BindingResult::Variable(v) => v == 2, _ => false });
    }
    
    #[test]
    fn can_lookup_tool_name() {
        let tool_environment = DynamicEnvironment::new();
        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut binding = BindingEnvironment::new(&tool_environment);
        
        assert!(match binding.lookup("test") { BindingResult::Tool(_) => true, _ => false });
    }
    
    #[test]
    fn variable_name_has_precedence_over_tool() {
        let tool_environment = DynamicEnvironment::new();
        tool_environment.define("test", Box::new(make_pure_tool(|_: ()| "Success")));

        let mut binding = BindingEnvironment::new(&tool_environment);

        binding.allocate_variable("test");
        
        assert!(match binding.lookup("test") { BindingResult::Variable(v) => v == 0, _ => false });
    }
}
