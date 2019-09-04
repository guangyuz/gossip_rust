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
    fn one_hundred_nodes() {

        //let mut servers = Vec::new();
        let mut handles = Vec::new();

        for i in 0..100 {
            let mut server = Server::new(&("127.0.0.1:".to_string()+ &(8080+i).to_string()));
            server.join(&("127.0.0.1:".to_string()+&(8080+i+1).to_string()));
            //servers.push(&server);
            let handle = thread::spawn(move || {
                server.run();
            });
            handles.push(handle);
        }

        let mut server = Server::new(&("127.0.0.1:8180").to_string());
        server.join(&("127.0.0.1:8080").to_string());
        //servers.push(&server);
        let handle = thread::spawn(move || {
            server.run();
        });
        handles.push(handle);

        for h in handles {
            h.join().unwrap();
        }
    }
}