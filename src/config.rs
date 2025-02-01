use crate::filter::Filters;
use inquire::{required, Text};
use serde::{Deserialize, Serialize};
use std::fs::*;
use std::io::*;
#[derive(Serialize, Deserialize)]

pub struct Config {
    pub key: String,
    pub local_place: String,
    pub canvas_url: String,
    pub allow_term: bool,
    pub filters: Option<Filters>,
}

impl Config {
    pub fn print(&self) {
        info!(
            "local_place={:?},canvas_url={:?},{:?}",
            self.local_place, self.canvas_url, self.filters
        );
    }
    pub fn read_file(s: &str) -> Vec<Self> {
        let file = File::open(s).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).expect("Error while reading config file")
    }
    pub fn new() -> Self {
        let canvas_url = Text::new(&t!("Your Canvas URL"))
            .with_help_message(&t!("format would looks like https://oc.sjtu.edu.cn"))
            .with_validator(required!())
            .prompt()
            .unwrap();
        let key = Text::new(&t!("Your Canvas key"))
            .with_formatter(&|s| {
                if !s.starts_with("B") {
                    "Bearer ".to_string() + s
                } else {
                    s.to_string()
                }
            })
            .with_validator(required!())
            .prompt()
            .unwrap();
        let local_place = Text::new(&t!("Place to download files"))
            .with_default("./canvas")
            .with_help_message(&t!("default is ./canvas"))
            .with_validator(required!())
            .prompt()
            .unwrap();
        Self {
            key,
            local_place,
            canvas_url,
            allow_term: true,
            filters: None,
        }
    }
    pub fn save(&self, str: &str) {
        let mut temp = std::fs::File::create(str).expect("Wrong in creating the config file");
        let vec = vec![self];
        let data: String = serde_json::to_string(&vec).expect("Wrong in creating the config file");
        temp.write_all(data.as_bytes())
            .expect("Wrong in creating the config file");
    }
}
