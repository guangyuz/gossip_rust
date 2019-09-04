# gossip_rust

## What is it

This is a simplified gossip protocol implementation, just to practice
Rust programming skills. The function is very limited and unstable.
It tries to meet the following goals:
1. The client randomly sends packets to a node.  
2. The node sorts the messages by ID and calculates the hash value. The hash result of the node is consistent with the client.  
3. The message is broadcast to a randomly selected peer. The hash result of the peer is also consistent with the client.  
4. (not implemented) Node support restarts at any time, dynamically discover new nodes  

## How to run

Below example shows how to setup a three nodes network and send message
to it.

the network connection diagram:  
server A --> server B, server C  
server B --> server C  
server C --> server B  

first, lunch server A in one terminal:  
```cargo run 127.0.0.1:8080 '127.0.0.1:8081;127.0.0.1:8082'```  
then, lunch server B in another terminal:  
```cargo run 127.0.0.1:8081 127.0.0.1:8082```  
and lunch server C in the third terminal:  
```cargo run 127.0.0.1:8082 127.0.0.1:8081```  
last, start client in a new terminal,which will send 1000 random messages to server A:    
```cargo test --package gossiprust --test start_client -- --nocapture```


If you don't want to lunch servers one by one, there are also some 
codes in the 'tests' folder, you can lunch multiple servers directly.  
lunch_2_servers.rs,  
lunch_3_servers.rs,  
lunch_10_servers.rs,  
lunch_100_servers.rs  

for example to lunch 10 servers in one terminal, with each server running 
in single thread:  
```cargo test --package gossiprust --test lunch_10_servers -- --nocapture```  
then start client in another terminal:  
```cargo test --package gossiprust --test start_client -- --nocapture```
