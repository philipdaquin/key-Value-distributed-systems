use std::{net::{ToSocketAddrs, TcpStream}, io::{BufReader, BufWriter, Write}};
use serde_json::{Deserializer, de::IoRead};
use serde::Deserialize;

use crate::{engines::kvstore::{command::Command, kvs::Cache}, response::ServerResponse, error::CacheError};

use crate::error::Result;

trait Client { 
    fn connect<T: ToSocketAddrs>(addr: T) -> Result<Self> where Self: Sized;
    fn get(&mut self, key: String) -> Result<Option<String>>;
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn remove(&mut self, key: String) -> Result<()>;
    fn update_disk(&mut self, command: Command) -> Result<()>;
}

pub struct KvsClient { 
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    writer: BufWriter<TcpStream>
}

impl Client for KvsClient {
    #[tracing::instrument(skip(addr),  level = "debug")]
    fn connect<T: ToSocketAddrs>(addr: T) -> Result<Self> {
        let tcp = TcpStream::connect(addr)?;
        let tcp_clone = tcp.try_clone()?;

        let reader = Deserializer::from_reader(BufReader::new(tcp));
        let writer = BufWriter::new(tcp_clone);

        Ok(Self { 
            reader,
            writer
        })
    }
    
    #[tracing::instrument(skip(self),  level = "debug")]
    fn get(&mut self, key: String) -> Result<Option<String>> {
        self.update_disk(Command::Get(key))?;

        let deserializer = ServerResponse::deserialize(&mut self.reader)?;

        match deserializer { 
            ServerResponse::Ok(v) => Ok(v),
            ServerResponse::Err(e) => Err(CacheError::ServerError(e))
        }
    }

    #[tracing::instrument(skip(self),  level = "debug")]
    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.update_disk(Command::Set(key, value))?;

        let deserializer = ServerResponse::deserialize(&mut self.reader)?;

        match deserializer { 
            ServerResponse::Ok(v) => Ok(v),
            ServerResponse::Err(e) => Err(CacheError::ServerError(e))
        }
    }

    #[tracing::instrument(skip(self),  level = "debug")]
    fn remove(&mut self, key: String) -> Result<()> {

        self.update_disk(Command::Remove(key))?;

        let deserializer = ServerResponse::deserialize(&mut self.reader)?;

        match deserializer { 
            ServerResponse::Ok(v) => Ok(v),
            ServerResponse::Err(e) => Err(CacheError::ServerError(e))
        }
    }

    #[tracing::instrument(skip(self),  level = "debug")]
    fn update_disk(&mut self, command: Command) -> Result<()> { 
        // Serialise the response and write it to the writer 
        serde_json::to_writer(&mut self.writer, &command)?;
        
        // Flush the writer to ensure data is written to the TcpStream
        self.writer.flush()?;

        Ok(())
    }
}