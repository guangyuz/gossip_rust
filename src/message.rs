use std::collections::HashMap;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message{
    pub nonce: u32,
    pub bytes: String,
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
        let n = rng.gen_range(0, 512);
        rng.sample_iter(&Alphanumeric)
            .take(n)
            .collect::<String>()
    }

    pub fn generate_digest(input: &String) -> String {
        let mut hasher = Sha256::new();
        hasher.input_str(input);
        hasher.result_str()
    }
}


