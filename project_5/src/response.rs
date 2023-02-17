use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerResponse<T> { 
    Ok(T), 
    Err(String)
}