use std::{io::{Read, BufReader, Seek, SeekFrom}, fs::File, collections::HashMap};
use crate::error::Result;

///
/// LogReaderWithPos
/// 
/// Buffering can also improve read performance. When data is read from a buffer,
/// it is already in memory, which allows for faster access than reading data directly disk
/// 
/// In addition, by using a technique like hinting, LFS can quickly locate a specific key in the 
/// buffer instead of scanning the entire log, which improves read performance
/// 
#[derive(Debug)]
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
    pub fn new(mut inner: R) -> Result<Self> { 
        let index = inner.seek(SeekFrom::Current(0))?;
        Ok( Self { 
            reader: BufReader::new(inner),
            index
        })
    }
}
