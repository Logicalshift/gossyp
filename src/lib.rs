//!
//! This is a library for building small tools that can be composed into bigger pieces of software.
//!
//! There are three main concepts behind it:
//!
//! ## Tools
//!
//! Tools are simply small programs that perform a single task. They accept a single piece of input,
//! in the form of a JSON object and produce a single piece of output in the form of another JSON object.
//!
//! Tools can be chained together simply by passing the output of one into the input of another. As they
//! should be able to set themselves up, they are easy to test: no code is required in most cases, just
//! the input and the expected output. Using a strict data-only means of communication means that a tool
//! can effectively run anywhere, and using a loose, dynamic data type makes it easy to write tools that
//! can work together without needing a lot of extra knowledge.
//!
//! Tools can be created by implementing the `Tool` trait, but this library provides some convenience
//! functions for making new ones from Rust functions:
//!
//! ```
//! # #[macro_use] extern crate serde_json;
//! # #[macro_use] extern crate silkthread_base;
//! # #[macro_use] extern crate serde;
//! # fn main() {
//! # 
//! use silkthread_base::*;
//! use silkthread_base::basic::*;
//! use serde_json::*;
//!
//! let tool = make_pure_tool(|(x, y): (i32, i32)| x+y);
//! assert!(tool.invoke_json(json![ [ 1, 2 ] ], &EmptyEnvironment::new()) == Ok(json![ 3 ]));
//! # }
//! ```
//!
//! ## The environment
//!
//! In order to invoke a tool, we need to be able to find it. This is what the environment is for: it takes
//! the name of a tool and returns an implementation.
//!
//! Using the environment to find tools make it easy to substitute one tool for another and reduces the effort
//! required to adapt a system to new circumstances.
//!
//! When tools are created, they are given a 'birth environment', where they can find other tools that they
//! may wish to invoke. This provides for static behaviour. Sometimes a tool may want more dynamic behaviour -
//! for instance to call another tool named in its input. To allow for this, tools are provided with their
//! environment when they are invoked.
//!
//! The easiest environment to use is the dynamic environment, which allows for tools to be defined on the fly:
//!
//! ```
//! use silkthread_base::basic::*;
//!
//! let env = DynamicEnvironment::new();
//! env.define("add", Box::new(make_pure_tool(|(x, y): (i32, i32)| x+y)));
//! ```
//!
//! There are some convenience functions to make tools from environments easier to work with:
//!
//! ```
//! # use silkthread_base::basic::*;
//! # 
//! # let env = DynamicEnvironment::new();
//! # env.define("add", Box::new(make_pure_tool(|(x, y): (i32, i32)| x+y)));
//! let typed_tool  = env.get_typed_tool("add").unwrap();
//! let result      = typed_tool.invoke((4, 5), &env);
//! assert!(result == Ok(9));
//! ```
//!
//! ## The orchestration layer
//!
//! Tools usually don't provide much of a feedback loop: they run, perform a single task, and then they stop.
//! To create an actual application, an 'orchestration layer' is used. This chains tools together and deals with
//! feedback. For a web app, this might be the HTML and Javascript that runs in the browser. For other apps,
//! this could be written in Rust or it could be written using a scripting language.
//!
//! With the right collection of tools, this layer can be kept comparatively simple - making it possible to
//! quickly create large applications.
//!
//! # Why use it?
//!
//! It's much easier to maintain smaller programs than larger ones. This library provides an isolation mechanism
//! that can be used to ensure that small programs stay small. It provides what's intended to be a bare minimum
//! way for programs to communicate. This makes it easier to build larger programs out of smaller ones and also
//! makes it easier to maintain and change larger programs.
//!

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

pub mod tool;
pub mod environment;
pub mod basic;

pub use self::tool::*;
pub use self::environment::*;
