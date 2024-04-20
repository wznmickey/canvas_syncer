use std::io::Read;

use crate::course::*;
use reqwest::blocking;
use reqwest::blocking::Response;
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
    fn get_remote_resource(&self, url: &str) -> Vec<Response> {
        let mut page_num = 1;
        let mut ans = Vec::new();
        loop {
            let _ = match || -> Option<()> {
                let myurl = url.to_string() + "&page=" + page_num.to_string().as_str();
                let body = self.client.get(myurl).send().ok()?;
                // println!("{:?}", body.headers().get("link").unwrap());
                if (body
                    .headers()
                    .get("link")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .contains("rel=\"next\""))
                {
                    page_num = page_num + 1;
                    ans.push(body);
                    Some(())
                } else {
                    ans.push(body);
                    None
                }
            }() {
                Some(_) => {}
                None => {
                    break;
                }
            };
        }

        ans
    }
    pub fn get_course_list(&self) -> Vec<Course> {
        let mut ans: Vec<Course> = Vec::new();
        let responses = self
            .get_remote_resource(format!("{}/api/v1/courses?include[]=term", self.url).as_str());
        for response in responses {
            let _ = match || -> Option<()> {
                let result: Value = response.json().ok()?;
                let result: &Vec<Value> = result.as_array()?;
                for i in result {
                    let course = get_course_from_json(i);
                    println!("course={course:?}");
                    match course {
                        None => continue,
                        Some(course) => {
                            ans.push(course);
                        }
                    }
                }
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
    pub fn get_folder_list(&self,id:i64)->Vec<Folder>{
        let url = format!("{}/api/v1/courses/{}/folders?",self.url,id);
        let responses = self.get_remote_resource(url.as_str());
        let mut ans = Vec::new();
        for response in responses {
            let _ = match || -> Option<()> {
                let result: Value = response.json().ok()?;
                let result: &Vec<Value> = result.as_array()?;
                for i in result {
                    let folder = get_folder_from_json(i);
                    println!("course={folder:?}");
                    match folder {
                        None => continue,
                        Some(folder) => {
                            ans.push(folder);
                        }
                    }
                }
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
}
