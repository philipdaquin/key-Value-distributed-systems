use clap::{Arg, Command, command, Subcommand, ArgMatches};
use project_2::kvs::{KvStore, Cache};
use project_2::error::{Result, CacheError};

use std::env::current_dir;
use std::process::exit;

fn main() -> Result<()> { 
    let matches = Command::new("cargo")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .name(env!("CARGO_PKG_NAME"))
        
        .subcommand(
            Command::new("set")
                .about("Set the value of a string key to a string")
                .arg(Arg::new("KEY").help("A string key").required(true))
                .arg(
                    Arg::new("VALUE")
                        .help("The string value of the key")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("cargo")
                .name("get")
                .about("get the value of a Key")
                .arg(
                    Arg::new("KEY")
                        .help("A string key")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("cargo")
                .name("rm")
                .about("removes a given key")
                .arg(
                    Arg::new("KEY")
                        .help("A string key")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("cargo")
                .name("-V")
                .about("version of the key value storage")
        )
        
        
        .get_matches();
        match matches.subcommand() { 
            Some(("set", arg)) => {     
                let mut store = KvStore::open(current_dir()?)?;


                let key = &*arg.get_one::<String>("KEY").expect("Missing key");
                let value = &*arg.get_one::<String>("VALUE").expect("Missing value");
                println!("Adding a {key} : {value}");
                
                store.set(key.to_string(), value.to_string())?;

            },
            Some(("get", arg)) => {
                let mut store = KvStore::open(current_dir()?)?;

                println!("Getting the value for key: {arg:?}");

                let key = &*arg.get_one::<String>("KEY").expect("Missing key");
                println!("Getting value for Key: {key}");

                if let Some(val) = store.get(key.to_string())? { 
                    println!("{val:?}");
                } else { 
                   return Err(CacheError::KeyNotFound)
                }
            },
            Some(("rm", arg)) => {
                let mut store = KvStore::open(current_dir()?)?;
                
                let key = &*arg.get_one::<String>("KEY").expect("Missing key");
                println!("Remove the key for: {key}");

                match store.remove(key.to_string()) {
                    Err(CacheError::KeyNotFound) => { 
                        println!("Key not found!"); 
                        exit(1)
                    },
                    Ok(()) => {},
                    Err(e) => return Err(e)
                };

            },
            Some(("-V", arg)) => {
                KvStore::version()
            },
            _ => panic!()
        }

        Ok(())
}



fn execute(matches: ArgMatches) -> Result<()> {
    match matches.subcommand() { 
            Some(("set", arg)) => {     
                let mut store = KvStore::open(current_dir()?)?;


                let key = &*arg.get_one::<String>("KEY").expect("Missing key");
                let value = &*arg.get_one::<String>("VALUE").expect("Missing value");
                println!("Adding a {key} : {value}");
                
                store.set(key.to_string(), value.to_string())?;

            },
            Some(("get", arg)) => {
                let mut store = KvStore::open(current_dir()?)?;

                println!("Getting the value for key: {arg:?}");

                let key = &*arg.get_one::<String>("KEY").expect("Missing key");
                println!("Getting value for Key: {key}");

                if let Some(val) = store.get(key.to_string())? { 
                    println!("{val:?}");
                } else { 
                   return Err(CacheError::KeyNotFound)
                }
            },
            Some(("rm", arg)) => {
                let mut store = KvStore::open(current_dir()?)?;
                
                let key = &*arg.get_one::<String>("KEY").expect("Missing key");
                println!("Remove the key for: {key}");

                match store.remove(key.to_string()) {
                    Err(CacheError::KeyNotFound) => { 
                        println!("Key not found!"); 
                        exit(1)
                    },
                    Ok(()) => {},
                    Err(e) => return Err(e)
                };

            },
            _ => panic!()
        }

        Ok(())

}