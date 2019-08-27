use std::collections::HashMap;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::{Serialize, Deserialize};
use crypto::digest::Digest;
use crypto::sha2::Sha256;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message{
    pub nonce: u32,
    pub bytes: String
}

impl Message {
    pub fn new(nonce: u32, bytes: String) -> Message {
        Message{
            nonce,
            bytes
        }
    }

    pub fn clone(&self) -> Message {
        Message::new(self.nonce, self.bytes.clone())
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn deserialize(str: String) -> Message {
        serde_json::from_str(&str).unwrap()
    }

    pub fn generate_random_string() -> String {
        let mut rng = thread_rng();
        let n = rng.gen_range(0, 128);
        rng.sample_iter(&Alphanumeric)
            .take(n)
            .collect::<String>()
    }

    pub fn generate_digest(input: &String) -> String {
        let mut hasher = Sha256::new();
        hasher.input_str(input);
        hasher.result_str()
    }

    pub fn generate_cumulative_hash(messages: &HashMap<u32, String>, digests: &mut HashMap<u32, String>, mut index: u32){
        let mut current;
        let mut last_digest = None;
        if index == 0 {
            last_digest = Some(String::from(""));
            current = messages.get(&0);
        } else {
            current = messages.get(&index);
            if let Some(v) = digests.get(&(index - 1)) {
                last_digest = Some(String::from(v));
            }
        }

        while last_digest.is_some() && current.is_some() {
            let digest_input = last_digest.unwrap() + current.unwrap();
            let digest = Message::generate_digest(&digest_input);
            digests.insert(index, digest.clone());
            println!("Server generate digest for message: nonce={}, digest={}",
                     index, digest);
            index += 1;
            last_digest = Some(digest);
            current = messages.get(&index);
        }
    }
}


