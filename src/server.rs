use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::error::Error;

use crate::message::Message;
use rand::{thread_rng, Rng};

extern crate tokio;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

extern crate futures;
use futures::Future;
use futures::future::*;
use futures::sync::mpsc::{Sender, Receiver, channel};

#[derive(Debug)]
struct Broadcaster {
    receivers: Vec<SocketAddr>,
}

impl Broadcaster {
    pub fn new(receivers: Vec<SocketAddr>) -> Broadcaster {
        Broadcaster{
            receivers,
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
pub struct Shared {
    messages: HashMap<u32, String>,
    digests: HashMap<u32, String>,
}

impl Shared {
    fn new() -> Self {
        Shared {
            messages: HashMap::new(),
            digests: HashMap::new(),
        }
    }
}

pub struct Server {
    address: SocketAddr,
    peers: Vec<Peer>,
}

impl Server {
    pub fn new(server_details: &String) -> Server {
        let address: SocketAddr = server_details.to_string()
            .parse()
            .expect("Unable to parse socket address");
        Server {
            address,
            peers: Vec::new(),
        }
    }

    fn broadcast_task(rx: Receiver<String>, broadcaster: Broadcaster)
        -> impl Future<Item = (), Error = ()>
    {
        rx.filter(|msg| msg.ne("please ignore")).for_each(move |msg| {
            let target = thread_rng().gen_range(0, broadcaster.receivers.len());
            let addr = broadcaster.receivers[target];
            TcpStream::connect(&addr)
                .and_then(|stream| {
                    io::write_all(stream, msg.into_bytes())
                        .then(|result| {
                            Ok(())
                        })
                })
                .map_err(|err| {
                    println!("connection error = {:?}", err);
                })
        })
    }

    fn generate_cumulative_hash(state: Arc<Mutex<Shared>>, mut index: u32){
        let mut current = None;
        let mut last_digest = None;
        if index == 0 {
            last_digest = Some(String::from(""));
            if let Some(v) = state.lock().unwrap().messages.get(&0) {
                current = Some(String::from(v));
            }
        } else {
            if let Some(v) = state.lock().unwrap().messages.get(&index) {
                current = Some(String::from(v));
            }
            if let Some(v) = state.lock().unwrap().digests.get(&(index - 1)) {
                last_digest = Some(String::from(v));
            }
        }

        while last_digest.is_some() && current.is_some() {
            let digest_input = last_digest.unwrap() + &current.unwrap();
            let digest = Message::generate_digest(&digest_input);
            state.lock().unwrap().digests.insert(index, digest.clone());
            println!("Server generate digest for message: nonce={}, digest={}, {:?}",
                     index, digest, time::get_time());
            index += 1;
            last_digest = Some(digest);
            current = None;
            if let Some(v) = state.lock().unwrap().messages.get(&index) {
                current = Some(String::from(v));
            }
        }
    }

    fn process(socket: TcpStream, tx: Sender<String>, state: Arc<Mutex<Shared>>) {
        let done = io::read_to_end(socket, vec![])
            .and_then(move |(_, buf)| {
                let mut content  = String::from_utf8_lossy(&buf[..]);

                let mut message_to_broadcast = content.to_string().clone();
                let mut message: Message = Message::deserialize(content.to_string());
                let mut index = message.nonce;

                if !state.lock().unwrap().messages.contains_key(&index) {
                    state.lock().unwrap().messages.insert(index, message.bytes);
                    Server::generate_cumulative_hash( state.clone(), index);
                } else {
                    message_to_broadcast = String::from("please ignore");
                }

                tx.send(message_to_broadcast)
                    .map_err(|_| io::ErrorKind::Other.into())
            })
            .map(|_| println!("write complete"))
            .map_err(|e| println!("socket error = {:?}", e));

        tokio::spawn(done);
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let address = self.address;
        let peers = &self.peers;
        let mut receivers = Vec::new();
        for i in peers {
            receivers.push(i.address);
        }
        tokio::run(lazy(move || {
            let listener = TcpListener::bind(&address).unwrap();
            let (tx, rx) = channel(1_024);
            let state = Arc::new(Mutex::new(Shared::new()));

            let broadcaster = Broadcaster::new(receivers);
            tokio::spawn(Server::broadcast_task(rx, broadcaster));

            listener.incoming()
                .map_err(|e| println!("accept failed = {:?}", e))
                .for_each(move |socket| {
                    let tx = tx.clone();
                    Server::process(socket, tx, state.clone());
                    Ok(())
                })
        }));
        Ok(())
    }

    pub fn join(&mut self, address: &String) {
        let addrs: Vec<&str> = address.split(';').collect();
        for addr in addrs {
            let peer = Peer::new(addr.to_string());
            self.peers.push(peer);
        }
    }

}
