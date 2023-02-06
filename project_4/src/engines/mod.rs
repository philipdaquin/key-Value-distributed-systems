pub mod kvstore;
pub mod sledkvstore;
use std::{fs::{File, self}, ffi::OsStr, collections::BTreeMap, path::{Path, PathBuf}, io::Seek};

use crate::error::Result;

use self::kvstore::{log_reader::LogReaderWithPos, command::{CmdMetadata, Command}};

pub trait KvsEngine: Clone + Send + 'static { 
    
    /// 
    /// Set the value of a string ley to a string
    /// - Return an Error if the value is not written successfully
    fn set(&self, key: String, value: String) -> Result<()>;

    /// Get the string value of a string key. If the key does not exist, return None
    /// - Return an error if the value is not read successfully
    fn get(&self, key: String) -> Result<Option<String>>;

    /// 
    /// Remove a given string key 
    /// - Return an error if the key does not exit or value is not read successfully
    fn remove(&self, key: String) -> Result<()>;

}

pub fn log_path(direction: &Path, gen: u64) -> PathBuf { 
    direction.join(format!("{}.log", gen))
}
/// 
/// Load the whole log file and store value locations in the index map 
/// 
/// Return how many bytes can be saved after a compaction 
pub fn load_log_file(
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

///
/// A generation number is a ID that is used to 
/// distinguish different versions of a file or data 
/// 
pub fn sorted_generation(path: &Path) -> Result<Vec<u64>> { 
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