

use crossbeam::channel::{Sender, Receiver, self};

use super::ThreadPool;

type MultiThreaded = Box<dyn FnOnce() + Send + 'static>;

struct SharedQueueThreadPool { 
    pool: Sender<MultiThreaded>
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(num: u32) -> crate::error::Result<Self> where Self: Sized {
        let (sender, receiver) = channel::unbounded::<MultiThreaded>();
        
        for _ in 0..num { 
            let recv= TaskPool(receiver.clone());
            
            std::thread::Builder::new().spawn(move || run_task(recv))?;
        }


        todo!()



    }

    fn spawn<F>(&self, job: F) -> crate::error::Result<()>
        where F: FnOnce() + Send + 'static  {

        self.pool.send(Box::new(job)).unwrap();

        Ok(())
    }
}

#[derive(Clone)]
struct TaskPool(Receiver<MultiThreaded>);

fn run_task(receiver: TaskPool) {
    while let Ok(task) = receiver.0.recv() { 
        log::info!("Running Taskk...");
        task()
    }
}

impl Drop for TaskPool { 
    fn drop(&mut self) {
        if std::thread::panicking() {
            let f = self.clone();
            if let Err(e) = std::thread::Builder::new().spawn(move || run_task(f)) { 
                log::error!("{e}")
            }
        } 
    }
}