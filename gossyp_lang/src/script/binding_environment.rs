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
struct BaseBindingEnvironment<'a> {
    /// The external environment that contains tools we can bind to
    environment: &'a Environment,

    /// The next variable value that will be alllocated
    next_to_allocate: u32,

    /// The current set of binding allocations
    bindings: HashMap<String, u32>
}

///
/// A child environment for a base environment
///
struct ChildBindingEnvironment<'a> {
    /// The base binding environment that this child environment is for
    base_environment: &'a mut BindingEnvironment,

    /// The current set of binding allocations
    bindings: HashMap<String, u32>
}

///
/// Trait implemented by objects that represent a binding environment
///
pub trait BindingEnvironment {
    ///
    /// Allocates a variable location without assigning a name
    ///
    fn allocate_location(&mut self) -> u32;

    ///
    /// Allocates a variable location with an assigned name (which must not 
    /// already be in use)
    ///
    fn allocate_variable(&mut self, name: &str) -> Result<u32, BindingError>;

    ///
    /// Looks up a variable location by name
    ///
    fn lookup(&self, name: &str) -> BindingResult;

    ///
    /// Returns the total number of variables allocated for this environment
    ///
    fn get_number_of_variables(&self) -> u32;

    ///
    /// Creates a sub environment
    ///
    /// This allows names to be re-used with new variables, but existing variables
    /// will continue to refer to their current locations
    ///
    fn create_sub_environment<'a>(&'a mut self) -> Box<BindingEnvironment + 'a>;
}

impl BindingEnvironment {
    ///
    /// Creates a new binding environment. New variables will be mapped from 0
    ///
    pub fn new<'a>(environment: &'a Environment) -> Box<BindingEnvironment+'a> {
        Box::new(BaseBindingEnvironment { 
            environment:        environment, 
            next_to_allocate:   0, 
            bindings:           HashMap::new() 
        })
    }
}

impl<'a> BindingEnvironment for BaseBindingEnvironment<'a> {
    ///
    /// Creates a new sub-environment, where new variable names can be a
    ///
    fn create_sub_environment<'b>(&'b mut self) -> Box<BindingEnvironment + 'b> {
        Box::new(ChildBindingEnvironment {
            base_environment:   self,
            bindings:           HashMap::new()
        })
    }

    ///
    /// Allocates a new variable location in this binding (without assigning a name to it)
    ///
    fn allocate_location(&mut self) -> u32 {
        // If there's no parent, just allocate directly
        let allocation          = self.next_to_allocate;
        self.next_to_allocate   = allocation + 1;

        allocation
    }

    ///
    /// Allocates a new variable
    ///
    fn allocate_variable(&mut self, name: &str) -> Result<u32, BindingError> {
        if self.bindings.contains_key(name) {
            // Variable name is already taken
            Err(BindingError::AlreadyInUse)
        } else {
            // Can assign this location to value
            let allocation = self.allocate_location();
            self.bindings.insert(String::from(name), allocation);

            Ok(allocation)
        }
    }

    ///
    /// Looks up a name in this binding environment
    ///
    fn lookup(&self, name: &str) -> BindingResult {
        if let Some(variable) = self.bindings.get(name) {
            // Try to retrieve as a variable directly from this environment
            BindingResult::Variable(*variable)
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
    fn get_number_of_variables(&self) -> u32 {
        self.next_to_allocate
    }
}

impl<'a> BindingEnvironment for ChildBindingEnvironment<'a> {
    fn allocate_location(&mut self) -> u32 {
        self.base_environment.allocate_location()
    }

    fn allocate_variable(&mut self, name: &str) -> Result<u32, BindingError> {
        if self.bindings.contains_key(name) {
            // Variable name is already taken
            Err(BindingError::AlreadyInUse)
        } else {
            // Can assign this location to value
            let allocation = self.allocate_location();
            self.bindings.insert(String::from(name), allocation);

            Ok(allocation)
        }
    }

    fn lookup(&self, name: &str) -> BindingResult {
        if let Some(variable) = self.bindings.get(name) {
            BindingResult::Variable(*variable)
        } else {
            self.base_environment.lookup(name)
        }
    }

    fn get_number_of_variables(&self) -> u32 {
        self.base_environment.get_number_of_variables()
    }

    fn create_sub_environment<'b>(&'b mut self) -> Box<BindingEnvironment + 'b> {
        Box::new(ChildBindingEnvironment {
            base_environment:   self,
            bindings:           HashMap::new()
        })
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

        assert!(binding.allocate_location() == 2);
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
