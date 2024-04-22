use crate::config::*;
use crate::course::*;
use crate::download::*;
use dialoguer::{theme::ColorfulTheme, Confirm};
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::fmt::Write;
use std::fs;
use std::fs::*;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
pub struct Account {
    config: Config,
    remote_data: RemoteData,
    course: Vec<Rc<Course>>,
    folders: Vec<Rc<Folder>>,
    files: Vec<Rc<crate::course::File>>,
    need_update_files: Vec<Rc<crate::course::File>>,
    need_download_files: Vec<Rc<crate::course::File>>,
    download_size: u64,
    update_size: u64,
}

impl Account {
    pub fn new(st: &str) -> Self {
        let config = Config::read_file(st);
        config.print();
        let remote_data = RemoteData::new(&config.key, &config.canvas_url);
        let course: Vec<Rc<Course>> = remote_data.get_course_list();
        Account {
            config,
            remote_data,
            course,
            folders: Vec::new(),
            files: Vec::new(),
            need_download_files: Vec::new(),
            need_update_files: Vec::new(),
            download_size: 0,
            update_size: 0,
        }
    }
    pub fn get_folders(&mut self) -> () {
        let pb = ProgressBar::new(self.course.len() as u64);
        for course in &self.course {
            let mut ans = self.remote_data.get_folder_list(Rc::clone(&course));
            self.folders.append(&mut ans);
            pb.inc(1);
        }
        pb.finish_with_message("done");
    }
    pub fn create_folders(&self) -> () {
        let pb = ProgressBar::new(self.folders.len() as u64);
        for folder in &self.folders {
            create_dir_all(
                self.config.local_place.clone()
                    + "/"
                    + &folder.course.name
                    + " "
                    + &folder.fullname,
            )
            .unwrap();
            pb.inc(1);
        }
        pb.finish_with_message("done");
    }
    pub fn get_files(&mut self) -> () {
        let pb = ProgressBar::new(self.folders.len() as u64);
        for folder in &self.folders {
            let mut ans = self
                .remote_data
                .get_file_list(Rc::clone(&folder), self.config.local_place.clone().into());
            self.files.append(&mut ans);
            pb.inc(1);
        }
        pb.finish_with_message("done");
    }
    pub fn calculate_files(&mut self) -> () {
        for file in &self.files {
            let temp = file.get_status();
            match temp {
                FileStatus::NeedUpdate => {
                    self.need_update_files.push(Rc::clone(file));
                    self.update_size += file.size;
                }
                FileStatus::NotExist => {
                    self.need_download_files.push(Rc::clone(file));
                    self.download_size += file.size;
                }
                FileStatus::Latest => {}
            }
        }
    }
    pub fn download_files(&self) -> () {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let msg = rt.block_on(self.download_files_help());
    }
    pub async fn download_files_help(&self) -> () {
        for file in &self.need_download_files {
            println!("{:?}", file.my_full_path);
        }
        println!("download size: {}", self.download_size);
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to download?")
            .interact()
            .unwrap()
        {
            println!("Download files...");
            let pb = ProgressBar::new(self.download_size);
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));
            for file in &self.need_download_files {
                // println!("start downloading : {:?}", file.my_full_path);
                self.remote_data
                    .download_file(&file.my_full_path, file.url.as_str())
                    .await;
                // println!("finished: {:?}", file.my_full_path);
                pb.inc(file.size);
            }
            pb.finish_with_message("done");
        } else {
            println!("Do not download");
        }
    }
    pub fn update_files(&self) -> () {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let msg = rt.block_on(self.update_files_help());
    }
    pub async fn update_files_help(&self) -> () {
        for file in &self.need_update_files {
            println!("{:?}", file.my_full_path);
        }
        println!("update size: {}", self.update_size);
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to update?")
            .interact()
            .unwrap()
        {
            println!("Update files...");
            let pb = ProgressBar::new(self.update_size);
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));
            let result = join_all(self.need_update_files.iter().map(|file| async {
                // println!("start updating : {:?}", file.my_full_path);
                let x = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string();
                let mut my_full_path_old = file.my_full_path.clone();
                my_full_path_old.pop();
                my_full_path_old
                    .push(x + "_" + file.my_full_path.file_name().unwrap().to_str().unwrap());
                fs::copy(&file.my_full_path, my_full_path_old);
                self.remote_data
                    .download_file(&file.my_full_path, file.url.as_str())
                    .await;
                // println!("finished: {:?}", file.my_full_path);
                pb.inc(file.size);
            }));
            // for file in &self.need_update_files {

            // }
            pb.finish_with_message("done");
        } else {
            println!("Do not update");
        }
    }
}
