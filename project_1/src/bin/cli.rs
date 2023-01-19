use clap::{Arg, Command, command, Subcommand};


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

        match matches.subcommand_name() { 
            Some("set") => {            
                eprintln!("unimplemented");
            },
            Some("get") => {
                eprintln!("unimplemented");

            },
            Some("rm") => {
                eprintln!("unimplemented");

            },
            _ => panic!()
        }
        
}