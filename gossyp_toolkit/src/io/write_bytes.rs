//!
//! The write_bytes tool writes out a series of bytes (provided as an array) to its stream
//!

use std::result::Result;
use std::error::Error;
use std::io::*;
use std::sync::*;
use serde_json::*;
use gossyp_base::*;

///
/// Tool that writes out bytes to a stream
/// 
pub struct WriteBytesTool<Stream: Write+Send> {
    stream: Mutex<Stream>
}

impl<Stream: Write+Send> WriteBytesTool<Stream> {
    pub fn new_with_stream(stream: Stream) -> WriteBytesTool<Stream> {
        WriteBytesTool { stream: Mutex::new(stream) }
    }
}

impl<Stream: Write+Send> Tool for WriteBytesTool<Stream> {
    fn invoke_json(&self, input: Value, _environment: &Environment) -> Result<Value, Value> {
        let bytes = from_value::<Vec<u8>>(input);

        bytes.map(|bytes| {
            let mut stream      = self.stream.lock().unwrap();
            let write_result    = stream.write(&bytes);

            write_result
                .map(|_ok| Value::Null)
                .map_err(|erm| {
                    json![{
                        "error": "Write failed",
                        "description": erm.description()
                     }]
                })
        }).unwrap_or(Err(json![ {
            "error": "Write must be called with an array of bytes"
        } ]))
    }
}
