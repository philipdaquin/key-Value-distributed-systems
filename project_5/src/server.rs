use tokio::io::{BufReader, BufWriter, AsyncWrite, AsyncRead};
use std::io::Write;
use std::ops::Deref;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use futures::StreamExt;
use tokio::io::AsyncWriteExt;
use serde_json::{from_reader, Deserializer, Value};
use tokio_serde_json::ReadJson;
use tokio_util::codec::{LengthDelimitedCodec, FramedRead};
use crate::engines::KvsEngine;
use crate::engines::kvstore::command::Command;
use crate::error::{Result, CacheError};
use crate::response::ServerResponse;
use crate::threadpool::ThreadPool;
use tokio_serde::formats::SymmetricalJson;
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
        let listener = TcpListener::bind(addr).await?;


        // For each incoming connections, it will have its own unique engine instance 
        //
        //
        // `Sync` trait may be use when a type needs to be safe for concurrent use from 
        //  threads, but since each spawned thread get its own engine instance, there's no need for 
        //  the type to implement the `Sync` trait
        while let Ok((stream, _)) = listener.accept().await {
            let engine = self.engine.clone();
            Self::serve(engine, Box::new(stream))
                .await
                .map_err(move |e| log::error!("Connection failed {e}"))
                .map(|_| ());
            }
        Ok(())
    }
    ///
    /// Process client TCP requests which comes in the form of TcpStream 
    /// - `params` tcpStream handle incoming TCP connections  
    /// - `throws` CacheError 
    #[tracing::instrument(skip(tcp, engine), level = "debug")]
    async fn serve(engine: E, mut tcp: Box<TcpStream>) -> Result<()> { 

        let (mut reader, mut writer) = tcp.into_split();
        // Create a read and write data to the stream 

        // let mut reader = BufReader::new(&mut tcp);
        // let mut writer = BufWriter::new(&mut tcp);

        // Read the frame of bytes from `AsyncRead`
        let mut frame_limited  = FramedRead::new(reader, LengthDelimitedCodec::new());
        

        // First time actually using meta programming here 
        macro_rules! update_disk {
            ($resp:expr) => {{
                let command = $resp;

                let bytes = serde_json::to_vec(&command).unwrap();
                writer.write_all(&bytes).await.unwrap();
            }};
        }

        // Read JSON messages from the stream 
        while let Some(Ok(bytes)) = frame_limited.next().await { 
            // let mut deserialised:tokio_serde::Framed<FramedRead<tokio::io::BufReader<&mut tokio::net::TcpStream>, LengthDelimitedCodec>, Value, Value, tokio_serde::formats::Json<serde_json::Value, serde_json::Value>> = tokio_serde::SymmetricallyFramed::new(
            //     frame_limited, 
            //     SymmetricalJson::<Value>::default()
            // );
            // Process the tokio worker 
            let engine = engine.clone();
            tokio::spawn(async move {

                let command: Command = serde_json::from_slice(&bytes).unwrap();

                match command {
                    Command::Set(key, _) => update_disk!(match engine.get(key) {
                        Ok(val) => Ok(val), 
                        Err(e) => Err(ServerResponse::<String>::Err(format!("{}", e)))
                    }),
                    Command::Get(key) => update_disk!(match engine.get(key) {
                        Ok(val) => Ok(val),
                        Err(e) => Err(ServerResponse::<String>::Err(format!("{}", e)))
                    }),
                    Command::Remove(key) => update_disk!(match engine.remove(key) {
                        Ok(val) => Ok(val),
                        Err(e) => Err(ServerResponse::<String>::Err(format!("{}", e)))
                    }),
                }
                
            });
        }

        Ok(())
    }
}