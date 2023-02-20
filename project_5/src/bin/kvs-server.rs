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
use structopt::{StructOpt, clap::arg_enum};
use project_5::threadpool::ThreadPool;
use strum::{EnumString, EnumVariantNames, VariantNames, Display};
use log::LevelFilter;
use std::env;
use env_logger;

const V: &[&str] = &["Engines::VARIANTS"];

#[derive(EnumString, EnumVariantNames, Debug, PartialEq, Eq, Clone, Copy, Display)]
#[allow(non_camel_case_types)]
enum Engines { 
    kvs,
    sled
}
// #[derive(EnumString, EnumVariantNames, Debug, PartialEq, Eq, Clone, Copy, Display)]
// #[allow(non_camel_case_types)]
// #[strum(serialize_all = "kebab_case")]
// enum Engines { 
//     kvs,
//     sled
// }

// impl Engines { 
//     fn variants() -> [&'static str; 2] { 
//         ["kvs", "sled"]
//     }
// }


// arg_enum! {
//     #[allow(non_camel_case_types)]
//     #[derive(Debug, Copy, Clone, PartialEq, Eq)]
//     enum Engines {
//         kvs,
//         sled
//     }
// }



#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "kvs-server")]
struct Opt { 
    #[structopt(
        long, 
        value_name = "IP:PORT", 
        help = "Sets the listening address",
        default_value = "127.0.0.1:4000", 
        parse(try_from_str)
    )]
    addr: SocketAddr,

    #[structopt(
        long,
        value_name = "ENGINE-NAME",
        help = "Sets the storage engine",
        possible_values = &Engines::VARIANTS,
        case_insensitive = true
    )]
    engine: Option<Engines>,
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


#[tokio::main]
async fn main() -> Result<()> { 
    log::info!("HELLOWOROOASDSADASDASDASDA");
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .init();

    let mut opt = Opt::from_args();
    let res = check_current_engine().and_then(|curr_engine| Ok(async move {
        if opt.engine.is_none() {
            opt.engine = curr_engine;
        }
        if curr_engine.is_some() && opt.engine != curr_engine {
            log::error!("Wrong engine!");
            exit(1);
        }
        run_server(opt).await
    }));
    if let Err(e) = res {
        log::error!("{}", e);
        exit(1)
    }

    Ok(())

}

async fn run_server(opt: Opt) -> Result<()> {
    // let current_engine = opt.engine.unwrap_or(Engines::kvs);
    let current_engine = Engines::kvs;
    
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    log::info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    
    log::info!("Storage engine: {}", current_engine);
    
    log::info!("Listening on {}", opt.addr);

    std::fs::write(current_dir()?.join("engines"), format!("{current_engine}"))?;

    let pool = RayonThreadPool::new(1)?;

    match current_engine {
        Engines::kvs => {
            log::info!("KVS {:?}", env::current_dir()?);

            let mut path = env::current_dir()?;
            // path.push(format!("new_test_{}", chrono::Utc::now().timestamp()));
            server_spawn(KvStore::open(path)?, pool, opt.addr).await?;


        },
        Engines::sled => {
            log::info!("sled");
            server_spawn(
                SledKvsEngine::new(sled::open(current_dir()?)?),
                pool,
                opt.addr
            ).await?;
        },
    }
    Ok(())
}


async fn server_spawn<E: KvsEngine, P: ThreadPool>(engine: E, pool: P, addr: SocketAddr) -> Result<()> {
    let server = KvsServer::new(engine);
    server.run(addr).await?;


    Ok(())
}