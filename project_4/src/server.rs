use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use serde_json::{from_reader, Deserializer};

use crate::engines::KvsEngine;
use crate::engines::kvstore::command::Command;
use crate::error::{Result, CacheError};
use crate::response::ServerResponse;

pub struct KvsServer<E: KvsEngine> { 
    engine: E
}

impl<E> KvsServer<E> where E: KvsEngine { 
    fn new(engine: E) -> Self { 
        Self { 
            engine
        }
    } 

    /// 
    /// Aim: Start the server enginee
    /// 
    /// ### Input: 
    /// - `A`: ToSocketAddrs
    /// 
    /// ### Panics 
    /// - if unable to accept any connections, return `CacheError::ServerError`
    /// 
    /// ### Returns 
    /// - Return<()>
    /// 
    /// 
    #[tracing::instrument(skip(addr), level = "debug")]
    fn run<A: ToSocketAddrs>(addr: A) -> Result<()> {  
        let listener = TcpListener::bind(addr).expect("Invalid TCP address");
        
        for stream in listener.incoming() { 
            if let Ok(tcp) = stream {
                log::info!("Connected server at {}",  tcp.peer_addr()?);
            } else { 

                log::error!("Connection failed");
                return Err(CacheError::UnexpectedError)
            }
        }
        Ok(())
    }
    ///
    /// Process client TCP requests which comes in the form of TcpStream 
    /// - `params` tcpStream handle incoming TCP connections  
    /// - `throws` CacheError 
    #[tracing::instrument(skip(tcp, self), level = "debug")]
    fn serve(&mut self, tcp: TcpStream) -> Result<()> { 

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
                    self.engine.set(key.clone(), val.clone())?;
                    serde_json::to_writer(&mut writer, &Command::Set(key, val))?;
                    writer.flush()?;
                },
                Command::Get(key) => update_disk!(match self.engine.get(key) {
                    Ok(val) => Ok(val),
                    Err(e) => Err(ServerResponse::<String>::Err(format!("{}", e)))
                }),
                Command::Remove(key) => update_disk!(match self.engine.remove(key) {
                    Ok(val) => Ok(val),
                    Err(e) => Err(ServerResponse::<String>::Err(format!("{}", e)))
                }),
            }
        }
        Ok(())
    }
    fn read_cmd(&self, tcp: &TcpStream) -> Result<Command> {
        // Create a read and write data to the stream 
        let reader = BufReader::new(tcp);
        let mut writer = BufWriter::new(tcp);

        // Deserialize the incoming data as a Command Request
        let request = Deserializer::from_reader(reader).into_iter::<Command>();
        
        // First time actually using meta programming here 
        for req in request { 
            return Ok(req.expect("Reading Command Error"))
        }

        Err(CacheError::KeyNotFound)

    }
    fn process_cmd(&self, cmd: Command) -> Result<()> {
        todo!()
    }
}