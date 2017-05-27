//!
//! The print tool displays a JSON string on the current terminal
//!

use std::result::Result;
use std::error::Error;
use std::io::*;
use std::sync::*;
use serde_json::*;
use gossyp_base::*;

///
/// Tool that prints out text for its parameter to a stream
///
pub struct PrintTool<Stream: Write+Send> {
    stream: Mutex<Stream>
}

impl PrintTool<Stdout> {
    ///
    /// Creates a new print tool that writes to stdout
    ///
    pub fn new() -> PrintTool<Stdout> {
        PrintTool::<Stdout>::new_with_stream(stdout())
    }

}

impl<Stream: Write+Send> PrintTool<Stream> {
    ///
    /// Creates a new print tool that will write to a particular stream
    ///
    pub fn new_with_stream<TStream: Write+Send>(stream: TStream) -> PrintTool<TStream> {
        PrintTool { stream: Mutex::new(stream) }
    }
}

impl<Stream: Write+Send> Tool for PrintTool<Stream> {
    fn invoke_json(&self, input: Value, _environment: &Environment) -> Result<Value, Value> {
        // Decide what to print
        let print_string = match input {
            Value::String(ref s) => {
                // If the input is just a string, then we just print that
                s.clone()
            },

            other_value => {
                // Other values are formatted as serde_json
                to_string_pretty(&other_value).unwrap_or(String::from("<Error>"))
            }
        };

        // Acquire the stream for printing
        let mut target = self.stream.lock().unwrap();

        // Write out the string as UTF-8
        let write_result = target.write(print_string.as_bytes());

        if let Err(erm) = write_result {
            return Err(json![ {
                "error":        "Write failed",
                "description":  erm.description()
            } ]);
        }

        // We flush the stream as we're 'printing' at this point
        let flush_result = target.flush();

        if let Err(erm) = flush_result {
            return Err(json![ {
                "error":        "Flush failed",
                "description":  erm.description()
            } ]);
        }

        // We always succeed
        Ok(Value::Null)
    }
}
