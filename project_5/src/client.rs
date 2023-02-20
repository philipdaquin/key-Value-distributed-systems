use std::net::SocketAddr;
use futures::lock::Mutex;
use serde::de::DeserializeOwned;
// use parking_lot::Mutex;
use tokio::{net::{ToSocketAddrs, TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}}, 
    io::{BufReader, BufWriter, AsyncWrite, AsyncRead, ReadBuf, ReadHalf, WriteHalf, AsyncWriteExt}

};
use serde_json::{Deserializer, de::IoRead};
use tokio_serde::SymmetricallyFramed;
use crate::{engines::kvstore::{command::Command, kvs::Cache}, response::ServerResponse, error::CacheError};
use tokio::io::AsyncBufReadExt;
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
    reader: Mutex<BufReader<OwnedReadHalf>>,
    writer: Mutex<BufWriter<OwnedWriteHalf>>
}

#[async_trait]
impl Client for KvsClient {
    /// 
    /// Create a TCP connection to the server  
    /// 
    /// Errors are handled by backing off and retrying.
    /// - Exponential backoff strategy is used 
    /// 
    /// 
    #[tracing::instrument(skip(addr),  level = "debug")]
    async fn connect(addr: SocketAddr) -> Result<Self> {
        // let mut backoff = 1;

        // Implement maintenance techniques for fault tolerant systems 
        let (read, write) = TcpStream::connect(addr).await?.into_split();

        let reader = Mutex::new(BufReader::new(read));
        let writer = Mutex::new(BufWriter::new(write));
       
        Ok(Self { 
            reader, 
            writer
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
        let response: Result<Option<String>> = self.process_requests(Command::Set(key, value))
            .await?.ok();
        Ok(())
    }

    #[tracing::instrument(skip(self),  level = "debug")]
    async fn remove(&mut self, key: String) -> Result<()> {
        let response: Result<Option<String>> = Ok(self.process_requests(Command::Remove(key)).await?.ok()?);

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