use std::str::from_utf8;

use sled::{Db, Tree};
use crate::{engines::KvsEngine, error::{Result, CacheError}};

#[derive(Debug, Clone)]
pub struct SledKvsEngine  { 
    store: Db
}

impl SledKvsEngine { 
    pub fn new(store: Db) -> Self { 
        Self { 
            store
        }
    }
}


impl KvsEngine for SledKvsEngine {
    fn set(&self, key: String, value: String) -> Result<()> {

        if let Err(e) = self.store.insert(key.as_bytes(), value.as_bytes()) { 
            return Err(CacheError::SledError(e))
        }
        self.store.flush()?;

        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        if let Ok(Some(val)) = self.store.get(key.as_bytes()) { 
            let res = from_utf8(&val).expect("Utf8 error").to_string();
            return Ok(Some(res))
        }
        Ok(None)
    }

    fn remove(&self, key: String) -> Result<()> {
        if let Ok(Some(_)) = self.store.remove(key) { 
            return Ok(())
        }

        self.store.flush()?;

        Err(CacheError::KeyNotFound)
    }
}
