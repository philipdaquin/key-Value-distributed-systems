use rayon::{prelude::*, ThreadPoolBuilder, ThreadPool as RayonThread};

use super::ThreadPool;



pub struct RayonThreadPool {
    pool: RayonThread
}

impl ThreadPool for RayonThreadPool {
    fn new(num: u32) -> crate::error::Result<Self> where Self: Sized {
        Ok(Self {
            pool: ThreadPoolBuilder::new()
                .num_threads(num as usize)
                .build()
                .unwrap()
        })
    }

    fn spawn<F>(&self, job: F) -> crate::error::Result<()>
        where F: FnOnce() + Send + 'static  {
        self.pool.spawn(job);
        Ok(())
    }
}