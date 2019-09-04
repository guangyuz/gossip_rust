use std::thread;
use std::time::Duration;
use std::net::{Incoming, SocketAddr, TcpListener, TcpStream};
use std::io::prelude;
use std::io::Write;
use std::env;

mod server;
mod message;
use server::Server;
use message::Message;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("too few arguments");
    } else {
        let address = &args[1];
        let peers = &args[2];

        // Create a new Server
        let mut server = Server::new(address);
        server.join(peers);
        server.run();
    }
}

