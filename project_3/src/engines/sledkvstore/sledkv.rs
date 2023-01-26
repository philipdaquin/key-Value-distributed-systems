use std::str::from_utf8;

use sled::Db;
use crate::{engines::KvsEngine, error::{Result, CacheError}};
struct SledKvsEngine  { 
    store: Db
}

impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {

        if let Err(e) = self.store.insert(key.as_bytes(), value.as_bytes()) { 
            return Err(CacheError::SledError(e))
        }

        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Ok(Some(val)) = self.store.get(key.as_bytes()) { 
            let res = from_utf8(&val).expect("Utf8 error").to_string();
            return Ok(Some(res))
        }
        Ok(None)
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if let Ok(Some(_)) = self.store.remove(key) { 
            return Ok(())
        }
        Err(CacheError::KeyNotFound)
    }
}


