use crate::course::*;
use reqwest::blocking;
use reqwest::header;
use reqwest::{self};
use serde_json::Value;
pub struct RemoteData {
    client: reqwest::blocking::Client,
    url: String,
}

impl RemoteData {
    pub fn new(key: &str, url: &str) -> Self {
        let mut header = header::HeaderMap::new();
        header.insert("Authorization", header::HeaderValue::from_str(key).unwrap());
        Self {
            client: blocking::ClientBuilder::new()
                .default_headers(header)
                .build()
                .unwrap(),
            url: url.to_string(),
        }
    }
    pub fn get_course_list(&self) -> Vec<Course> {
        let mut ans: Vec<Course> = Vec::new();
        let mut page = 1;
        loop {
            let _ = match || -> Option<()> {
                let body = self
                    .client
                    .get(format!(
                        "{}/api/v1/courses?include[]=term&page={}",
                        self.url, page
                    ))
                    .send()
                    .ok()?;
                let result: Value = body.json().ok()?;
                let result: &Vec<Value> = result.as_array()?;
                for i in result {
                    let course = get_course_from_json(i)?;
                    // println!("course={course:?}");
                    ans.push(course);
                }
                page += 1;
                Some(())
            }() {
                Some(_) => {}
                None => {
                    break;
                }
            };
        }
        ans
    }
    // pub fn get_folder_list(self,id:i64)->String{

    // }
}
