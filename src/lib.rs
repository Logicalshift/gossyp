//!
//! This is a base library for a simple software architecture designed to make it easy to create large
//! applications by combining small tools. The idea is nothing new: it's a core idea behind UNIX, component
//! libraries like COM+ and more recently microservices. This version is based around Rust and JSON, and it's
//! intended to strip the idea down to its bare essentials.
//!
//! When we learn to program, we're trained on small example programs because those are easy to write and
//! assess. Useful software, however, tends to be much larger and trying to apply the skills obtained through
//! writing smaller pieces of software has a tendency to result in a bit of a mess. A big issue is that the
//! high degree of coupling found in large pieces of software makes it difficult to change design descisions
//! made early on. This library provides a really very simple structure that can be used to build large
//! programs out of small parts.
//!
//! There are three parts to this architecture:
//!
//! ## Tools
//!
//! Tools are small programs that perform a single task. They take input (in the form of a JSON object) and
//! generate an output (also in the form of a JSON object). They can be invoked with no initialisation. In 
//! general, they run immediately, perform their task on their input and return immediately.
//!
//! This design makes it easy to compose tools: feed the input of one into another. The input and output of
//! a tool is easy to understand and can be used to invoke it within the same process, within a different
//! process or even over the network. Tools are an excellent target for tests: for many tools, a test
//! requires no code: just the input JSON and the expected output JSON.
//!
//! Keeping tools to simple single purpose use cases means that any given tool should remain comparatively
//! simple. Making them composable makes it so that larger tools and applications can be built from smaller
//! ones. Having it so that they only talk in terms of concrete data means that it's easy to substitute one
//! tool for another even at runtime.
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
//! ## The automation layer
//!
//! Tools usually don't provide much of a feedback loop: they run, perform a single task, and then they stop.
//! To create an actual application, an 'automation layer' is used. This chains tools together and deals with
//! feedback. For a web app, this might be the HTML and Javascript that runs in the browser. For other apps,
//! this could be written in Rust or it could be written using a scripting language.
//!
//! With the right collection of tools, this layer can be kept comparatively simple - making it possible to
//! quickly create large applications.
//!

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

pub mod tool;
pub mod environment;

pub use tool::*;
pub use environment::*;
