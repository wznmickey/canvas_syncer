use serde::{Deserialize, Serialize};
use std::fs::*;
use std::io::*;
#[derive(Serialize, Deserialize)]

pub struct Config {
    pub key: String,
    pub local_place: String,
    pub canvas_url: String,
}


impl Config {
    pub  fn print(&self) -> () {
        println!(
            "local_place={local_place},canvas_url={canvas_url}",
            local_place = self.local_place,
            canvas_url = self.canvas_url
        )
    }
    pub  fn read_file(s:&str) -> Self {
        let file = File::open(s).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).expect("Error while reading config file")
    }
    pub fn new() -> Self {
        Self {
            key: "".to_string(),
            local_place: "".to_string(),
            canvas_url: "".to_string(),
        }
    }
}
