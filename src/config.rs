use dialoguer::{theme::ColorfulTheme, Input};
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
    pub fn print(&self) -> () {
        println!(
            "local_place={local_place},canvas_url={canvas_url}",
            local_place = self.local_place,
            canvas_url = self.canvas_url
        )
    }
    pub fn read_file(s: &str) -> Self {
        let file = File::open(s).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).expect("Error while reading config file")
    }
    // pub fn new(key: &str, local_place: &str, canvas_url: &str) -> Self {
    //     Self {
    //         key: key.to_string(),
    //         local_place: local_place.to_string(),
    //         canvas_url: canvas_url.to_string(),
    //     }
    // }
    pub fn new() -> Self {
        let key: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Your Canvas key")
            .interact_text()
            .unwrap();
        let local_place: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Place to download files:")
            .interact_text()
            .unwrap();
        let canvas_url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Your Canvas URL,like https://oc.sjtu.edu.cn")
            .interact_text()
            .unwrap();
        Self {
            key: key,
            local_place: local_place,
            canvas_url: canvas_url,
        }
    }
    pub fn save(&self, str: &str) {
        let mut temp = std::fs::File::create(str).expect("Wrong in creating the config file");
        let data = serde_json::to_string(&self).expect("Wrong in creating the config file");
        temp.write(data.as_bytes())
            .expect("Wrong in creating the config file");
    }
}
