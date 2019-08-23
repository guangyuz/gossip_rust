use std::net::{Incoming, SocketAddr, TcpListener, TcpStream, IpAddr};
use std::io::prelude;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crypto::sha2::Sha256;

#[derive(Serialize, Deserialize, Debug)]
pub struct Content{
    nonce: u32,
    bytes: String,
    cumulative_hash: Option<String>
}

impl Content {

    pub fn new(nonce: u32, bytes: String) -> Content {

        Content{
            nonce,
            bytes,
            cumulative_hash: None
        }
    }

}

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
                        let content = Listener::retrieve_content(s);
                        //println!("Listener received content");
                        // Sleep random seconds
                        //thread::sleep(Duration::from_millis(1000));
                        sender.send(content);
                    });
                },
                Err(e) => println!("Error: {}", e)
            }
        }
    }
    fn retrieve_content(mut stream: TcpStream) -> String {
        let mut buffer = [0; 256];
        let n = stream.read(&mut buffer).unwrap();
        let mut content  = String::from_utf8_lossy(&buffer[0..n]);
        content.to_string()
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
    id: Uuid,
    address: SocketAddr,
    peers: Vec<Peer>,
    fan_out: u8,
    message_list: Vec<Content>,
    message_num: usize,
    waiting_list: HashMap<u32, Content>,
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
            id: Uuid::new_v4(),
            address,
            peers: Vec::new(),
            fan_out: 3,
            message_list: Vec::new(),
            message_num: 0,
            waiting_list: HashMap::new(),
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
            println!("Main thread received content:{}", content);
            let message: Content = serde_json::from_str(&content).unwrap();
            let mut index = self.message_num as u32;
            let nonce = message.nonce;
            if nonce == index{
                let broadcast_message = Content::new(index, message.bytes.clone());
                self.message_list.insert(self.message_num, message);
                self.message_num += 1;

                index += 1;
                println!("Insert into Message list:{}", content);

                self.broadcast(serde_json::to_string(&broadcast_message).unwrap());
                //self.process_waiting_list();
                //index = self.message_num as u32;
                //let mut message_list = &self.message_list;
                //let mut waiting_list = &self.waiting_list;
                loop {
                    match self.waiting_list.get_mut(&index) {
                        Some(message) => {
                            let mut content = Content::new(index, message.bytes.clone());
                            self.message_list.insert(index as usize, content);
                            //self.waiting_list.remove(&index);
                            let mut broadcast_message = Content::new(index, message.bytes.clone());
                            self.message_num += 1;
                            index += 1;
                            println!("Insert into Message list:{}", message.nonce);

                            self.broadcast(serde_json::to_string(&broadcast_message).unwrap());

                        }
                        None => {
                            break;
                        }
                    }
                }
            } else if nonce > index {
                self.waiting_list.insert(message.nonce, message);
            } else {
                //nonce < message_num, do nothing
            }

            //self.broadcast(content);
        }
    }

    pub fn broadcast (&self, content: String) {
        println!("Main thread is broadcasting content:{}", content);
        let mut counter = 0;
        //for i in 0..self.peers.len() {
        for i in &self.peers {
            if counter >= self.fan_out {
                break;
            }
            //println!("Broadcast done: {}",self.peers[i].address);

            if let Ok(mut stream) = TcpStream::connect(i.address) {
                stream.write(content.as_bytes());
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
