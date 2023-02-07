use super::{ThreadPool, ThreadPoolMessage};
use crate::error::Result;

pub struct NaiveThreadPool;

impl ThreadPool for NaiveThreadPool {
    fn new(num: u32) -> crate::error::Result<Self> where Self: Sized {
        Ok(NaiveThreadPool)
    }

    fn spawn<F>(&self, job: F) -> Result<()> where F: FnOnce() + Send + 'static  {
        std::thread::spawn(job);
        Ok(())
    }
}