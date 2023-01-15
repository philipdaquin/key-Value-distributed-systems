use std::{hash::Hash, fmt::Debug, collections::HashMap};


trait Cache<K, V> { 
    fn set(&mut self, key: K, value: V); 
    fn get(&self, key: K) -> Option<V>;
    fn remove_key(&mut self, key: K) -> bool;
    fn version_(&self);
}

struct KvStore<K, V> 
    where 
        K: Debug + Hash + PartialEq + Eq,
        V: Debug + Clone + Copy { 

    cache: HashMap<K, V>
}

impl<K, V> Cache<K, V> for KvStore<K, V> 
    where 
        K: Debug + Hash + PartialEq + Eq,
        V: Debug + Clone + Copy {
    
    /// Inserts a key-value pair into the map.
    /// If the map did not have this key present, None is returned.
    fn set(&mut self, key: K, value: V) {

        self.cache.insert(key, value);

    }

    fn get(&self, key: K) -> Option<V> {
        if let Some(&v) = self.cache.get(&key) {
            return Some(v)
        }
        None
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

fn main() { 
    println!("Hello World")
}