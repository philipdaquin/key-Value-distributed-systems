use project_4::client::{KvsClient, Client};
use project_4::engines::KvsEngine;
use project_4::engines::kvstore::kvs::{KvStore, Cache};
use project_4::error::{Result, CacheError};
use structopt::StructOpt;

use std::env::current_dir;
use std::net::SocketAddr;
use std::process::exit;

#[derive(StructOpt, Debug)]
enum Command { 
    #[structopt(name = "get")]
    Get {
        #[structopt(name = "KEY")]
        key: String,

        #[structopt(
            long, 
            value_name = "IP:PORT", 
            default_value = "127.0.0.1:4000", 
            parse(try_from_str)
        )]
        addr: SocketAddr
    },

    #[structopt(name = "set")]
    Set {
        #[structopt(name = "KEY")]
        key: String, 

        #[structopt(name = "VALUE")]
        value: String,

        #[structopt(
            long, 
            value_name = "IP:PORT", 
            default_value = "127.0.0.1:4000", 
            parse(try_from_str)
        )]
        addr: SocketAddr
    },

    #[structopt(name = "rm")]
    Remove {
        #[structopt(name = "KEY")]
        key: String,

        #[structopt(
            long, 
            value_name = "IP:PORT", 
            default_value = "127.0.0.1:4000", 
            parse(try_from_str)
        )]
        addr: SocketAddr
    }
}


#[derive(StructOpt, Debug)]
#[structopt(name = "kvs-client")]
struct Opt { 
    #[structopt(subcommand)]
    command: Command
}


fn main() -> Result<()> { 
    let opt = Opt::from_args();

    if let Err(e) = match_cmds(opt) {
        eprintln!("{e}");
        exit(1)
    }

    Ok(())

}


fn match_cmds(opt: Opt) -> Result<()> {
    match opt.command {
        Command::Get { key, addr } => {
            // Connect to server 
            let mut client = KvsClient::connect(addr)?;

            if let Some(val) = client.get(key)? {
                println!("key: {{&key}}, value: {val}")
            } else { 
                println!("Key not found")
            }

        },
        Command::Set { key, value, addr } => {
            let mut client = KvsClient::connect(addr)?;
            client.set(key, value)?;
        },
        Command::Remove { key, addr } => {
            let mut client = KvsClient::connect(addr)?;
            client.remove(key)? 
        },
    }
    Ok(())
}