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
    fn ten_nodes() {

        let address_list = vec![
            "127.0.0.1:8080",
            "127.0.0.1:8081",
            "127.0.0.1:8082",
            "127.0.0.1:8083",
            "127.0.0.1:8084",
            "127.0.0.1:8085",
            "127.0.0.1:8086",
            "127.0.0.1:8087",
            "127.0.0.1:8088",
            "127.0.0.1:8089",
            "127.0.0.1:8080", // add the first address twice, to receive broadcast
        ];

        //let mut servers = Vec::new();
        let mut handles = Vec::new();

        for i in 0..10 {
            let mut server = Server::new(&address_list[i].to_string());
            server.join(&address_list[i + 1].to_string());
            //servers.push(&server);
            let handle = thread::spawn(move || {
                server.run();
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }
    }
}