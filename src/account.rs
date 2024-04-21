use crate::config::*;
use crate::course::*;
use crate::download::*;
use std::fs::*;
use std::rc::Rc;
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
        let course = remote_data.get_course_list();
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
        for course in &self.course {
            let mut ans = self.remote_data.get_folder_list(Rc::clone(&course));
            self.folders.append(&mut ans);
        }
    }
    pub fn create_folders(&self) -> () {
        for folder in &self.folders {
            create_dir_all(
                self.config.local_place.clone()
                    + "/"
                    + &folder.course.name
                    + " "
                    + &folder.fullname,
            )
            .unwrap();
        }
    }
    pub fn get_files(&mut self) -> () {
        for folder in &self.folders {
            let mut ans = self
                .remote_data
                .get_file_list(Rc::clone(&folder), self.config.local_place.clone().into());
            self.files.append(&mut ans);
        }
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
}
