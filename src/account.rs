use crate::config::*;
use crate::course::*;
use crate::download::*;
use std::fs::*;
use std::rc::Rc;
pub struct Account {
    config: Config,
    remote_data: RemoteData,
    course: Vec<Rc<Course>>,
    folders: Vec<Folder>,
}

impl Account{
    pub fn new(st: &str) -> Self {
        let config = Config::read_file(st);
        let remote_data = RemoteData::new(&config.key, &config.canvas_url);
        let course = remote_data.get_course_list();
        Account{
            config,
            remote_data,
            course,
            folders: Vec::new(),
        }
    }
    pub fn get_folders (& mut self) -> () {
        for course in &self.course {
            let mut ans = self.remote_data.get_folder_list(Rc::clone(&course));
            self.folders.append(&mut ans);
        }
    }
    pub fn create_folders(&self) -> () {
        for folder in &self.folders {
            create_dir_all(self.config.local_place.clone() + "/" +&folder.course.name + " "+ &folder.fullname).unwrap();
        }
    }
}
