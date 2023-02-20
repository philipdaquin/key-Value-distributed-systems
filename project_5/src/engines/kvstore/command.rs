use std::ops::Range;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Command  { 
    // Set {key, value }
    Set(String, String),

    // Get { key: String }
    Get(String),

    // Remove Key: String 
    Remove(String)
}
/// Metadata of Command 
#[derive(Clone, Debug)]
pub struct CmdMetadata { 
    pub generation_num: u64,
    pub position: u64, 
    pub len: u64 
}

impl From<(u64, Range<u64>)> for CmdMetadata {
    fn from((generation_num, Range { start, end }): (u64, Range<u64>)) -> Self {
        Self { 
            generation_num, 
            position: start,
            len: end - start
        }
    }
}