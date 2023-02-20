use std::{net::SocketAddr, sync::Arc, time::Duration};
use futures::lock::Mutex;
use serde::de::DeserializeOwned;
// use parking_lot::Mutex;
use tokio::{net::{ToSocketAddrs, TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}}, 
    io::{BufReader, BufWriter, AsyncWriteExt}

};
use crate::{engines::kvstore::{command::Command, kvs::Cache}, response::ServerResponse, error::CacheError};
use crate::error::Result;
use async_trait::async_trait;
use tokio::io::AsyncReadExt;

#[async_trait]
pub trait Client { 
    async fn connect(addr: SocketAddr) -> Result<Self> where Self: Sized;
    async fn get(&mut self, key: String) -> Result<Option<String>>;
    async fn set(&mut self, key: String, value: String) -> Result<()>;
    async fn remove(&mut self, key: String) -> Result<()>;
    async fn process_requests<T: DeserializeOwned>(&mut self, command: Command) -> Result<ServerResponse<T>>;
}

#[derive(Debug)]
pub struct KvsClient { 
    reader: Arc<Mutex<BufReader<OwnedReadHalf>>>,
    writer: Arc<Mutex<BufWriter<OwnedWriteHalf>>>
}

#[async_trait]
impl Client for KvsClient {
    /// 
    /// Create a TCP connection to the server  
    /// 
    /// 
    /// 
    /// Fault tolerant solutions
    /// Errors are handled by backing off and retrying. Exponential backoff strategy is used
    /// - max retries = 5
    /// - max backoff time = 54
    /// 
    #[tracing::instrument(field(addr),  level = "debug")]
    async fn connect(addr: SocketAddr) -> Result<Self> {
        let mut backoff = 1;
        let max_backoff = 64;
        let mut retries = 0;
        let max_retries = 5;
        log::info!("🚀 Client connecting to Server");

        /*
            Limited number of retries. Keep looping until we reach maximum number of retries.
            Else, the loop ceases and we return `CacheError::ServerError`
        */
        // while retries < max_retries { 
        //     match TcpStream::connect(addr).await {
        //         Ok(stream) => { 
        //             let (read, write) = stream.into_split();
                    
        //             // Initialise Readers and writers 
        //             let reader = Mutex::new(BufReader::new(read));
        //             let writer = Mutex::new(BufWriter::new(write));
                
        //             return Ok(Self { 
        //                 reader: Arc::new(reader), 
        //                 writer: Arc::new(writer)
        //             })
                    
        //         },
        //         Err(e) => { 
        //             // Pause execution until the back off period elapses
        //             if backoff > max_backoff { 
        //                 return Err(e.into())
        //             }

        //             tokio::time::sleep(Duration::from_secs(backoff)).await;
        //             backoff *= 2;
        //             retries += 1;
        //         }
        //     }
        // }

        // Err(CacheError::ServerError("Connection failed after max retries ".to_string()))
        let (read, write) = TcpStream::connect(addr).await?.into_split();

        let reader = Mutex::new(BufReader::new(read));
        let writer = Mutex::new(BufWriter::new(write));
       
        return Ok(Self { 
                            reader: Arc::new(reader), 
                            writer: Arc::new(writer)
                        })
    }
    
    
    
    #[tracing::instrument(skip(self),  level = "debug")]
    async fn get(&mut self, key: String) -> Result<Option<String>> {
        self.process_requests(Command::Get(key))
            .await?
            .ok()
    }

    #[tracing::instrument(skip(self),  level = "debug")]
    async fn set(&mut self, key: String, value: String) -> Result<()> {
        let _: Result<Option<String>> = self.process_requests(Command::Set(key, value))
            .await?.ok();
        Ok(())
    }

    #[tracing::instrument(skip(self),  level = "debug")]
    async fn remove(&mut self, key: String) -> Result<()> {
        let _: Result<Option<String>> = Ok(self.process_requests(Command::Remove(key)).await?.ok()?);

        Ok(())
    }

    ///
    /// Process the user request and return server reponse 
    #[tracing::instrument(fields(repository = "command"), level = "debug")]
    async fn process_requests<T: DeserializeOwned>(&mut self, command: Command) -> Result<ServerResponse<T>> { 
        // Serialize the request into a byte buffer 
        let request_data = serde_json::to_vec(&command)?;

        // Send the request to the server
        self.writer.lock().await.write_all(&request_data).await?;

        // Read the response 
        let mut buffer = Vec::new();
        
        //Pulls some bytes from this source into the specified buffer, returning how many bytes were read.
        let i = self.reader.lock().await.read(&mut buffer).await?;

        // Deserialize the server and return to the client
        let response: ServerResponse<_> = serde_json::from_slice(&buffer[..i])?;

        match response {
            ServerResponse::Ok(res) => Ok(ServerResponse::Ok(res)),
            ServerResponse::Err(e) => Err(CacheError::ServerError(e)),
        }
    }
}