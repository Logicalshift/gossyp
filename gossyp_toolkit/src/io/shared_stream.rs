use std::io::*;
use std::sync::*;
use std::fmt::Arguments;

///
/// Stream that can be shared with multiple tools
///
pub struct SharedRead<TStream: 'static+Read+Send> {
    stream: Arc<Mutex<TStream>>
}

impl<TStream: 'static+Read+Send> SharedRead<TStream> {
    ///
    /// Creates a new shared stream
    ///
    pub fn new(stream: TStream) -> SharedRead<TStream> {
        SharedRead { stream: Arc::new(Mutex::new(stream)) }
    }
}

impl<TStream: 'static+Read+Send> Clone for SharedRead<TStream> {
    fn clone(&self) -> SharedRead<TStream> {
        SharedRead { stream: self.stream.clone() }
    }
}

impl<TStream: 'static+Read+Send> Read for SharedRead<TStream> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut stream = self.stream.lock().unwrap();

        stream.read(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let mut stream = self.stream.lock().unwrap();

        stream.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        let mut stream = self.stream.lock().unwrap();

        stream.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let mut stream = self.stream.lock().unwrap();

        stream.read_exact(buf)
    }
}

///
/// Stream that can be shared with multiple tools
///
pub struct SharedWrite<TStream: 'static+Write+Send> {
    stream: Arc<Mutex<TStream>>
}

impl<TStream: 'static+Write+Send> SharedWrite<TStream> {
    ///
    /// Creates a new shared stream
    ///
    pub fn new(stream: TStream) -> SharedWrite<TStream> {
        SharedWrite { stream: Arc::new(Mutex::new(stream)) }
    }
}

impl<TStream: 'static+Write+Send> Clone for SharedWrite<TStream> {
    fn clone(&self) -> SharedWrite<TStream> {
        SharedWrite { stream: self.stream.clone() }
    }
}

impl<TStream: 'static+Write+Send> Write for SharedWrite<TStream> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut stream = self.stream.lock().unwrap();

        stream.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        let mut stream = self.stream.lock().unwrap();

        stream.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        let mut stream = self.stream.lock().unwrap();

        stream.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: Arguments) -> Result<()> { 
        let mut stream = self.stream.lock().unwrap();

        stream.write_fmt(fmt)
    }
}
