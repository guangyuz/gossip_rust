use std::net::{Incoming, SocketAddr, TcpListener, TcpStream, IpAddr};
use std::io::prelude;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::collections::HashMap;

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
    fan_out: u8,
    messages: HashMap<u32, String>,
    digests: HashMap<u32, String>,
    message_num: usize,
    sender: Sender<String>,
    receiver: Receiver<String>
}

impl Server {
    pub fn new(server_details: String) -> Server {
        let address: SocketAddr = server_details
            .parse()
            .expect("Unable to parse socket address");
        let (sender, receiver) = channel();
        Server {
            address,
            peers: Vec::new(),
            fan_out: 3,
            messages: HashMap::new(),
            digests: HashMap::new(),
            message_num: 0,
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

        // Main loop of the server
        for content in self.receiver.iter() {
            let mut message: Message = Message::deserialize(content);
            let mut index = message.nonce;
            if self.messages.contains_key(&index) {
                // already exist, do nothing
            } else {
                // step 1
                let broadcast_message = message.clone();
                self.broadcast(broadcast_message.serialize());

                // step 2
                println!("Server {} received message: nonce={}, bytes={}", self.address, message.nonce,
                         message.bytes);
                self.messages.insert(index, message.bytes);


                // step 3
                //self.generate_digest(index);
                if index == 0 {
                    let digest_input = self.messages.get(&index).unwrap();
                    self.digests.insert(0,Message::generate_digest(digest_input));
                } else {
                    let mut current = self.messages.get(&index);
                    let mut last = self.messages.get(&(index - 1));
                    //let current_digest = self.digests.get(&(index));
                    let mut last_digest =  String::new();
                    let last_digest_is_some = self.digests.get(&(index - 1)).is_some();
                    if last_digest_is_some {
                        last_digest = String::from(self.digests.get(&(index - 1)).unwrap());
                    }

                    while last.is_some() && last_digest_is_some && current.is_some() {
                        let digest_input = last_digest + current.unwrap();
                        let digest = Message::generate_digest(&digest_input);
                        self.digests.insert(index, digest.clone());
                        println!("Server {} message: nonce={}, digest={}",
                                 self.address, index, digest);
                        index += 1;
                        last = current;
                        last_digest = digest;
                        current = self.messages.get(&index);
                    }
                }
            }
        }
    }

    fn broadcast (&self, message: String) {
        let mut counter = 0;
        for i in &self.peers {
            if counter >= self.fan_out {
                break;
            }
            if let Ok(mut stream) = TcpStream::connect(i.address) {
                stream.write(message.as_bytes());
            } else {
                println!("Client #{} couldn't connect to server...", i.address);
            }
            counter += 1;
        }
    }
    pub fn join(&mut self, address: String) {
        let peer = Peer::new(address);
        self.peers.push(peer);
    }

}
