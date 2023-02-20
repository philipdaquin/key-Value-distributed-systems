use serde::{Serialize, Deserialize};

use crate::error::{Result, CacheError};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ServerResponse<T> { 
    Ok(Option<T>), 
    Err(String),
}

impl<T: Clone> ServerResponse<T> {
    pub fn ok(&self) -> Result<Option<T>> { 
        
        match self {
            ServerResponse::Ok(ref res) => {
                match res.clone() { 
                    Some(m) => Ok(Some(m)),
                    Some(_) => Err(CacheError::ServerError("Invalid Response".to_string())),
                    None => Err(CacheError::ServerError("No response received".to_string()))
                }
            },
            ServerResponse::Err(e) => Err(CacheError::ServerError(e.to_string())),
         
        }
    }
}

