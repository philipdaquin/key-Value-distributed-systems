use std::{path::{PathBuf, Path}, collections::{HashMap, BTreeMap}, process::exit, io::{Write, Seek}, fs::File, ffi::OsStr};
use serde::{Serialize, Deserialize};
use crate::{writer::LogWriterWithPos, reader::LogReaderWithPos, error::CacheError};
use crate::command::Command;
use crate::error::Result;
use std::fs;

pub trait Cache  {
    fn get(&self, key: String) -> Result<Option<String>>;
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn open(&self, path: impl Into<PathBuf>) -> Result<Self> where Self: Sized;
    fn remove(&mut self, key: String) -> Result<()>; 
    fn version();
}

// #[derive(Serialize, Deserialize, Clone, Default)]
pub struct KvStore { 

    directory: PathBuf,

    // Locates position of inode of file
    current: u64,

    index: BTreeMap<String, Command>,

    /// in memory Buffer Writer and writes data individually  
    writer: LogWriterWithPos<File>,

    /// In memory Buffer Reader and reads data chunks 
    reader: HashMap<u64, LogReaderWithPos<File>>
}

impl Cache for KvStore {
    
    /// 
    /// Retrieve a value by key from a KVstore
    /// 
    fn get(&self, key: String) -> Result<Option<String>> {
        if let Some(buffer) = self.index.get(&key) { 
            if let Command::Get(value) = buffer { 
               return Ok(Some(value.clone()))
            } else { 
                Err(CacheError::NotFound.into())
            }
        } else { 
            Ok(None)
        }
        
    }
    ///
    /// If the key already exists, the previous value will be overwritten
    /// 
    /// *** Improve performance of write by buffering a sequence of file changes 
    ///     in the file cache and then writing all the chnage to disk sequentially 
    fn set(&mut self, key: String, value: String) -> Result<()> {
        // Represent the command
        let command = Command::Set(key, value);
        
        // Serialise the command to I/O 
        serde_json::to_writer(&mut self.writer, &command).expect("");

        // Buffer forced to write to disk immediately
        self.writer.flush()?;
        
        Ok(())

    }

    ///
    /// Open a new or existing KvStore datastore for read only access
    /// The directory and all files in it must be readable by this process
    ///  
    fn open(&self, path: impl Into<PathBuf>) -> Result<Self> where Self: Sized {
        let path = path.into();

        let (mut reader, mut index) = (HashMap::new(), BTreeMap::new());



        Ok(Self { 
            directory: path, 
            index, 
            reader, 
        })
    }

    ///
    /// 
    /// Delete a key from Kvstore
    /// 
    fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) { 
            self.index.remove(&key);
        } else { 
            return Err(CacheError::KeyNotFound.into())
        }

        Ok(())
    }

    fn version() {
        println!("1.0")
    }
}

///
/// A generation number is a ID that is used to 
/// distinguish different versions of a file or data 
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