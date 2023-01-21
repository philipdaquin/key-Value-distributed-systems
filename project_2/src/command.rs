use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum Command  { 
    // Set {key, value }
    Set(String, String),

    // Get { key: String }
    Get(String)
}



