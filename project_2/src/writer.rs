use std::{io::{Seek, BufWriter, Write}};


pub struct LogWriterWithPos<W> where W: Write + Seek { 
    pub writer: BufWriter<W>, 
    pub index: u64
}

impl<W> Write for LogWriterWithPos<W> where W: Write + Seek { 
    // Appends the data to the log file 
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)?;
        self.index +=1;
        Ok(self.index as usize)
    }
    // Flushes the data to disk 
    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}  

impl<W> Seek for LogWriterWithPos<W> where W: Write + Seek {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.index = self.writer.seek(pos)?;
        Ok(self.index)
    }
}