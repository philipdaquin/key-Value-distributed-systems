use super::{ThreadPool, ThreadPoolMessage};
use crate::error::Result;

pub struct NaiveThread;

impl ThreadPool for NaiveThread {
    fn new(num: u32) -> crate::error::Result<Self> where Self: Sized {
        Ok(NaiveThread)
    }

    fn spawn<F>(&self, job: F) -> Result<()> where F: FnOnce() + Send + 'static  {
        std::thread::spawn(job);
        Ok(())
    }
}