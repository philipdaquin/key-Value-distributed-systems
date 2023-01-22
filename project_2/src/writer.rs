use std::{io::{Seek, BufWriter, Write, SeekFrom}, path::Path, fs::{OpenOptions, File}};
use crate::{error::Result, reader::LogReaderWithPos};

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

    fn new(mut inner: W) -> Result<Self> { 
        let index = inner.seek(SeekFrom::Current(0))?;
        Ok( Self { 
            writer: BufWriter::new(inner),
            index
        })
    }

    fn new_log_file(path: &Path, gen: u64) -> Result<LogWriterWithPos<File>> { 
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .write(true)
            .open(path)?;

        let log_writer = LogWriterWithPos::new(file)?;  

        Ok(log_writer)
    }
   
}