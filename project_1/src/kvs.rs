use std::{hash::Hash, fmt::Debug, collections::HashMap};
use std::marker::Copy;

pub trait Cache<K, V> 
    where 
        K: Debug + Hash + PartialEq + Eq  + Clone,
        V: Debug + Clone  { 

    fn new() -> Self;
    fn set(&mut self, key: K, value: V); 
    fn get(&self, key: K) -> Option<V>;
    fn remove_key(&mut self, key: K) -> bool;
    fn version_(&self);
}

pub struct KvStore<K, V> 
    where 
        K: Debug + Hash + PartialEq + Eq+ Clone,
        V: Debug + Clone { 

    pub cache: HashMap<K, V>
}

impl<K, V> Cache<K, V> for KvStore<K, V> 
    where 
        K: Debug + Hash + PartialEq + Eq + Clone,
        V: Debug + Clone {
    fn new() -> Self {
        Self { 
            cache: HashMap::new()
        }
    }
    /// Inserts a key-value pair into the map.
    /// If the map did not have this key present, None is returned.
    fn set(&mut self, key: K, value: V) {
        self.cache.insert(key, value);
    }

    fn get(&self, key: K) -> Option<V> {

        self.cache.get(&key).cloned()
    }

    fn remove_key(&mut self, key: K) -> bool {
        if self.cache.remove(&key).is_some() {
            return true
        }
        false
    }

    fn version_(&self) {
        println!("Version 1.0")
    }
}