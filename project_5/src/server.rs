use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use serde_json::{from_reader, Deserializer};


use crate::engines::KvsEngine;
use crate::engines::kvstore::command::Command;
use crate::error::{Result, CacheError};
use crate::response::ServerResponse;
use crate::threadpool::ThreadPool;
use crate::threadpool::thread::NaiveThreadPool;

pub struct KvsServer<E: KvsEngine, T: ThreadPool> { 
    engine: E,
    thread_pool: T
}

impl<E, T> KvsServer<E, T> where E:  KvsEngine, T: ThreadPool { 
    pub fn new(engine: E, thread_pool: T) -> Self { 
        Self { 
            engine,
            thread_pool
        }
    } 

    /// 
    /// Aim: Start the server enginee
    /// 
    /// Input: 
    /// - `A`: ToSocketAddrs
    /// 
    /// Panics 
    /// - if unable to accept any connections, return `CacheError::ServerError`
    /// 
    /// Returns 
    /// - Return<()>
    /// 
    /// 
    #[tracing::instrument(skip(addr, self), level = "debug")]
    pub fn run<A: ToSocketAddrs>(&self, addr: A) -> Result<()> {  
        let listener = TcpListener::bind(addr).expect("Invalid TCP address");
        
        for stream in listener.incoming() { 
            // For each incoming connections, it will have its own unique engine instance 
            //
            //
            // `Sync` trait may be use when a type needs to be safe for concurrent use from 
            //  threads, but since each spawned thread get its own engine instance, there's no need for 
            //  the type to implement the `Sync` trait
            let engine = self.engine.clone();


            self.thread_pool.spawn(move || match stream { 
                Ok(tcp ) => if let Err(e) = Self::serve(engine, tcp) {
                    log::error!("Something went wrong: {e}");
                },
                Err(e) => {
                    log::error!("Connection failed {e}");
                    // Err::<((), CacheError)>(CacheError::UnexpectedError);
                }
            })?;
        }
        Ok(())
    }
    ///
    /// Process client TCP requests which comes in the form of TcpStream 
    /// - `params` tcpStream handle incoming TCP connections  
    /// - `throws` CacheError 
    #[tracing::instrument(skip(tcp, engine), level = "debug")]
    fn serve(engine: E, tcp: TcpStream) -> Result<()> { 

        // Create a read and write data to the stream 
        let reader = BufReader::new(&tcp);
        let mut writer = BufWriter::new(&tcp);

        // Deserialize the incoming data as a Command Request
        let request = Deserializer::from_reader(reader).into_iter::<Command>();
        
        /// First time actually using meta programming here 
        macro_rules! update_disk {
            ($resp:expr) => {{
                let command = $resp;
                serde_json::to_writer(&mut writer, &command)?;
                writer.flush()?;
            }};
        }

        // Process incoming requests and return the proper response 
        for req in request { 
            match req? {
                Command::Set(key, val) => {
                    engine.set(key.clone(), val.clone())?;
                    serde_json::to_writer(&mut writer, &Command::Set(key, val))?;
                    writer.flush()?;
                },
                Command::Get(key) => update_disk!(match engine.get(key) {
                    Ok(val) => Ok(val),
                    Err(e) => Err(ServerResponse::<String>::Err(format!("{}", e)))
                }),
                Command::Remove(key) => update_disk!(match engine.remove(key) {
                    Ok(val) => Ok(val),
                    Err(e) => Err(ServerResponse::<String>::Err(format!("{}", e)))
                }),
            }
        }
        Ok(())
    }
}