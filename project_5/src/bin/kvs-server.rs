use core::num;
use std::env::current_dir;
use std::process::exit;
use std::{net::SocketAddr, str::FromStr};
use project_5::engines::KvsEngine;
use project_5::engines::kvstore::kvs::KvStore;
use project_5::engines::sledkvstore::sledvs::SledKvsEngine;
use project_5::server::KvsServer;
use project_5::threadpool::rayon::RayonThreadPool;
use project_5::error::{Result, CacheError};
use structopt::{StructOpt};
use project_5::threadpool::ThreadPool;
use strum::{EnumString, EnumVariantNames, VariantNames, Display};
use log::LevelFilter;
use std::env;
use env_logger;
const V: &[&str] = &["Engines::VARIANTS"];

// #[derive(EnumString, EnumVariantNames, Debug, PartialEq, Eq, Clone, Copy, Display)]
// #[allow(non_camel_case_types)]
// enum Engines { 
//     kvs,
//     sled
// }
#[derive(EnumString, EnumVariantNames, Debug, PartialEq, Eq, Clone, Copy, Display)]
#[allow(non_camel_case_types)]
// #[strum(serialize_all = "kebab_case")]
enum Engines { 
    kvs,
    sled
}


impl Engines { 
    fn variants() -> [&'static str; 2] { 
        ["kvs", "sled"]
    }
}


#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "kvs-server")]
struct Opt { 
    #[structopt(
        long, 
        value_name = "IP:PORT", 
        default_value = "127.0.0.1:4000", 
        parse(try_from_str)
    )]
    addr: SocketAddr,

    #[structopt(
        long, 
        value_name = "ENGINE-NAME",
        possible_values = V,
    )]
    engine: Option<Engines>
}

fn check_current_engine() -> Result<Option<Engines>> { 
    let engine = current_dir()?.join("engine");

    if !engine.exists() { 
        return Ok(None)
    } 

    match std::fs::read_to_string(engine)?.parse() { 
        Ok(engine) => Ok(Some(engine)),
        Err(e) => { 
            
            log::warn!("The content of file is invalid {e}");
            
            Ok(None)
        } 
    }
}



fn main() -> Result<()> { 

    env_logger::builder()
    .filter_level(LevelFilter::Info)
    .init();

    let mut opt = Opt::from_args();
    let engine = check_current_engine()?
        .ok_or(|e| CacheError::ServerError(e));

    if let Ok(v) = engine { 
        

        if let Some(curr) = opt.engine { 
            if v != curr {  
                log::error!("Wrong Engine");
                exit(1)
            }
        } else { 
            opt.engine = Some(v)
        }
    } 

    if let Err(e) = run_server(opt) {
        log::error!("{}", e);
        exit(1)
    }

    Ok(())

}

fn run_server(opt: Opt) -> Result<()> {
    let current_engine = opt.engine.unwrap_or(Engines::kvs);
    
    log::info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    
    log::info!("Storage engine: {}", current_engine);
    
    log::info!("Listening on {}", opt.addr);

    std::fs::write(current_dir()?.join("engine"), format!("{current_engine}"))?;

    let pool = RayonThreadPool::new(1)?;

    match current_engine {
        Engines::kvs => {

            let store = KvStore::open(current_dir()?)?;
            let server = KvsServer::new(store, pool);
            server.run(opt.addr)?;
        },
        Engines::sled => {
            server_spawn(
                SledKvsEngine::new(sled::open(current_dir()?)?),
                pool,
                opt.addr
            )?;
        },
    }
    Ok(())
}


fn server_spawn<E: KvsEngine, P: ThreadPool>(engine: E, pool: P, addr: SocketAddr) -> Result<()> {
    let server = KvsServer::new(engine, pool);
    server.run(addr)
}


