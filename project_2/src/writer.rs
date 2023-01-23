use std::{io::{Seek, BufWriter, Write, SeekFrom}, path::Path, fs::{OpenOptions, File}, collections::HashMap};
use crate::{error::Result, reader::LogReaderWithPos};

/// 
/// LogWriterWithPos (Buffer Cache)
/// In an LFS, buffering is used to improve the performance of write and read operations
/// 
/// `BufWriter`
/// Keep an in memory buffer of data and writes it to an underlying writer 
/// in larger, in frequent batchs, rather than writing each piece of data individually
/// 
/// This improve performance by reducing the number of disk I/O operation and 
///     and by allowing the LFS to write data in a more controlled manner 
/// 
/// 
/// `index`
/// Keeps track of current position in the file 
/// 
pub struct LogWriterWithPos<W> where W: Write + Seek { 
    pub writer: BufWriter<W>, 
    pub index: u64
}

impl<W> Write for LogWriterWithPos<W> where W: Write + Seek { 
    /// Write a buffer into this witer, return many bytes were written
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)?;
        self.index +=1;
        Ok(self.index as usize)
    }
    /// Flushes the data to disk, ensuring that all intermediately buffered contents reach 
    /// 
    /// their destination 
    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}  

impl<W> Seek for LogWriterWithPos<W> where W: Write + Seek {
    /// Seeking always writes out the internal before seeking 
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.index = self.writer.seek(pos)?;
        Ok(self.index)
    }
}

impl<W> LogWriterWithPos<W> where W: Write + Seek { 

    pub fn new(mut inner: W) -> Result<Self> { 
        let index = inner.seek(SeekFrom::Current(0))?;
        Ok( Self { 
            writer: BufWriter::new(inner),
            index
        })
    }

    pub fn new_log_file(path: &Path, gen: u64, readers: &mut HashMap<u64, LogReaderWithPos<File>>) -> Result<LogWriterWithPos<File>> { 
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .write(true)
            .open(path)?;
        let log_writer = LogWriterWithPos::new(file)?;  


        readers.insert(gen, LogReaderWithPos::new(File::open(&path)?)?);

        Ok(log_writer)
    }
   
}