use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerResponse { 
    Ok(()), 
    Err(String)
}