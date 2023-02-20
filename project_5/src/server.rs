use std::time::Duration;

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
        Self { engine }
    } 

    /// 
    /// Accept an inbound connection
    /// 
    /// ### Arguments: 
    /// - `A`: ToSocketAddrs
    /// 
    /// ### Result 
    /// 
    /// 
    /// ### Exponential Backoff 
    /// Errors are handled by backing off and retrying. 
    /// Exponential backoff is implemented, increasing the waiting time between retries up to a maximum backoff time.
    /// After the first failure, the task waits for 1 + `random_number_seconds` seconds and retry the request...
    /// and so, up to a `max_backoff` time.
    /// 
    /// Continue waiting and rettying up to some maximum number of retries, but do not increase the wait
    /// period between retries 
    /// 
    /// where, 
    /// 
    /// - `max_backoff` is between 32 or 64 seconds
    /// - The wait time is `min(((2^n)+random_number_milliseconds), maximum_backoff)`, 
    ///     with `n` incremented by 1 for each iteration (request)
    /// - `random_number_seconds ` is a random number of seconds <= to 1000
    /// 
    #[tracing::instrument(skip(addr, self), level = "debug")]
    async fn run<A: ToSocketAddrs>(&self, addr: A) -> Result<()> {  
        let mut backoff = 1;
        let max_backoff = 64;
        let mut retries = 0;
        let max_retries = 5;

        /*
            Limited number of retries. Keep looping until we reach maximum number of retries.
            Else, the loop ceases and we return `CacheError::ServerError`
        */
        while retries < max_retries {
            
            // Bind the listener to the address 
            let listener = TcpListener::bind(&addr).await;

            match listener?.accept().await { 
                Ok((socket, _)) => {
                    // Once client is connected, serve their requests
                    self.serve_client(socket)
                        .await
                        .map_err(move |e| log::error!("Connection failed {e}"))
                        .map(|_| ())
                        .expect("Server Error");
                },
                
                Err(e) => {

                    if backoff > max_backoff {
                        return Err(e.into())
                    }
                    // Pause execution until the back off period elapses
                    tokio::time::sleep(Duration::from_secs(backoff)).await;
                    backoff *= 2;
                    retries += 1;
                }
            }
        }

        Err(CacheError::ServerError("Connection failed after max retries ".to_string()))
    }

    ///
    /// ### INTERNAL FUNCTION: 
    /// Listen for incoming connections 
    ///
    ///
    /// ### Arguments
    ///
    /// * `tcp` - A TCP stream between a local and a remote socket.
    ///
    /// ### Returns
    ///
    /// A `Result<()>, CacheError>` containing the boxed vector of bytes
    /// representing the response, or an `std::io::Error` if there is an issue reading or
    /// writing to a socket, or a `serde_json::Error` if there is an issue deserializing
    /// or serializing JSON.
    ///
    /// ### Examples
    ///
    /// ```
    /// # use my_engine::{Engine, Command};
    /// # use std::error::Error;
    /// # async fn example() -> Result<(), Box<dyn Error>> {
    /// # let engine = Engine::new(kvs);
    /// 
    /// let server = KvsServer::new(engine)
    ///                        .serve_client(tcpstream)
    ///                        .await?;
    /// 
    /// # Ok(())
    /// 
    /// # }
    /// ```
    #[tracing::instrument(skip(tcp, self), level = "debug")]
    async fn serve_client(&self, mut tcp: TcpStream) -> Result<()> { 
        // Seperate Read and Write Handle for the connection 
        // `Split I/O` resources 
        let (mut reader, mut writer) = tcp.split();
        
        // Create a read and write data to the stream 
        let mut buffer = Vec::new();
        let n = reader.read(&mut buffer).await.unwrap();

        // Process the request 
        let response = self.process_request(&buffer[..n]).await?;
        
        let debug_: Command = serde_json::from_slice(&response).unwrap();
        log::info!("{:#?}", debug_);
        
        // Send the response back to the client 
        writer.write_all(&response).await?;
        writer.flush().await?;
        
        Ok(())
    }

    /// Processes a request byte slice and returns a boxed vector of bytes as output.
    ///
    /// # Arguments
    ///
    /// * `request` - A byte slice containing the request to process.
    ///
    /// # Returns
    ///
    /// A `Result<Box<Vec<u8>>, std::io::Error>` containing the boxed vector of bytes
    /// representing the response, or an `std::io::Error` if there is an issue reading or
    /// writing to a socket, or a `serde_json::Error` if there is an issue deserializing
    /// or serializing JSON.
    ///
    /// # Examples
    ///
    /// ```
    /// # use kvs::{Engine, Command};
    /// # use std::error::Error;
    /// # async fn example() -> Result<(), Box<dyn Error>> {
    /// # let engine = Engine::new();
    /// let request = b"{\"command\":\"get\",\"key\":\"my_key\"}";
    /// let response = process_request(&engine, request).await?;
    /// # Ok(())
    /// 
    /// # }
    /// ```
    #[tracing::instrument(skip(request, self), level = "debug")]
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
                // Serialize Command into Box<Vec<u8>>
                let buff = serde_json::to_vec(&f)?;
                Ok(Box::new(buff))
            });

            response
        
    }
}