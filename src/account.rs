use crate::config::*;
use crate::course::*;
use crate::download::*;
use dialoguer::{theme::ColorfulTheme, Confirm};
use futures::future::join_all;
use std::cell::RefCell;
use std::fmt::Write;
use std::fs;
use std::fs::*;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use indicatif::{MultiProgress, ProgressBar, ProgressState, ProgressStyle};
pub struct Account {
    config: Config,
    remote_data: RemoteData,
    course: Vec<Rc<RefCell<Course>>>,
    folders: Vec<Rc<RefCell<Folder>>>,
    files: Vec<Rc<RefCell<CourseFile>>>,
    need_update_files: Vec<Rc<RefCell<CourseFile>>>,
    need_download_files: Vec<Rc<RefCell<CourseFile>>>,
    download_size: u64,
    update_size: u64,
}

impl Account {
    pub fn new(c: Config) -> Self {
        let config: Config = c;
        config.print();
        let remote_data = RemoteData::new(&config.key, &config.canvas_url);
        let course: Vec<Rc<RefCell<Course>>> = remote_data.get_course_list();
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
        let rt = tokio::runtime::Runtime::new().unwrap();
        self.folders = rt.block_on(self.get_folders_helper());
    }

    pub async fn get_folders_helper(&self) -> Vec<Rc<RefCell<Folder>>> {
        let pb = &ProgressBar::new(self.course.len() as u64);
        let result = join_all(self.course.iter().map(|course| async move {
            let temp = self.remote_data.get_folder_list(Rc::clone(&course)).await;
            pb.inc(1);
            temp
        }))
        .await
        .into_iter()
        .flatten()
        .collect();
        pb.finish_with_message("done");
        result
    }
    pub fn create_folders(&self) -> () {
        let pb = ProgressBar::new(self.folders.len() as u64);
        for folder in &self.folders {
            let folder = folder.borrow();
            if self.config.allow_term {
                create_dir_all(
                    self.config.local_place.clone()
                        + "/"
                        + &folder.course.borrow().term_name
                        + "/"
                        + &folder.course.borrow().name
                        + " "
                        + &folder.fullname,
                )
                .unwrap();
            } else {
                create_dir_all(
                    self.config.local_place.clone()
                        + "/"
                        + &folder.course.borrow().name
                        + " "
                        + &folder.fullname,
                )
                .unwrap();
            }
            pb.inc(1);
        }
        pb.finish_with_message("done");
    }

    pub fn get_files(&mut self) -> () {
        let rt = tokio::runtime::Runtime::new().unwrap();
        self.files = rt.block_on(self.get_files_helper());
    }

    async fn get_files_helper(&self) -> Vec<Rc<RefCell<CourseFile>>> {
        let pb = &ProgressBar::new(self.folders.len() as u64);
        let result = join_all(self.folders.iter().map(|folder| async move {
            let path = if !self.config.allow_term {
                self.config.local_place.clone()
            } else {
                self.config.local_place.clone()
                    + "/"
                    + folder.borrow().course.borrow().term_name.as_str()
            };
            let temp = self
                .remote_data
                .get_file_list(Rc::clone(&folder), (path).into())
                .await;
            pb.inc(1);
            temp
        }))
        .await
        .into_iter()
        .flatten()
        .collect();
        pb.finish_with_message("done");
        result
    }
    pub fn calculate_files(&mut self) -> () {
        for file in &self.files {
            let temp = file.borrow().get_status();
            match temp {
                FileStatus::NeedUpdate => {
                    self.need_update_files.push(Rc::clone(file));
                    self.update_size += file.borrow().size;
                }
                FileStatus::NotExist => {
                    self.need_download_files.push(Rc::clone(file));
                    self.download_size += file.borrow().size;
                }
                FileStatus::Latest => {}
            }
        }
    }
    pub fn download_files(&self) -> () {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let msg = rt.block_on(self.download_files_helper());
    }
    async fn download_files_helper(&self) -> () {
        // for file in &self.need_download_files {
        //     println!("In {:?}: {}", file.my_parent_path, file.display_name);
        // }
        if self.need_download_files.len() == 0 {
            println!("No files need to download");
            return;
        }
        println!(
            "download size: {} MiB",
            self.download_size as f64 / 1024.0 / 1024.0
        );
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to download?")
            .interact()
            .unwrap()
        {
            let m = MultiProgress::new();
            let pb = m.add(ProgressBar::new(self.download_size));
            println!("Download files...");
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta} {bytes_per_sec})")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));

            let result = join_all(self.need_download_files.iter().map(|file| async {
                // for file in &self.need_download_files {
                // println!("start downloading : {:?}", file.my_full_path);
                self.remote_data
                    .download_file(
                        &file.borrow().my_parent_path,
                        file.borrow().url.as_str(),
                        &file.borrow().display_name,
                        &pb,
                        &m,
                        file.borrow().size,
                    )
                    .await;
                // println!("finished: {:?}", file.my_full_path);
            }));
            result.await;
            pb.finish_with_message("All downloaded");
        } else {
            println!("Do not download");
        }
    }
    pub fn update_files(&self) -> () {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.update_files_helper())
    }
    async fn update_files_helper(&self) -> () {
        // for file in &self.need_update_files {
        //     println!("In {:?}: {}", file.my_parent_path, file.display_name);
        // }
        if self.need_update_files.len() == 0 {
            println!("No files need to update");
            return;
        }
        println!(
            "update size: {} MiB",
            self.update_size as f64 / 1024.0 / 1024.0
        );
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to update?")
            .interact()
            .unwrap()
        {
            println!("Update files...");
            let m = MultiProgress::new();
            let pb = m.add(ProgressBar::new(self.update_size));
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta} {bytes_per_sec})")
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

                let my_full_path_old = file
                    .borrow()
                    .my_parent_path
                    .join(x + "_" + file.borrow().display_name.as_str());
                let my_full_path_new = file
                    .borrow()
                    .my_parent_path
                    .join(&file.borrow().display_name);
                fs::copy(my_full_path_new, my_full_path_old);

                self.remote_data
                    .download_file(
                        &file.borrow().my_parent_path,
                        file.borrow().url.as_str(),
                        &file.borrow().display_name,
                        &pb,
                        &m,
                        file.borrow().size,
                    )
                    .await;
                // println!("finished: {:?}", file.my_full_path);
            }));
            result.await;
            pb.finish_with_message("all updated");
        } else {
            println!("Do not update");
        }
    }
}
