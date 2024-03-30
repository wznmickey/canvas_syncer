use crate::course::*;
use crate::my_key::*;
use reqwest::blocking;
use reqwest::header;
use reqwest::{self};
use serde_json::Value;
pub struct RemoteData {
    client: reqwest::blocking::Client,
}

impl RemoteData {
    pub fn new() -> Self {
        let mut header = header::HeaderMap::new();
        header.insert(
            "Authorization",
            header::HeaderValue::from_str(my_key()).unwrap(),
        );
        Self {
            client: blocking::ClientBuilder::new()
                .default_headers(header)
                .build()
                .unwrap(),
        }
    }
    pub fn get_course_list(self) -> Vec<Course> {
        let mut ans: Vec<Course> = Vec::new();
        let mut page = 1;
        loop {
            let _ = match || -> Option<()> {
                let body = self
                    .client
                    .get(format!(
                        "https://jicanvas.com/api/v1/courses?include[]=term&page={}",
                        page
                    ))
                    .send()
                    .ok()?;
                let result: Value = body.json().ok()?;
                let result: &Vec<Value> = result.as_array()?;
                for i in result {
                    let course = get_course_from_json(i)?;
                    println!("course={course:?}");
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
}
