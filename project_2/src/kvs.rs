use std::{path::PathBuf, collections::HashMap, process::exit, io::{Write, Seek}, fs::File};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use crate::writer::LogWriterWithPos;
use crate::command::Command;

trait Cache  {
    fn get(&self, key: String) -> Result<Option<String>>;
    fn set(&mut self, key: String, value: String);
    fn open(&self, path: impl Into<PathBuf>) -> Result<Self> where Self: Sized;
    fn remove(&mut self, key: String) -> Result<()>; 
    fn version();
}

// #[derive(Serialize, Deserialize, Clone, Default)]
struct KvStore { 
    // Locates position of inode of file
    inode: usize,
    // Locates position of inode in log
    inode_map: HashMap<usize, LogWriterWithPos<File>>,
    
    index: Vec<usize>,

    writes: LogWriterWithPos<File>
}

impl Cache for KvStore {
    fn get(&self, key: String) -> Result<Option<String>> {
        todo!()
    }

    fn set(&mut self, key: String, value: String) {

        let command = Command::Set(key, value);
        
        let command_string = serde_json::to_string(&command).expect("");

        let index = self.writes.write(command_string.as_bytes()).expect("");


        self.inode_map.insert(index, value);
    }

    fn open(&self, path: impl Into<PathBuf>) -> Result<Self> where Self: Sized {
        todo!()
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if self.inode_map.contains_key(&key) { 
            println!("Key not found");
            exit(0)
        }
        self.inode_map.remove(&key);

        Ok(())
    }

    fn version() {
        println!("1.0")
    }
}