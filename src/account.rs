use crate::config::*;
use crate::course::*;
use crate::download::*;
use std::fs::*;
use std::rc::Rc;

use indicatif::ProgressBar;
pub struct Account {
    config: Config,
    remote_data: RemoteData,
    course: Vec<Rc<Course>>,
    folders: Vec<Rc<Folder>>,
    files: Vec<Rc<crate::course::File>>,
    need_update_files: Vec<Rc<crate::course::File>>,
    need_download_files: Vec<Rc<crate::course::File>>,
}

impl Account {
    pub fn new(st: &str) -> Self {
        let config = Config::read_file(st);
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
        let mut update_size = 0;
        let mut download_size = 0;
        for file in &self.files {
            let temp = file.get_status();
            match temp {
                FileStatus::NeedUpdate => {
                    self.need_update_files.push(Rc::clone(file));
                    update_size += file.size;
                }
                FileStatus::NotExist => {
                    self.need_download_files.push(Rc::clone(file));
                    download_size += file.size;
                }
                FileStatus::Latest => {}
            }
        }
        println!("update size: {}", update_size);
        for file in &self.need_update_files {
            println!("{:?}", file.my_full_path);
        }
        println!("download size: {}", download_size);
        for file in &self.need_download_files {
            println!("{:?}", file.my_full_path);
        }
    }
    pub fn download_one_file(&self) -> () {
        let temp = self.need_download_files.get(0).unwrap();
        println!("download 1: {:?}", temp.my_full_path);
        self.remote_data
            .download_file(&temp.my_full_path, temp.url.as_str());
    }
    pub fn download_files(&self) -> () {
        let pb = ProgressBar::new(self.need_download_files.len() as u64);
        for file in &self.need_download_files {
            println!("start downloading : {:?}", file.my_full_path);
            self.remote_data
                .download_file(&file.my_full_path, file.url.as_str());
            println!("finished: {:?}", file.my_full_path);
            pb.inc(1);
        }
        pb.finish_with_message("done");
    }
}
