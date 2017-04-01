pub mod print;
pub mod write_bytes;
pub mod read_line;
pub mod tool;
pub mod shared_stream;

pub use self::print::*;
pub use self::write_bytes::*;
pub use self::read_line::*;

use std::io::*;
use gossyp_base::*;
use gossyp_base::basic::*;
use self::shared_stream::*;

///
/// Tools for performing I/O
///
pub struct IoTools<ReadStream: Read+Send, WriteStream: Write+Send> {
    read_tools: ReadTools<ReadStream>,
    write_tools: WriteTools<WriteStream>
}

///
/// Tools for reading from a stream
///
pub struct ReadTools<ReadStream: Read+Send> {
    read_stream: ReadStream
}

///
/// Tools for writing to a stream
///
pub struct WriteTools<WriteStream: Write+Send> {
    write_stream: WriteStream
}

impl<ReadStream: 'static+Read+Send, WriteStream: 'static+Write+Send> IoTools<ReadStream, WriteStream> {
    ///
    /// Creates a new set of I/O tools with a read and a write stream
    ///
    pub fn new_with_streams(read: ReadStream, write: WriteStream) -> IoTools<ReadStream, WriteStream> {
        IoTools {
            read_tools:     ReadTools::new(read),
            write_tools:    WriteTools::new(write)
        }
    }
}

impl IoTools<Stdin, Stdout> {
    ///
    /// Creates a set of I/O tools for reading and writing to stdin/stdout
    ///
    pub fn new_stdio() -> IoTools<Stdin, Stdout> {
        IoTools::new_with_streams(stdin(), stdout())
    }
}

impl<ReadStream: Read+Send> ReadTools<ReadStream> {
    ///
    /// Creates a set of tools for reading from a particular stream
    ///
    pub fn new(stream: ReadStream) -> ReadTools<ReadStream> {
        ReadTools { read_stream: stream }
    }
}

impl<WriteStream: Write+Send> WriteTools<WriteStream> {
    ///
    /// Creates a set of tools for writing to a particular stream
    ///
    pub fn new(stream: WriteStream) -> WriteTools<WriteStream> {
        WriteTools { write_stream: stream }
    }
}

impl<ReadStream: 'static+Read+Send, WriteStream: 'static+Write+Send> ToolSet for IoTools<ReadStream, WriteStream> {
    fn create_tools(self, environment: &Environment) -> Vec<(String, Box<Tool>)> {
        let mut result = self.read_tools.create_tools(environment);
        result.extend(self.write_tools.create_tools(environment));

        result
    }
}

impl<ReadStream: 'static+Read+Send> ToolSet for ReadTools<ReadStream> {
    fn create_tools(self, _environment: &Environment) -> Vec<(String, Box<Tool>)> {
        let read_stream = SharedRead::new(self.read_stream);

        vec![
            (String::from(self::tool::READ_LINE), Box::new(ReadLineTool::new_with_stream(read_stream)))
        ]
    }
}

impl<WriteStream: 'static+Write+Send> ToolSet for WriteTools<WriteStream> {
    fn create_tools(self, _environment: &Environment) -> Vec<(String, Box<Tool>)> {
        let write_stream = SharedWrite::new(self.write_stream);

        vec![
            (String::from(self::tool::PRINT),       Box::new(PrintTool::<SharedWrite<WriteStream>>::new_with_stream(write_stream.clone()))),
            (String::from(self::tool::WRITE_BYTES), Box::new(WriteBytesTool::new_with_stream(write_stream.clone())))
        ]
    }
}
