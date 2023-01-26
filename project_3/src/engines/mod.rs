pub mod kvstore;
pub mod sledkvstore;
use crate::error::Result;

pub trait KvsEngine { 
    
    /// 
    /// Set the value of a string ley to a string
    /// - Return an Error if the value is not written successfully
    fn set(&mut self, key: String, value: String) -> Result<()>;

    /// Get the string value of a string key. If the key does not exist, return None
    /// - Return an error if the value is not read successfully
    fn get(&mut self, key: String) -> Result<Option<String>>;

    /// 
    /// Remove a given string key 
    /// - Return an error if the key does not exit or value is not read successfully
    fn remove(&mut self, key: String) -> Result<()>;

}