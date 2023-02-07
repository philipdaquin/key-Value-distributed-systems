pub mod thread;

use crate::error::Result;

enum ThreadPoolMessage { 
    RunJob(Box<dyn FnOnce() + Send + 'static>),
    Shutdown
}

pub trait ThreadPool { 

    /// 
    /// Create a new thread pool, immediately spawning the specified numbe of threads
    /// 
    /// Return any any threads fail to spawn 
    fn new(num: u32) -> Result<Self> where Self: Sized;


    ///
    /// Spawning a function into the thread pool 
    /// 
    fn spawn<F>(&self, job: F) -> Result<()>
        where F: FnOnce() + Send + 'static ;
}