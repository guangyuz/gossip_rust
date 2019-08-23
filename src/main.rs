use std::thread;
use std::time::Duration;
use std::net::{Incoming, SocketAddr, TcpListener, TcpStream};
use std::io::prelude;
use std::io::Write;
mod gossip;
use gossip::{Server, Content};
use serde::{Serialize, Deserialize};

fn main() {
    // Create a new Server
    let mut server_A = Server::new("127.0.0.1:8080".to_string());
    server_A.join("127.0.0.1:8081".to_string());
    let mut server_B = Server::new("127.0.0.1:8081".to_string());
    //server_B.join("127.0.0.1:8080".to_string());

    // Start server in new thread
    let handle_A = thread::spawn(move || {
        server_A.run();
    });
    let handle_B = thread::spawn(move || {
        server_B.run();
    });

    // Sleep 1 second
    thread::sleep(Duration::from_millis(1000));

    // Send message to Server
    for i in 0..10000 {
        if let Ok(mut stream) = TcpStream::connect("localhost:8080") {
            //println!("Client #{} connected to the server!", i);
            let content = Content::new(i, "hello".to_string());
            let serialized = serde_json::to_string(&content).unwrap();

            stream.write(serialized.as_bytes());
            //stream.flush().unwrap();
        } else {
            println!("Client #{} couldn't connect to server...", i);
        }
    }

    // todo
    handle_A.join().unwrap();
    handle_B.join().unwrap();
}

