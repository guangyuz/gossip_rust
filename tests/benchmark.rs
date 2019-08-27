#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;
    use std::net::{Incoming, SocketAddr, TcpListener, TcpStream};
    use std::io::prelude;
    use std::io::Write;

    extern crate gossiprust;
    use gossiprust::server::Server;
    use gossiprust::message::Message;

    #[test]
    fn two_nodes() {
        // Create a new Server
        let mut server_A = Server::new(&"127.0.0.1:8080".to_string());
        server_A.join(&"127.0.0.1:8081".to_string());
        let mut server_B = Server::new(&"127.0.0.1:8081".to_string());
        server_B.join(&"127.0.0.1:8080".to_string());

        // Start server in new thread
        let handle_A = thread::spawn(move || {
            server_A.run();
        });
        let handle_B = thread::spawn(move || {
            server_B.run();
        });

        // Sleep 1 second
        //thread::sleep(Duration::from_millis(1000));

        // Send message to Server
        let mut digest = String::from("");
        for i in 0..1000 {
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

        // todo
        handle_A.join().unwrap();
        handle_B.join().unwrap();
    }
}