#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;
    use std::io::prelude;
    use std::io::Write;

    extern crate gossiprust;
    use gossiprust::server::Server;
    use gossiprust::message::Message;

    extern crate tokio;
    use tokio::io;
    use tokio::net::TcpStream;
    use tokio::prelude::*;

    #[test]
    fn two_nodes() {
        // Create new Servers
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

        // todo
        handle_A.join().unwrap();
        handle_B.join().unwrap();
    }
}