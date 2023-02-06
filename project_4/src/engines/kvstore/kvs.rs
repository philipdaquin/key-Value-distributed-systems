use std::{
    path::{PathBuf, Path}, 
    collections::{HashMap, BTreeMap}, 
    process::exit, 
    io::{Write, Seek, Read}, 
    fs::File, 
    ffi::OsStr, sync::{Arc, atomic::AtomicU64}, cell::RefCell
};
use parking_lot::{Mutex, RwLock};

use crate::engines::kvstore::{
    log_writer::LogWriterWithPos, 
    log_reader::LogReaderWithPos, 
    command::{CmdMetadata, Command}
};
use crate::error::{Result, CacheError};

use crate::engines::KvsEngine;


use super::{
    super::{sorted_generation, load_log_file, log_path}, kv_reader::{KvReader, Reader}, kv_writer::{KvWriter, Writer}
};

const MEM_THRESHOLD: u64 = 1024 * 1024;

pub trait Cache  {
    fn get(&self, key: String) -> Result<Option<String>>;
    fn set(&self, key: String, value: String) -> Result<()>;
    fn open(path: impl Into<PathBuf>) -> Result<Self> where Self: Sized;
    fn remove(&self, key: String) -> Result<()>; 
    fn version();
}

#[derive(Clone)]
pub struct KvStore { 
    /// `PathBuf`
    ///  Represents a file or directory on the file system and provides method for with paths
    directory: Arc<PathBuf>,
    /// In memory key indexes to on disk values 
    /// 
    /// - `Command`: request or the representation of a request made to the database 
    /// - A `BTreeMap` in memory stores the keys and the value locations for fast query 
    index : Arc<BTreeMap<String, CmdMetadata>>,
    /// in memory Buffer Writer and writes data individually  
    writer: Arc<Mutex<KvWriter>>,
    /// In memory Buffer Reader and reads data chunks 
    /// Key: Generational Number, Value: Reader 
    reader: KvReader,
}



impl KvStore { 
     ///
    /// Open a new or existing KvStore datastore for read only access
    /// The directory and all files in it must be readable by this process
    ///  
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> where Self: Sized {
        let path = Arc::new(path.into());

        // In Memory Buffer Reader and Buffer Writer 
        let (mut reader, 
            mut index) = (HashMap::new(), BTreeMap::new());
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

      
        let kv_read = KvReader {
            reader: RwLock::new(reader),
            index: index.clone().into(),
            directory: Arc::clone(&path), 
            safe_point: Arc::new(AtomicU64::new(0))
        };
        let kv_writer = KvWriter {
            
            directory: Arc::clone(&path), 
            curr_gen,
            writer,
            uncompacted_space,
            index: Arc::new(RwLock::new(index.clone())),
            reader: kv_read.clone()
            
        };


        Ok(Self { 
            directory: Arc::new(path.to_path_buf()), 
            index: Arc::new(index), 
            reader: kv_read, 
            writer: Arc::new(Mutex::new(kv_writer)),
        })
    }
    pub fn version() {
        println!("{}", env!("CARGO_PKG_VERSION"))
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
    fn get(&self, key: String) -> Result<Option<String>> {
        
        // Check if the metadata is in memory 
        if let Some(cmd) = self.index.get(&key) { 
            // Access the latest version 
            // if let Some(buffer_file) = self.reader.reader.into_inner().get_mut(&cmd.generation_num) { 
                // buffer_file.seek(std::io::SeekFrom::Start(*position))?;
                let cmd = cmd.clone();
                // let command = buffer_file.take(*len);

                if let Command::Set(_, value) = self.reader.read_command(cmd)? { 
                    return Ok(Some(value))
                } else { 
                    Err(CacheError::KeyNotFound)
                }
            // } else { 
                // Err(CacheError::KeyNotFound)
            // }
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
    fn set(&self, key: String, value: String) -> Result<()> {
        self.writer.lock().set(key, value)

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
    fn remove(&self, key: String) -> Result<()> {
        self.writer.lock().remove(key)
    }
}
