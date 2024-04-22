use crate::course::*;
use reqwest::blocking;
use reqwest::blocking::Response;
use reqwest::header;
use reqwest::*;
use reqwest::{self};
use serde_json::Value;
// use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct RemoteData {
    client: reqwest::blocking::Client,
    url: String,
    async_client: reqwest::Client,
}

impl RemoteData {
    pub fn new(key: &str, url: &str) -> Self {
        let mut header = header::HeaderMap::new();
        header.insert("Authorization", header::HeaderValue::from_str(key).unwrap());
        Self {
            client: blocking::ClientBuilder::new()
                .default_headers(header.clone())
                .build()
                .unwrap(),
            url: url.to_string(),
            async_client: reqwest::ClientBuilder::new()
                .default_headers(header)
                .build()
                .unwrap(),
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
                if body
                    .headers()
                    .get("link")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .contains("rel=\"next\"")
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
    pub fn get_course_list(&self) -> Vec<Rc<Course>> {
        let mut ans = Vec::new();
        let responses = self
            .get_remote_resource(format!("{}/api/v1/courses?include[]=term", self.url).as_str());
        for response in responses {
            let _ = match || -> Option<()> {
                let result: Value = response.json().ok()?;
                let result: &Vec<Value> = result.as_array()?;
                for i in result {
                    let course = get_course_from_json(i);
                    // println!("course={course:?}");
                    match course {
                        None => continue,
                        Some(course) => {
                            let rc = Rc::new(course);
                            ans.push(rc);
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
    pub fn get_folder_list(&self, course: Rc<Course>) -> Vec<Rc<Folder>> {
        let url = format!("{}/api/v1/courses/{}/folders?", self.url, course.id);
        let responses = self.get_remote_resource(url.as_str());
        let mut ans = Vec::new();
        for response in responses {
            let _ = match || -> Option<()> {
                let result: Value = response.json().ok()?;
                let result: &Vec<Value> = result.as_array()?;
                for i in result {
                    let folder = get_folder_from_json(i, Rc::clone(&course));
                    // println!("folder={folder:?}");
                    match folder {
                        None => continue,
                        Some(folder) => {
                            let rc = Rc::new(folder);
                            ans.push(rc);
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

    pub fn get_file_list(&self, folder: Rc<Folder>, path: PathBuf) -> Vec<Rc<File>> {
        let url = folder.filelink.as_str();
        // println!("{url}");
        let responses = self.get_remote_resource(url);
        let mut ans = Vec::new();
        for response in responses {
            let _ = match || -> Option<()> {
                let result: Value = response.json().ok()?;
                let result: &Vec<Value> = result.as_array()?;
                for i in result {
                    let file = get_file_from_json(i, Rc::clone(&folder), path.clone());
                    // println!("file={file:?}");
                    match file {
                        None => continue,
                        Some(file) => {
                            let rc = Rc::new(file);
                            ans.push(rc);
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

    pub async fn download_file(&self, path: &Path, url: &str) -> () {
        let temp = self.async_client.get(url).send().await;
        match temp {
            Ok(temp) => {
                let temp = &temp.bytes().await;
                match temp {
                    Ok(temp) => {
                        let temp_file = std::fs::File::create(path);
                        let mut file: std::fs::File;
                        match temp_file {
                            Err(e) => {
                                println!("{e}");
                                return;
                            }
                            Ok(temp) => file = temp,
                        };
                        match file.write(temp) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("{e}");
                            }
                        };
                    }
                    Err(e) => {
                        println!("{e}")
                    }
                };
            }
            Err(e) => println!("{e}"),
        }
    }
}
