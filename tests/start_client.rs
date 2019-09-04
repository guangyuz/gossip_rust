#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;
    use std::net::{Incoming, SocketAddr, TcpListener, TcpStream};
    use std::io::prelude;
    use std::io::Write;
    use std::env;

    extern crate gossiprust;
    use gossiprust::server::Server;
    use gossiprust::message::Message;

    #[test]
    fn client() {

        let mut total_msg = 1000;
        let args: Vec<String> = env::args().collect();
        if args.len() < 2 {
            println!("too few arguments, use default msg# value: 1000");
        } else {
            //total_msg = *&args[1].parse::<u32>().unwrap();
        }

        // Send message to Server
        let mut digest = String::from("");
        for i in 0..total_msg {
            if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8080") {
                let content = Message::generate_random_string();
                let digest_input = digest + &content;
                digest = Message::generate_digest(&digest_input);
                let message = Message::new(i, content);
                println!("Client send message: nonce={}, bytes={}, digest={}", message.nonce,
                         message.bytes, digest);
                stream.write(message.serialize().as_bytes());
                stream.flush().unwrap();
            } else {
                println!("Client #{} couldn't connect to server...", i);
            }
        }

    }
}