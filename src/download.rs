use crate::course::*;
use indicatif::MultiProgress;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::header;
use reqwest::Response;
use serde_json::Value;
use std::cell::RefCell;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::Semaphore;
use tokio::task::yield_now;
use tokio::time::{sleep, Duration};
pub struct RemoteData {
    url: String,
    async_client: reqwest::Client,
    sem: Arc<Semaphore>,
}

impl RemoteData {
    pub fn new(key: &str, url: &str) -> Self {
        let mut header = header::HeaderMap::new();
        header.insert("Authorization", header::HeaderValue::from_str(key).unwrap());
        Self {
            url: url.to_string(),
            async_client: reqwest::ClientBuilder::new()
                .default_headers(header)
                .build()
                .unwrap(),
            // according to https://community.canvaslms.com/t5/Canvas-Developers-Group/API-Rate-Limiting/ba-p/255845 , it should be 700. But my test gives me 600. Maybe my canvas has a different setting.
            // https://canvas.instructure.com/doc/api/file.throttling.html
            sem: Arc::new(Semaphore::new(600)),
        }
    }
    async fn get_remote_resource(&self, url: &str) -> Vec<Response> {
        let mut page_num = 1;
        let mut ans = Vec::new();
        loop {
            let response: Response;
            let permit = self.sem.acquire_many(50).await.unwrap();
            let temp = self
                .async_client
                .get(url.to_string() + "&page=" + page_num.to_string().as_str())
                .send();
            match temp.await {
                Err(e) => {
                    println!("In getting {url} : {e}");
                    return ans;
                }
                Ok(body) => {
                    response = body;
                }
            }
            let temp: f64 = response
                .headers()
                .get("x-rate-limit-remaining")
                .unwrap()
                .to_str()
                .unwrap()
                .parse()
                .unwrap();
            if (temp < 0.0) {
                println!("In getting {url} : rate limit exceeded wait 10s");
                sleep(Duration::from_millis(1000 * 10)).await;
                continue;
            }

            if response.headers().get("status").unwrap().to_str().unwrap() == "401 Unauthorized" {
                return ans;
            }

            if response.headers().get("status").unwrap().to_str().unwrap() != "200 OK" {
                println!("{:?} in {:?}", response.headers(), url);
                sleep(Duration::from_millis(1000)).await;
                continue;
            }
            if response
                .headers()
                .get("link")
                .unwrap()
                .to_str()
                .unwrap()
                .contains("rel=\"next\"")
            {
                page_num = page_num + 1;
                ans.push(response);
            } else {
                ans.push(response);
                return ans;
            }
        }
    }

    async fn get_remote_json_list<A: Clone, B: Clone, T: GetFromJson<T, A, B>>(
        &self,
        st: &str,
        a: A,
        b: B,
    ) -> Vec<Rc<RefCell<T>>> {
        let mut ans = Vec::new();
        let responses = self.get_remote_resource(st);
        for response in responses.await {
            let result: Value;

            match response.json().await.ok() {
                None => continue,
                Some(temp) => {
                    result = temp;
                }
            }
            match result.as_array() {
                None => continue,
                Some(result) => {
                    for i in result {
                        let item = T::get_from_json(i, a.clone(), b.clone());
                        // println!("course={course:?}");
                        match item {
                            None => continue,
                            Some(item) => {
                                let rc = Rc::new(RefCell::new(item));
                                ans.push(rc);
                            }
                        }
                    }
                }
            }
        }

        ans
    }
    pub fn get_course_list(&self) -> Vec<Rc<RefCell<Course>>> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.get_course_list_helper())
    }
    async fn get_course_list_helper(&self) -> Vec<Rc<RefCell<Course>>> {
        let url = format!("{}/api/v1/courses?include[]=term", self.url);
        self.get_remote_json_list::<i32, i32, Course>(&url, 1, 1)
            .await
    }
    pub async fn get_folder_list(&self, course: Rc<RefCell<Course>>) -> Vec<Rc<RefCell<Folder>>> {
        let url = format!("{}/api/v1/courses/{}/folders?", self.url, course.borrow().id);
        self.get_remote_json_list::<Rc<RefCell<Course>>, i32, Folder>(&url, course, 1)
            .await
    }

    pub async fn get_file_list(&self, folder: Rc<RefCell<Folder>>, path: PathBuf) -> Vec<Rc<RefCell<CourseFile>>> {
        let url = &folder.borrow().filelink;
        // println!("{url}");
        self.get_remote_json_list::<Rc<RefCell<Folder>>, PathBuf, CourseFile>(&url, folder.clone(), path)
            .await
    }

    pub async fn download_file(
        &self,
        path: &Path,
        url: &str,
        file_name: &str,
        pb: &ProgressBar,
        m: &MultiProgress,
        size: u64,
    ) -> () {
        let mut buf: BufWriter<File>;
        let permit = self.sem.acquire_many(50).await.unwrap();
        match tokio::fs::File::create(path.join(file_name.to_string() + ".temp")).await {
            Err(e) => {
                println!("In creating {file_name} : {e}");
                return;
            }
            Ok(temp) => {
                buf = BufWriter::new(temp);
            }
        }
        let response = self.async_client.get(url).send().await;
        let mpb = m.add(ProgressBar::new(size));
        mpb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {wide_msg} [{bar:.cyan/blue}] {bytes}/{total_bytes} ({eta} {bytes_per_sec})")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));
        mpb.set_message(format!("In {:?}: {file_name}", path));
        let mut length = 0;
        match response {
            Ok(mut temp) => {
                while let chunk = temp.chunk().await {
                    match chunk {
                        Ok(chunk) => match chunk {
                            None => {
                                sleep(Duration::from_millis(1000)).await;
                                break;
                            }
                            Some(chunk) => {
                                buf.write(&chunk.slice(0..chunk.len())).await;
                                pb.inc(chunk.len() as u64);
                                mpb.inc(chunk.len() as u64);
                                length += chunk.len();
                            }
                        },
                        Err(e) => {
                            println!("{:?}", temp.headers());
                            println!("In downloading[1] {file_name} : {e}");
                        }
                    }
                }
                let temp = buf.flush().await;
                match temp {
                    Err(e) => println!("In downloading[2] {file_name} : {e}"),
                    Ok(_) => {}
                }
                // match {}
                // println!("{:?} {:?} {:?}", temp, length, file_name);
                fs::rename(
                    path.join(file_name.to_string() + ".temp"),
                    path.join(file_name.to_string()),
                );
                pb.set_message(format!("{file_name} done"));
                mpb.finish_and_clear();
            }
            Err(e) => println!("In downloading[3] {file_name} : {e}"),
        }
    }
}
