use dialoguer::{theme::ColorfulTheme, Input};
use serde::{Deserialize, Serialize};
use std::fs::*;
use std::io::*;

use crate::filter::Filters;
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
        let mut key: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(t!("Your Canvas key"))
            .interact_text()
            .unwrap();
        let local_place: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(t!("Place to download files:"))
            .interact_text()
            .unwrap();
        let canvas_url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(t!("Your Canvas URL,like https://oc.sjtu.edu.cn"))
            .interact_text()
            .unwrap();
        if !key.starts_with("B") {
            key = "Bearer ".to_string() + &key;
        }
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
