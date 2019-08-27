use std::net::{Incoming, SocketAddr, TcpListener, TcpStream, IpAddr};
use std::io::prelude;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::collections::HashMap;

use time::*;
use crate::message::Message;

struct Listener {
    listener: TcpListener,
    sender: Sender<String>
}

impl Listener {
    pub fn new(address: SocketAddr, sender: Sender<String>) -> Listener {
        let listener = TcpListener::bind(address).unwrap();
        Listener {
            listener,
            sender
        }
    }

    pub fn run(&mut self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut s) => {
                    let sender = self.sender.clone();
                    thread::spawn(move || {
                        let message = Listener::retrieve_message(s);
                        sender.send(message);
                    });
                },
                Err(e) => println!("Error: {}", e)
            }
        }
    }

    fn retrieve_message(mut stream: TcpStream) -> String {
        let mut buffer = [0; 256];
        let n = stream.read(&mut buffer).unwrap();
        let mut message  = String::from_utf8_lossy(&buffer[0..n]);
        message.to_string()
    }
}

struct Broadcaster {
    receivers: Vec<SocketAddr>,
    fan_out: u8
}

impl Broadcaster {
    pub fn new(receivers: Vec<SocketAddr>) -> Broadcaster {
        Broadcaster{
            receivers,
            fan_out: 3
        }
    }

    pub fn broadcast (&self, message: String) {
        let mut counter = 0;
        for i in &self.receivers {
            if counter >= self.fan_out {
                break;
            }
            if let Ok(mut stream) = TcpStream::connect(i) {
                stream.write(message.as_bytes());
            } else {
                println!("Client #{} couldn't connect to server...", i);
            }
            counter += 1;
        }
    }
}

#[derive(Debug)]
pub struct Peer {
    address: SocketAddr
}

impl Peer {
    pub fn new(server_details: String) -> Peer {
        let address: SocketAddr = server_details
            .parse()
            .expect("Unable to parse socket address");
        Peer {
            address
        }
    }
}

#[derive(Debug)]
pub struct Server {
    address: SocketAddr,
    peers: Vec<Peer>,
    messages: HashMap<u32, String>,
    digests: HashMap<u32, String>,
    sender: Sender<String>,
    receiver: Receiver<String>
}

impl Server {
    pub fn new(server_details: &String) -> Server {
        let address: SocketAddr = server_details.to_string()
            .parse()
            .expect("Unable to parse socket address");
        let (sender, receiver) = channel();
        Server {
            address,
            peers: Vec::new(),
            messages: HashMap::new(),
            digests: HashMap::new(),
            sender,
            receiver
        }
    }

    pub fn run(&mut self) {

        // Create Listener in new thread
        let stream_sender = self.sender.clone();
        let address = self.address;
        thread::spawn(move || {
            Listener::new(address, stream_sender).run();
        });

        let start_time = time::get_time();
        // Main loop of the server
        for content in self.receiver.iter() {
            let mut message: Message = Message::deserialize(content);
            let mut index = message.nonce;
            if self.messages.contains_key(&index) {
                continue; // already exist, do nothing
            }
            // step 1: broadcast the message
            let broadcast_message = message.clone();
            let mut receivers = Vec::new();
            for i in &self.peers {
                receivers.push(i.address);
            }
            thread::spawn(move || {
                Broadcaster::new(receivers)
                    .broadcast(broadcast_message.serialize());
            });

            // step 2: save the message
            println!("Server {}, {:?}, message: nonce={}, bytes={}",
                     self.address, time::get_time() - start_time, index, message.bytes);
            self.messages.insert(index, message.bytes);

            // step 3: generate cumulative hash for the accepted messages
            Message::generate_cumulative_hash(&self.messages, &mut self.digests, index);
        }
    }

    pub fn join(&mut self, address: &String) {
        let peer = Peer::new(address.to_string());
        self.peers.push(peer);
    }

}
