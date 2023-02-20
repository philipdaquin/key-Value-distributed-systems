use project_5::client::{KvsClient, Client};
use project_5::engines::KvsEngine;
use project_5::engines::kvstore::kvs::{KvStore, Cache};
use project_5::error::{Result, CacheError};
use structopt::{StructOpt, clap::AppSettings::{DisableHelpSubcommand, VersionlessSubcommands}};
use std::env::current_dir;
use std::net::SocketAddr;
use std::process::exit;


const GLOBAL_SETTINGS: &[&str] = &["&[DisableHelpSubcommand]"];


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
#[structopt(
    name = "kvs-client", 
    global_setting = DisableHelpSubcommand, 
    global_setting = VersionlessSubcommands
)]
struct Opt { 
    #[structopt(subcommand)]
    command: Command
}

#[tokio::main]
async fn main() -> Result<()> { 

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let opt = Opt::from_args();

    if let Err(e) = match_cmds(opt).await {
        eprintln!("{e}");
        exit(1);
    }

    Ok(())

}


async fn match_cmds(opt: Opt) -> Result<()> {
    match opt.command {
        Command::Get { key, addr } => {
            log::info!("get!");

            // Connect to server 
            let mut client = KvsClient::connect(addr).await?;
            log::info!("Connected!");
            if let Some(val) = client.get(key).await? {
                println!("key: {{&key}}, value: {val}")
            } else { 
                println!("Key not found")
            }

        },
        Command::Set { key, value, addr } => {
            log::info!("setting!");
            
            let mut client = KvsClient::connect(addr).await?;
            client.set(key, value).await?;
        },
        Command::Remove { key, addr } => {
            log::info!("remove!");
            
            let mut client = KvsClient::connect(addr).await?;
            client.remove(key).await? 
        },
    }
    Ok(())
}