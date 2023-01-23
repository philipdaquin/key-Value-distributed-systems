use clap::{Arg, Command, command, Subcommand, ArgMatches};
use project_2::kvs::KvStore;
use project_2::kvs::Cache;

fn main() { 
    let matches = Command::new("cargo")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .name(env!("CARGO_PKG_NAME"))
        
        .subcommand(
            Command::new("cargo")
                .name("set")
                .about("Set the value of any Key to any Value")
                .arg(
                    Arg::new("KEY")
                        .help("A string key")
                        .required(true)
                )
                .arg(
                    Arg::new("VALUE")
                    .help("the value of the key")
                    .required(true)
                )
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
        execute(matches);

        
}



fn execute(matches: ArgMatches) {
    let mut store = KvStore::new();
    match matches.subcommand() { 
            Some(("set", arg)) => {     
                
                let key = &*arg.get_one::<String>("KEY").expect("Missing key");
                let value = &*arg.get_one::<String>("KEY").expect("Missing value");
                println!("Adding a {key} : {value}");
                
                store.set(key.to_string(), value.to_string());

            },
            Some(("get", arg)) => {
                println!("Getting the value for key: {arg:?}");

                let key = &*arg.get_one::<String>("KEY").expect("Missing key");
                println!("Getting value for Key: {key}");

                let val = store.get(key.to_string());

                println!("Value: {val:?}");
                

            },
            Some(("rm", arg)) => {
                
                let key = &*arg.get_one::<String>("KEY").expect("Missing key");
                println!("Remove the key for: {key}");

                store.remove_key(key.to_string());

            },
            _ => panic!()
        }
}