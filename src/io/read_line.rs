//!
//! Reads a line of text from a stream
//!

use std::result::Result;
use std::error::Error;
use std::io::*;
use std::sync::*;
use serde_json::*;
use silkthread_base::*;

#[derive(Serialize, Deserialize)]
pub struct ReadLineResult {
    pub eof:    bool,
    pub line:   String
}

///
/// Tool for reading text from a stream
///
pub struct ReadLineTool<Stream: 'static+Read+Send> {
    stream: Mutex<Stream>
}

impl<Stream: 'static+Read+Send> ReadLineTool<Stream> {
    ///
    /// Creates a new read line tool
    ///
    pub fn new_with_stream(stream: Stream) -> ReadLineTool<Stream> {
        ReadLineTool { stream: Mutex::new(stream) }
    }
}

impl<Stream: 'static+Read+Send> Tool for ReadLineTool<Stream> {
    fn invoke_json(&self, _input: Value, _environment: &Environment) -> Result<Value, Value> {
        // We hold the stream until we've read the entire line
        let mut stream = self.stream.lock().unwrap();

        // Read UTF-8 from the stream
        let mut result_utf8 = vec![];
        let reached_eof;
        loop {
            // Read the next character
            let mut chr     = [0; 1];
            let read_result = stream.read(&mut chr);

            // Error out if we reach an error condition
            if let Err(erm) = read_result {
                let before_error = String::from_utf8_lossy(&result_utf8);

                return Err(json![{
                    "error":                "I/O error",
                    "description":          erm.description(),
                    "read_before_error":    before_error
                }]);
            }

            // Stop if we reach EOF
            if read_result.unwrap_or(0) == 0 {
                reached_eof = true;
                break;
            }

            // Stop if it's a newline
            if chr[0] == b'\n' {
                reached_eof = false;
                break;
            }

            // Keep building the result otherwise
            result_utf8.push(chr[0]);
        }

        // Generate the final result
        Ok(json![{
            "eof":  reached_eof,
            "line": String::from_utf8_lossy(&result_utf8)
        }])
    }
}
