use std::sync::Arc;

use rayon::{prelude::*, ThreadPoolBuilder, ThreadPool as RayonThread};

use crate::error::CacheError;

use super::ThreadPool;


#[derive(Clone)]
pub struct RayonThreadPool {
    /// `Clone` is not implemented for RayonThread
    /// We wrap it inside of Atomic Reference Counter instead 
    pool: Arc<RayonThread>
}

impl ThreadPool for RayonThreadPool {
    fn new(num: u32) -> crate::error::Result<Self> where Self: Sized {

        let thread_pool =  ThreadPoolBuilder::new()
            .num_threads(num as usize)
            .build()
            .map_err(|e| CacheError::ServerError(format!("{e}")))?;

        let pool = Arc::new(thread_pool);

        Ok(Self { pool })
    }

    fn spawn<F>(&self, job: F) -> crate::error::Result<()>
        where F: FnOnce() + Send + 'static  {
        self.pool.spawn(job);
        Ok(())
    }
}