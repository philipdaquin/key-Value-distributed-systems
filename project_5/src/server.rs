use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::io::AsyncWriteExt;
use crate::engines::KvsEngine;
use crate::engines::kvstore::command::Command;
use crate::error::{Result, CacheError};
use tokio::io::AsyncReadExt;

pub struct KvsServer<E: KvsEngine> { 
    engine: E,
}

impl<E> KvsServer<E> where E:  KvsEngine { 
    pub fn new(engine: E) -> Self { 
        Self { 
            engine,
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
    pub async fn run<A: ToSocketAddrs>(&self, addr: A) -> Result<()> {  
        // Bind the listener to the address 
        let listener = TcpListener::bind(addr).await?;

        // Accept new incoming connection 
        while let Ok((socket, _)) = listener.accept().await {
            self.serve_client(socket)
                .await
                .map_err(move |e| log::error!("Connection failed {e}"))
                .map(|_| ())
                .expect("Server Error");
            }
        Ok(())
    }
    ///
    /// Process client TCP requests which comes in the form of TcpStream 
    /// - `params` tcpStream handle incoming TCP connections  
    /// - `throws` CacheError 
    #[tracing::instrument(skip(tcp, self), level = "debug")]
    async fn serve_client(&self, mut tcp: TcpStream) -> Result<()> { 
        // Seperate Read and Write Handle for the connection 
        // `Split I/O` resources 
        let (mut reader, mut writer) = tcp.split();
        
        // Create a read and write data to the stream 
        let mut buffer = Vec::new();
        let _ = reader.read(&mut buffer).await.unwrap();

        // Process the request 
        let response = self.process_request(&buffer[..]).await?;
        
        let debug_: Command = serde_json::from_slice(&response).unwrap();
        log::info!("{:#?}", debug_);
        
        // Send the response back to the client 
        writer.write_all(&response).await?;
        writer.flush().await?;
        
        Ok(())
    }

    async fn process_request(&self, request: &[u8]) -> Result<Box<Vec<u8>>> {
        // Deserialize request 
        let command = serde_json::from_slice(request);
        
        // Call internal server response and serialize the response 
        let response = command
            .map_err(CacheError::from)
            .and_then(
                move |res| -> Result<Command> {
                    match res {
                        Command::Set(key, value) => {
                            self.engine.set(key.clone(), value.clone())?;
                            Ok(Command::Set(key, value))
                        },
                        Command::Get(key) => {
                            self.engine.get(key.clone())?;
                            Ok(Command::Get(key))
                        },
                        Command::Remove(key) => {
                            self.engine.remove(key.clone())?;
                            Ok(Command::Remove(key))
                        }
                    }
                }
            )
            .and_then(|f| {
                // Serialize Command 
                let buff = serde_json::to_vec(&f)?;
                Ok(Box::new(buff))
            });

            response
        // Serialize the response into Vec<u8>
        
    }
}