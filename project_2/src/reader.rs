use std::io::{Read, BufReader, Seek, SeekFrom};
use crate::error::Result;

pub struct LogReaderWithPos<R> where R: Read + Seek { 
    pub reader: BufReader<R>,
    pub index: u64

}

impl<R> Read for LogReaderWithPos<R> where R: Read + Seek {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)?;
        self.index += 1;
        Ok(self.index as usize)
    }
}

impl<R> Seek for LogReaderWithPos<R> where R: Read + Seek {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.index = self.reader.seek(pos)?;
        Ok(self.index)
    }
}

impl<R> LogReaderWithPos<R> where R: Read + Seek { 
    fn new(mut inner: R) -> Result<Self> { 
        let index = inner.seek(SeekFrom::Current(0))?;
        Ok( Self { 
            reader: BufReader::new(inner),
            index
        })
    }
}


