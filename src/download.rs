use crate::structs::*;
use crate::util::GetFromJson;
use indicatif::MultiProgress;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::header;
use reqwest::Response;
use serde_json::Value;
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::Semaphore;
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
            sem: Arc::new(Semaphore::new(500)),
        }
    }
    async fn get_remote_resource(&self, url: &str) -> Vec<Response> {
        let mut page_num = 1;
        let mut ans = Vec::new();
        loop {
            let response: Response;
            // permit is necessary. Otherwise, the program will be blocked by the rate limit.
            let _permit = self.sem.acquire_many(50).await.unwrap();
            let temp = self
                .async_client
                .get(url.to_string() + "&page=" + page_num.to_string().as_str())
                .send();
            match temp.await {
                Err(e) => {
                    println!("{}", t!("In getting %{url} : %{e}", url = url, e = e));
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
            if temp < 0.0 {
                println!(
                    "{}",
                    t!(
                        "In getting %{url} : rate limit exceeded wait 10s",
                        url = url
                    )
                );
                sleep(Duration::from_millis(1000 * 10)).await;
                continue;
            }

            if response.headers().get("status").unwrap().to_str().unwrap() == "401 Unauthorized" {
                return ans;
            }
            if response.headers().get("status").unwrap().to_str().unwrap() == "403 Forbidden" {
                return ans;
            }
            if response.headers().get("status").unwrap().to_str().unwrap() == "404 Not Found" {
                return ans;
            }
            if response.headers().get("status").unwrap().to_str().unwrap() != "200 OK" {
                println!("{:?} in {:?}", response.headers(), url);
                sleep(Duration::from_millis(1000)).await;
                continue;
            }
            if let Some(link) = response.headers().get("link") {
                if link.to_str().unwrap().contains("rel=\"next\"") {
                    page_num += 1;
                    ans.push(response);
                } else {
                    ans.push(response);
                    return ans;
                }
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
            let result: Value = match response.json().await.ok() {
                None => continue,
                Some(temp) => temp,
            };
            match result.as_array() {
                None => {
                    let item = T::get_from_json(&result, a.clone(), b.clone());
                    match item {
                        None => continue,
                        Some(item) => {
                            let rc = Rc::new(RefCell::new(item));
                            ans.push(rc);
                        }
                    }
                }
                Some(result) => {
                    for i in result {
                        let item = T::get_from_json(i, a.clone(), b.clone());
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
    pub async fn get_assignment_list(
        &self,
        course: Rc<RefCell<Course>>,
    ) -> Vec<Rc<RefCell<Assignment>>> {
        let url = format!(
            "{}/api/v1/courses/{}/assignments?",
            self.url,
            course.borrow().id
        );
        self.get_remote_json_list::<Rc<RefCell<Course>>, i32, Assignment>(&url, course, 1)
            .await
    }

    pub async fn get_module_list(&self, course: Rc<RefCell<Course>>) -> Vec<Rc<RefCell<Module>>> {
        let url = format!(
            "{}/api/v1/courses/{}/modules?",
            self.url,
            course.borrow().id
        );
        self.get_remote_json_list::<Rc<RefCell<Course>>, i32, Module>(&url, course, 1)
            .await
    }

    pub async fn get_item_list(&self, module: Rc<RefCell<Module>>) -> Vec<Rc<RefCell<Item>>> {
        let url = &module.borrow().itemline.clone();
        self.get_remote_json_list::<Rc<RefCell<Module>>, i32, Item>(url, module, 1)
            .await
    }
    pub async fn get_page_list(&self, item: Rc<RefCell<Item>>) -> Vec<Rc<RefCell<Page>>> {
        let url = &item.borrow().url.clone();
        self.get_remote_json_list::<Rc<RefCell<Item>>, i32, Page>(url, item, 1)
            .await
    }
    pub async fn get_folder_list(&self, course: Rc<RefCell<Course>>) -> Vec<Rc<RefCell<Folder>>> {
        let url = format!(
            "{}/api/v1/courses/{}/folders?",
            self.url,
            course.borrow().id
        );
        self.get_remote_json_list::<Rc<RefCell<Course>>, i32, Folder>(&url, course, 1)
            .await
    }

    pub async fn get_file_list_from_folder(
        &self,
        folder: Rc<RefCell<Folder>>,
        path: PathBuf,
    ) -> Vec<Rc<RefCell<CourseFile>>> {
        let url = &folder.borrow().filelink.clone();
        self.get_remote_json_list::<Rc<RefCell<Folder>>, PathBuf, CourseFile>(
            url,
            folder.clone(),
            path,
        )
        .await
    }
    pub async fn get_file_list_from_assignment(
        &self,
        assignment: Rc<RefCell<Assignment>>,
        path: PathBuf,
    ) -> Vec<Rc<RefCell<CourseFile>>> {
        let mut ans = Vec::new();
        let filelinks = assignment.borrow().filelink.clone();
        for i in filelinks {
            let url = i + "?"; // It is necessary to add a ?. Not sure why.
            let mut tempans = self
                .get_remote_json_list::<Rc<RefCell<Assignment>>, PathBuf, CourseFile>(
                    &url,
                    assignment.clone(),
                    path.clone(),
                )
                .await;
            ans.append(tempans.as_mut());
        }
        ans
    }
    pub async fn get_file_list_from_page(
        &self,
        page: Rc<RefCell<Page>>,
        path: PathBuf,
    ) -> Vec<Rc<RefCell<CourseFile>>> {
        let mut ans = Vec::new();
        let filelinks = page.borrow().filelink.clone();
        for i in filelinks {
            let url: String = i + "?"; // It is necessary to add a ?. Not sure why.
            let mut tempans = self
                .get_remote_json_list::<Rc<RefCell<Page>>, PathBuf, CourseFile>(
                    &url,
                    page.clone(),
                    path.clone(),
                )
                .await;
            ans.append(tempans.as_mut());
        }
        ans
    }
    pub async fn download_file(
        &self,
        path: &Path,
        url: &str,
        file_name: &str,
        pb: &ProgressBar,
        m: &MultiProgress,
        size: u64,
    ) {
        let mut buf: BufWriter<File>;
        let _permit = self.sem.acquire_many(50).await.unwrap();
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
        match response {
            Ok(mut temp) => {
                loop {
                    let chunk = temp.chunk().await;
                    match chunk {
                        Ok(chunk) => match chunk {
                            None => {
                                sleep(Duration::from_millis(1000)).await;
                                break;
                            }
                            Some(chunk) => {
                                let res = buf.write(&chunk.slice(0..chunk.len())).await;
                                if let Err(e) = res {
                                    println!("In writing[1] {file_name} : {e}");
                                    return;
                                }
                                pb.inc(chunk.len() as u64);
                                mpb.inc(chunk.len() as u64);
                            }
                        },
                        Err(e) => {
                            println!("{:?}", temp.headers());
                            println!("In downloading[1] {file_name} : {e}");
                        }
                    }
                }
                let temp = buf.flush().await;
                if let Err(e) = temp {
                    println!("In downloading[2] {file_name} : {e}")
                }
                let res = fs::rename(
                    path.join(file_name.to_string() + ".temp"),
                    path.join(file_name),
                );
                if let Err(e) = res {
                    println!("In downloading[3] {file_name} : {e}")
                }
                pb.set_message(format!("{file_name} done"));
                mpb.finish_and_clear();
            }
            Err(e) => println!("In downloading[4] {file_name} : {e}"),
        }
    }
}
