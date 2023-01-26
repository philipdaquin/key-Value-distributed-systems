use std::net::{TcpListener, TcpStream};
use crate::engines::KvsEngine;

pub struct KvsServer<E: KvsEngine> { 
    engine: E
}

impl<E> KvsServer<E> where E: KvsEngine { 
    fn new(engine: E)  
    
    fn run(){ 

    }
}