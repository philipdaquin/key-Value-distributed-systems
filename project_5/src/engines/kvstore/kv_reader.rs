use std::{io::{Read, BufReader, Seek, SeekFrom}, fs::File, collections::{HashMap, BTreeMap}, sync::{Arc, atomic::AtomicU64}, path::PathBuf, cell::RefCell, ops::Deref, borrow::BorrowMut};
use parking_lot::RwLock;

use crate::{error::{Result, CacheError}, engines::log_path};

use super::{log_reader::LogReaderWithPos, command::{CmdMetadata, Command}};

pub trait Reader { 
    /// Read the log file at the given 'CommandPOs'
    fn read_and<A, R>(&self, cmd: CmdMetadata, a: A) -> Result<R> 
        where A: FnOnce(std::io::Take<&mut LogReaderWithPos<File>>) -> Result<R>;

    /// 
    /// ** As taken from the Project 4
    /// 
    /// `Safe Point` is updated to the latest compaction gen after a compaction finishes
    /// The compaction generation contains the sum of all operations before it and the in 
    /// memory index contains no entries with generation number less than safe_point 
    /// 
    /// This lets us close those files and their handles
    fn close_stale(&self);

    /// Read the log file at the given `CmdMetadata` and deserialize it to `Command`
    fn read_command(&self, cmd: CmdMetadata) -> Result<Command>;
}

/// A single thread reader 
/// 
/// User must read concurrently through multiple KvStore's 
/// in different threads 
pub struct KvReader { 

    /// `PathBuf`
    ///  Represents a file or directory on the file system and provides method for with paths
    pub directory: Arc<PathBuf>,

    // pub index : Arc<BTreeMap<String, CmdMetadata>>,
    /// 
    /// Reader
    /// - Is the read handle to the current log file. 
    /// - It needs to change to a new log file after compact 
    /// 
    pub reader: RwLock<HashMap<u64, LogReaderWithPos<File>>>,

    /// The latest compaction gen after a compaction finishes
    pub safe_point: Arc<AtomicU64>
}

impl Clone for KvReader { 
    fn clone(&self) -> Self {
        Self { 
            directory: self.directory.clone(),
            // index: self.index.to_owned(),
            reader: RwLock::new(HashMap::new()),
            safe_point: Arc::clone(&self.safe_point)
        }
    }
}


impl Reader for KvReader {
    
    fn read_command(&self, cmd: CmdMetadata) -> Result<Command> {
        self.read_and(cmd, |reader| {
            Ok(serde_json::from_reader(reader)?)
        })
    }    

    fn read_and<A, R>(&self, cmd: CmdMetadata, a: A) -> Result<R> 
            where A: FnOnce(std::io::Take<&mut LogReaderWithPos<File>>) -> Result<R> {
        
        let reader = &self.reader;
        
        self.close_stale();

        if reader.read().contains_key(&cmd.generation_num) { 
            // Open the file if we haven't opened it in this `LogStoreReader`
            let path = log_path(&self.directory, cmd.generation_num);
            let log_reader = LogReaderWithPos::new(File::open(path)?)?;

            reader.write().insert(cmd.generation_num, log_reader);

        }

        let mut binding = reader.write();
        let log_reader = binding.get_mut(&cmd.generation_num).unwrap();
        
        log_reader.seek(SeekFrom::Start(cmd.position))?;

        let cmd_reader = log_reader.take(cmd.len);

        // Take(cmd_reader)
        a(cmd_reader)

    }

    /// 
    /// Close file handles with generation number less than safe point 
    fn close_stale(&self) {
        let reader = &self.reader;
        
        while reader.read().len() > 0 { 
            let keys = *reader.read().keys().next().unwrap();

            if self.safe_point.load(std::sync::atomic::Ordering::SeqCst) <= keys { break }

            reader.write().remove(&keys);

        }
    }
}