use std::{
    path::{PathBuf, Path}, 
    collections::{HashMap, BTreeMap}, 
    process::exit, 
    io::{Write, Seek, Read}, 
    fs::File, 
    ffi::OsStr
};
use crate::engines::kvstore::{
    writer::LogWriterWithPos, 
    reader::LogReaderWithPos, 
    command::{CmdMetadata, Command}
};
use crate::error::{Result, CacheError};
use std::fs;
use crate::engines::KvsEngine;

const MEM_THRESHOLD: u64 = 1024 * 1024;

pub trait Cache  {
    fn get(&mut self, key: String) -> Result<Option<String>>;
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn open(path: impl Into<PathBuf>) -> Result<Self> where Self: Sized;
    fn remove(&mut self, key: String) -> Result<()>; 
    fn version();
    fn compact(&mut self) -> Result<()>;
    fn new_log_file(&mut self, gen: u64) -> Result<LogWriterWithPos<File>>;
}

// #[derive(Serialize, Deserialize, Clone, Default)]
pub struct KvStore { 
    
    /// `PathBuf`
    ///  Represents a file or directory on the file system and provides method for with paths
    directory: PathBuf,

    /// Locates position of inode of file
    /// 
    /// Current generation number is a monotononicaly increasing integer that is 
    /// assigned to each data file when it is created 
    /// -   It is use to version control the data file and it is incremented each time the data file 
    ///     is updated or rewritten 
    /// 
    /// -   Older version is the lower generation number, which is then considered state   
    curr_gen: u64,

    /// In memory key indexes to on disk values 
    /// 
    /// - `Command`: request or the representation of a request made to the database 
    /// - A `BTreeMap` in memory stores the keys and the value locations for fast query 
    index : BTreeMap<String, CmdMetadata>,

    /// in memory Buffer Writer and writes data individually  
    writer: LogWriterWithPos<File>,

    /// In memory Buffer Reader and reads data chunks 
    /// Key: Generational Number, Value: Reader 
    reader: HashMap<u64, LogReaderWithPos<File>>,

    uncompacted_space: u64

}


impl KvStore { 
     ///
    /// Open a new or existing KvStore datastore for read only access
    /// The directory and all files in it must be readable by this process
    ///  
    fn open(path: impl Into<PathBuf>) -> Result<Self> where Self: Sized {
        let path = path.into();

        // In Memory Buffer Reader and Buffer Writer 
        let (mut reader, mut index) = (HashMap::new(), BTreeMap::new());
        // Current generational value 
        let curr_gen = *sorted_generation(&path)?.last().unwrap_or(&0) + 1;
        
        // Intialise writer with the current generational value and directory
        let writer = LogWriterWithPos::<File>::new_log_file(&path, curr_gen, &mut reader)?;
        
        let generation_list = sorted_generation(&path)?;

        let mut uncompacted_space = 0;

        // Initialise all readers 
        for &version in generation_list.iter() { 
            let mut log_reader = LogReaderWithPos::new(File::open(log_path(&path, version))?)?;
            
            uncompacted_space += load_log_file(version, &mut log_reader, &mut index)?;
            reader.insert(version, log_reader);
        }

        Ok(Self { 
            directory: path, 
            index, 
            reader, 
            curr_gen,  
            writer,
            uncompacted_space
        })
    }
    fn version() {
        println!("{}", env!("CARGO_PKG_VERSION"))
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

        for cmd in &mut self.index.values_mut() { 

            // Check if the log reader exists 
            if let Some(reader) = self.reader.get_mut(&cmd.generation_num) { 
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
        let stale_gens = self
            .reader
            .keys()
            .filter(|gen| gen < &&compact_gen)
            .cloned()
            .collect::<Vec<u64>>();

        // Collect stale generation numbers 
        for stale in stale_gens { 
            self.reader.remove(&stale);
            std::fs::remove_file(log_path(&self.directory, stale))?;

        }
        // reset the stale files 
        self.uncompacted_space = 0;

        Ok(())
        
    }

    fn new_log_file(&mut self, gen: u64) -> Result<LogWriterWithPos<File>> {
        LogWriterWithPos::<File>::new_log_file(&self.directory, gen, &mut self.reader)
    }

}


impl KvsEngine for KvStore {
    /// 
    /// Retrieve a value by key from a KVstore
    /// 
    /// 1. First, it checks if the data is in memory cache (buffer cache)
    /// 
    /// 2. If the data is not in memory, it will be retrieved from the disk instead 
    /// 
    /// 3. We need an efficient way to find out which SStable contains the key
    /// 
    /// 4. SSTables return the result of the data set 
    /// 
    /// 5. The result of the data set is returned to the client
    /// 
    fn get(&mut self, key: String) -> Result<Option<String>> {
        
        // Check if the metadata is in memory 
        if let Some(CmdMetadata { generation_num , position, len } ) = self.index.get(&key) { 
            // Access the latest version 
            if let Some(buffer_file) = self.reader.get_mut(generation_num) { 
                buffer_file.seek(std::io::SeekFrom::Start(*position))?;
                
                let command = buffer_file.take(*len);

                if let Command::Set(_, value) = serde_json::from_reader(command)? { 
                    return Ok(Some(value))
                } else { 
                    Err(CacheError::KeyNotFound)
                }
            } else { 
                Err(CacheError::KeyNotFound)
            }
        } else { 
            Err(CacheError::KeyNotFound)
        }
    }
    ///
    /// If the key already exists, the previous value will be overwritten
    /// 
    /// Improve performance:  of write by buffering a sequence of file changes 
    ///     in the file cache and then writing all the chnage to disk sequentially 
    /// 
    /// 1.  Create Command K / V
    /// 
    /// 2.  Serialise object into Stream I/O in memory cache 
    /// 
    /// 3.  Write into disk
    /// 
    /// 4. Set CommandPos in BTreeMap
    /// 
    fn set(&mut self, key: String, value: String) -> Result<()> {
        // Represent the command
        let start = self.writer.index;
        // Serialise the command to I/O 
        serde_json::to_writer(&mut self.writer, &Command::Set(key.clone(), value)).expect("");

        // Buffer forced to write to disk immediately
        self.writer.flush()?;

        // Insert into the in memory key and value positions of command
        self.index.insert(key, (self.curr_gen, start..self.writer.index).into());


        if self.uncompacted_space > MEM_THRESHOLD { 
            self.compact()?;
        }

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
        
        if self.index.contains_key(&key) { 

            let command = Command::Remove(key);

            // Serialise the command to I/O 
            serde_json::to_writer(&mut self.writer, &command).expect("");

            // Buffer forced to write to disk immediately
            self.writer.flush()?;

            if let Command::Remove(val) = command { 
                self.index.remove(&val).expect("Key not Found!");
            }
            Ok(())
        } else { 
            Err(CacheError::KeyNotFound.into())
        }
    }

    

}

///
/// A generation number is a ID that is used to 
/// distinguish different versions of a file or data 
/// 
fn sorted_generation(path: &Path) -> Result<Vec<u64>> { 
    let mut gen_list: Vec<u64> = fs::read_dir(&path)?
        .flat_map(|res| -> Result<_> { Ok(res?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    gen_list.sort_unstable();
    Ok(gen_list)
}

fn log_path(direction: &Path, gen: u64) -> PathBuf { 
    direction.join(format!("{}.log", gen))
}

/// 
/// Load the whole log file and store value locations in the index map 
/// 
/// Return how many bytes can be saved after a compaction 
fn load_log_file(
    generation: u64,  
    reader: &mut LogReaderWithPos<File>, 
    index: &mut BTreeMap<String, CmdMetadata>
) -> Result<u64> { 

    let mut start = reader.seek(std::io::SeekFrom::Start(0))?;
    let mut stream = serde_json::Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted_data = 0;

    while let Some(val) = stream.next() { 
        // 
        let new = stream.byte_offset() as u64;

        match val? { 

            Command::Set(key, _) => { 
                if let Some(command) = index.insert(key, (generation, start..new).into()) { 
                    uncompacted_data += command.len
                }
            }
            
            Command::Remove(key) => { 
                if let Some(command) = index.remove(&key) { 
                    uncompacted_data += command.len;
                }
                uncompacted_data += new - start;
            }
            _ => {}
        }
        start = new
    }
    Ok(uncompacted_data)
} 