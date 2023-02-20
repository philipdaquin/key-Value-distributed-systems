use std::{
    io::{Seek, BufWriter, Write, SeekFrom, Read}, 
    path::{Path, PathBuf}, fs::{OpenOptions, File}, 
    collections::{HashMap, BTreeMap}, cell::RefCell, sync::Arc
};
use parking_lot::{Mutex, RwLock};

use crate::{error::{Result, CacheError}, engines::sorted_generation };
use crate::engines::kvstore::log_reader::LogReaderWithPos;

use super::{command::CmdMetadata, log_writer::{LogWriterWithPos}, kv_reader::KvReader};
use crate::engines::kvstore::command::Command;
use super::super::{load_log_file, log_path };





const MEM_THRESHOLD: u64 = 1024 * 1024;

pub trait Writer { 
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn remove(&mut self, key: String) -> Result<()>; 
    fn compact(&mut self) -> Result<()>;
    fn new_log_file(&mut self, gen: u64) -> Result<LogWriterWithPos<File>>;

}
#[derive(Debug)]
pub struct KvWriter { 

    /// `PathBuf`
    ///  Represents a file or directory on the file system and provides method for with paths
    pub directory: Arc<PathBuf>,

    /// Locates position of inode of file
    /// 
    /// Current generation number is a monotononicaly increasing integer that is 
    /// assigned to each data file when it is created 
    /// -   It is use to version control the data file and it is incremented each time the data file 
    ///     is updated or rewritten 
    /// 
    /// -   Older version is the lower generation number, which is then considered state   
    pub curr_gen: u64,

    /// 
    /// Writer 
    /// - Is the write handle to the current log file
    /// - So any write needs to mutable access to `writer`, adn the compaction
    /// needs to change the `writer` and the `curr_gen`
    pub writer: LogWriterWithPos<File>,

    /// The log Reader 
    pub reader: KvReader,

    /// The in memory index from jey to log pointer
    pub index : Arc<RwLock<BTreeMap<String, CmdMetadata>>>,

    /// The number of bytes representing the 'stale' data
    pub uncompacted_space: u64
}

// impl Clone for KvWriter { 
//     fn clone(&self) -> Self {
//         Self {
//             curr_gen: self.curr_gen.clone(),
//             writer: self.writer,
//             uncompacted_space: self.uncompacted_space.clone(),
//             directory: self.directory.clone(),
//             reader: self.reader.clone(),
//             index: self.index.clone(),
//         }
//     }
// }

impl Writer for KvWriter { 
    fn set(&mut self, key: String, value: String) -> Result<()> {

        let command = Command::Set(key.clone(), value);

        // Represent the command
        let start = self.writer.index;
        // Serialise the command to I/O 
        serde_json::to_writer(&mut self.writer, &command).expect("");

        // Buffer forced to write to disk immediately
        self.writer.flush()?;

        // Insert into the in memory key and value positions of command
        if let Command::Set(key, _) = command { 
            
            if let Some(prev_cmd) = self.index.read().get(&key) { 
                self.uncompacted_space += prev_cmd.len;
            }
            
            self.index.write().insert(key, (self.curr_gen, start..self.writer.index).into());

        }
        
        


        if self.uncompacted_space > MEM_THRESHOLD { 
            self.compact()?;
        }

        Ok(())

    }
    ///
    /// Compaction:
    ///     - Is the process of reducing the size of the database by remove stale commands
    ///     from the log 
    fn compact(&mut self) -> Result<()> { 
        // Create a new generation number for the compacted log file 
        let compact_gen = self.curr_gen + 1;
        self.curr_gen += 2;

        // Create a new log file for the current generation 
        self.writer = self.new_log_file(self.curr_gen)?;

        // Keep track of the new position in the compacted log 
        let mut new = 0;

        // Create a new log file for the compacted generation 
        let mut compact_writer = self.new_log_file(compact_gen)?;

        for cmd in &mut self.index.write().values_mut() { 

            // Check if the log reader exists 
            if let Some(reader) = self.reader.reader.write().get_mut(&cmd.generation_num) { 
                if reader.index != cmd.position { 
                    reader.seek(std::io::SeekFrom::Start(cmd.position))?;
                }
                let mut entry_reader = reader.take(cmd.len);
                
                let len = std::io::copy(&mut entry_reader, &mut compact_writer)?;
                
                // Update the entry 
                *cmd = (compact_gen, new..new + len).into();

                // increment the position 
                new += len
            }
        }
        compact_writer.flush()?;

        // Collect stale generation numbers
        let stale_gens = sorted_generation(&self.directory)?
            .into_iter()
            .filter(|f| f < &compact_gen);

        // Collect stale generation numbers 
        for stale in stale_gens { 
            std::fs::remove_file(log_path(&self.directory, stale))?;
        }
        // reset the stale files 
        self.uncompacted_space = 0;

        Ok(())
        
    }
    ///
    /// Delete a key from Kvstore
    /// 
    /// Error: `KeyNotFound` is the given key is not found 
    /// 
    /// Process 
    /// 1. First, it checks if the data is in memory cache (buffer cache), else we return KeyNotFound Error
    /// 
    /// 2. We overwrite the command with Remove Command in buffer then we update the disk.
    /// 
    /// 3. WE update our original in memory cache 
    /// 
    fn remove(&mut self, key: String) -> Result<()> {
        
        if self.index.read().contains_key(&key) { 

            let command = Command::Remove(key);

            // Serialise the command to I/O 
            serde_json::to_writer(&mut self.writer, &command).expect("");

            // Buffer forced to write to disk immediately
            self.writer.writer.flush()?;

            if let super::command::Command::Remove(val) = command { 
                self.index.write().remove(&val).expect("Key not Found!");
            }
            Ok(())
        } else { 
            Err(CacheError::KeyNotFound.into())
        }
    }
    fn new_log_file(&mut self, gen: u64) -> Result<LogWriterWithPos<File>> {
        LogWriterWithPos::<File>::new_log_file(&self.directory, gen)
    }
    

}
