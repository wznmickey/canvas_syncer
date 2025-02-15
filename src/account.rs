use crate::config::*;
use crate::download::*;
use crate::filter::object_filter_check;
use crate::structs::*;
use crate::ARGS;
use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressState, ProgressStyle};
use inquire::Confirm;
use logger_rust_i18n::*;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::Write;
use std::fs;
use std::fs::*;
use std::path::Path;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
pub struct Account {
    config: Config,
    remote_data: RemoteData,
    course: Vec<Rc<RefCell<Course>>>,
    folders: Vec<Rc<RefCell<Folder>>>,
    assignmnets: Vec<Rc<RefCell<Assignment>>>,
    modules: Vec<Rc<RefCell<Module>>>,
    items: Vec<Rc<RefCell<Item>>>,
    pages: Vec<Rc<RefCell<Page>>>,
    files: Vec<Rc<RefCell<CourseFile>>>,
    need_update_files: Vec<Rc<RefCell<CourseFile>>>,
    need_download_files: Vec<Rc<RefCell<CourseFile>>>,
    download_size: u64,
    update_size: u64,
    pub multi_progress_bar: MultiProgress,
    pub progress_bar: ProgressBar,
}
fn specia_char_replace(st: String) -> String {
    st.replace("/", "_")
        .replace("\\", "_")
        .replace(":", "_")
        .replace("?", "_")
        .replace("*", "_")
        .replace("<", "_")
        .replace(">", "_")
        .replace("|", "_")
        .trim_end_matches([' ', '.'])
        .to_string()
}
impl Account {
    pub fn new(c: Config) -> Self {
        let config: Config = c;
        config.print();
        let m = MultiProgress::new();
        let pb = m.add(ProgressBar::new(9));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.blue}  [{wide_bar:.green/green}] {pos}/{len}")
                .unwrap()
                .progress_chars("#>-"),
        );
        let remote_data = RemoteData::new(&config.key, &config.canvas_url);

        info!("Get courses list");

        let mut course: Vec<Rc<RefCell<Course>>> = remote_data.get_course_list();

        if let Some(z) = &config
            .filters
            .as_ref()
            .and_then(|x| x.term_filter.as_ref())
            .and_then(|y| y.object_filter.as_ref())
        {
            course = course
                .iter()
                .filter(|&x| object_filter_check(z, x.borrow().term_id, &x.borrow().term_name))
                .map(Rc::clone)
                .collect();
        }
        if let Some(z) = &config
            .filters
            .as_ref()
            .and_then(|x| x.course_filter.as_ref())
            .and_then(|y| y.object_filter.as_ref())
        {
            course = course
                .iter()
                .filter(|x| object_filter_check(z, x.borrow().id, &x.borrow().name))
                .map(Rc::clone)
                .collect();
        }
        course.iter().for_each(|x| {
            let temp = specia_char_replace(x.borrow().term_name.clone());
            x.borrow_mut().term_name = temp;
            let temp = specia_char_replace(x.borrow().name.clone());
            x.borrow_mut().name = temp
        });

        Account {
            config,
            remote_data,
            course,
            folders: Vec::new(),
            files: Vec::new(),
            assignmnets: Vec::new(),
            modules: Vec::new(),
            items: Vec::new(),
            pages: Vec::new(),
            need_download_files: Vec::new(),
            need_update_files: Vec::new(),
            download_size: 0,
            update_size: 0,
            progress_bar: pb,
            multi_progress_bar: m,
        }
    }

    pub fn run(&mut self) {
        self.get_folders();
        for folder in &self.folders {
            let temp = specia_char_replace(folder.borrow().fullname.clone());
            folder.borrow_mut().fullname = temp;
            let temp = specia_char_replace(folder.borrow().name.clone());
            folder.borrow_mut().name = temp;
        }
        debug!("{:?}", self.folders);
        self.get_assignments();
        for assignment in &self.assignmnets {
            let temp = specia_char_replace(assignment.borrow().name.clone());
            assignment.borrow_mut().name = temp;
        }
        debug!("{:?}", self.assignmnets);
        self.get_modules();
        for module in &self.modules {
            let temp = specia_char_replace(module.borrow().name.clone());
            module.borrow_mut().name = temp;
        }
        debug!("{:?}", self.modules);
        self.get_items();
        for item in &self.items {
            let temp = specia_char_replace(item.borrow().name.clone());
            item.borrow_mut().name = temp;
        }
        debug!("{:?}", self.items);
        self.get_pages();
        for page in &self.pages {
            let temp = specia_char_replace(page.borrow().name.clone());
            page.borrow_mut().name = temp;
        }
        debug!("{:?}", self.pages);
        self.create_folders();
        self.create_assignments();
        self.create_pages();
        self.get_files();
        debug!("{:?}", self.files);
        self.progress_bar.finish();
        self.calculate_files();
        self.download_files();
        self.update_files();
    }
    fn get_bar(&self, num: u64, msg: Cow<'_, str>) -> ProgressBar {
        let pb = self
            .multi_progress_bar
            .add(ProgressBar::new(num))
            .with_message(msg.into_owned());
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{msg}] [{bar:.cyan/blue}] {pos}/{len}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb
    }
    fn get_assignments(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        self.assignmnets = rt.block_on(self.get_assignments_helper());
        self.progress_bar.inc(1);
    }
    async fn get_assignments_helper(&self) -> Vec<Rc<RefCell<Assignment>>> {
        let pb = &self.get_bar(self.course.len() as u64, t!("Get assignments list"));
        let result = join_all(self.course.iter().map(|course| async move {
            let temp = self
                .remote_data
                .get_assignment_list(Rc::clone(course))
                .await;
            pb.inc(1);
            temp
        }))
        .await
        .into_iter()
        .flatten()
        .collect();
        pb.finish();
        result
    }

    fn get_modules(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        self.modules = rt.block_on(self.get_modules_helper());
        self.progress_bar.inc(1);
    }

    async fn get_modules_helper(&self) -> Vec<Rc<RefCell<Module>>> {
        let pb = &self.get_bar(self.course.len() as u64, t!("Get modules list"));
        let result = join_all(self.course.iter().map(|course| async move {
            let temp = self.remote_data.get_module_list(Rc::clone(course)).await;
            pb.inc(1);
            temp
        }))
        .await
        .into_iter()
        .flatten()
        .collect();
        pb.finish();
        result
    }

    fn get_items(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        self.items = rt.block_on(self.get_items_helper());
        self.progress_bar.inc(1);
    }

    async fn get_items_helper(&self) -> Vec<Rc<RefCell<Item>>> {
        let pb = &self.get_bar(self.modules.len() as u64, t!("Get items list"));
        let result = join_all(self.modules.iter().map(|modules| async move {
            let temp = self.remote_data.get_item_list(Rc::clone(modules)).await;
            pb.inc(1);
            temp
        }))
        .await
        .into_iter()
        .flatten()
        .collect();
        pb.finish();
        result
    }

    fn get_pages(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        self.pages = rt.block_on(self.get_pages_helper());
        self.progress_bar.inc(1);
    }

    async fn get_pages_helper(&self) -> Vec<Rc<RefCell<Page>>> {
        let pb = &self.get_bar(self.items.len() as u64, t!("Get pages list"));
        let result = join_all(self.items.iter().map(|item| async move {
            let temp = self.remote_data.get_page_list(Rc::clone(item)).await;
            pb.inc(1);
            temp
        }))
        .await
        .into_iter()
        .flatten()
        .collect();
        pb.finish();
        result
    }
    fn get_folders(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        self.folders = rt.block_on(self.get_folders_helper());
        self.progress_bar.inc(1);
    }

    async fn get_folders_helper(&self) -> Vec<Rc<RefCell<Folder>>> {
        let pb = &self.get_bar(self.course.len() as u64, t!("Get course file folders list"));
        let result = join_all(self.course.iter().map(|course| async move {
            let temp = self.remote_data.get_folder_list(Rc::clone(course)).await;
            pb.inc(1);
            temp
        }))
        .await
        .into_iter()
        .flatten()
        .collect();
        pb.finish();
        result
    }
    fn create_folders(&self) {
        let pb = &self.get_bar(self.folders.len() as u64, t!("Create file folders"));
        for folder in &self.folders {
            let folder = folder.borrow();
            debug!("{:?} creating", folder);
            if self.config.allow_term {
                create_dir_all(
                    Path::new(&self.config.local_place)
                        .join(&folder.course.borrow().term_name)
                        .join(folder.course.borrow().name.clone())
                        .join(&folder.fullname),
                )
                .unwrap();
            } else {
                create_dir_all(
                    Path::new(&self.config.local_place)
                        .join(folder.course.borrow().name.clone())
                        .join(&folder.fullname),
                )
                .unwrap();
            }
            debug!("{:?} created", folder);
            pb.inc(1);
        }
        pb.finish();
        self.progress_bar.inc(1);
    }
    fn create_assignments(&self) {
        let pb = &self.get_bar(
            self.assignmnets.len() as u64,
            t!("Create assignment folders"),
        );
        for assignment in &self.assignmnets {
            let assignment = assignment.borrow();
            debug!("{:?} creating", assignment);
            if self.config.allow_term {
                create_dir_all(
                    Path::new(&self.config.local_place)
                        .join(&assignment.course.borrow().term_name)
                        .join(assignment.course.borrow().name.clone())
                        .join(&assignment.name),
                )
                .unwrap();
            } else {
                create_dir_all(
                    Path::new(&self.config.local_place)
                        .join(assignment.course.borrow().name.clone())
                        .join(&assignment.name),
                )
                .unwrap();
            }
            debug!("{:?} created", assignment);
            pb.inc(1);
        }
        pb.finish();
        self.progress_bar.inc(1);
    }
    fn create_pages(&self) {
        let pb = &self.get_bar(self.pages.len() as u64, t!("Create page folders"));
        for page in &self.pages {
            let page = page.borrow();
            debug!("{:?} creating", page);
            if self.config.allow_term {
                create_dir_all(
                    Path::new(&self.config.local_place)
                        .join(&page.item.borrow().module.borrow().course.borrow().term_name)
                        .join(
                            page.item
                                .borrow()
                                .module
                                .borrow()
                                .course
                                .borrow()
                                .name
                                .clone(),
                        )
                        .join(&page.item.borrow().module.borrow().name)
                        .join(&page.item.borrow().name)
                        .join(&page.name),
                )
                .unwrap();
            } else {
                create_dir_all(
                    Path::new(&self.config.local_place)
                        .join(
                            page.item
                                .borrow()
                                .module
                                .borrow()
                                .course
                                .borrow()
                                .name
                                .clone(),
                        )
                        .join(&page.item.borrow().module.borrow().name)
                        .join(&page.item.borrow().name)
                        .join(&page.name),
                )
                .unwrap();
            }
            debug!("{:?} created", page);
            pb.inc(1);
        }
        pb.finish();
        self.progress_bar.inc(1);
    }
    fn get_files(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        self.files = rt.block_on(self.get_files_helper());
        self.progress_bar.inc(1);
    }

    async fn get_files_helper(&self) -> Vec<Rc<RefCell<CourseFile>>> {
        let pb = &self.get_bar(
            self.folders.len() as u64 + self.assignmnets.len() as u64 + self.pages.len() as u64,
            t!("Get files list"),
        );
        let mut result_a: Vec<Rc<RefCell<CourseFile>>> =
            join_all(self.folders.iter().map(|folder| async move {
                let path = if !self.config.allow_term {
                    self.config.local_place.clone()
                } else {
                    self.config.local_place.clone()
                        + "/"
                        + folder.borrow().course.borrow().term_name.as_str()
                };
                debug!("getting files in folder: {:?}", folder);
                let temp = self
                    .remote_data
                    .get_file_list_from_folder(Rc::clone(folder), (path).into())
                    .await;
                pb.inc(1);
                debug!("got files in folder: {:?}", folder);
                temp
            }))
            .await
            .into_iter()
            .flatten()
            .collect();
        let mut result_b: Vec<Rc<RefCell<CourseFile>>> =
            join_all(self.assignmnets.iter().map(|assignment| async move {
                let path = if !self.config.allow_term {
                    self.config.local_place.clone()
                } else {
                    self.config.local_place.clone()
                        + "/"
                        + assignment.borrow().course.borrow().term_name.as_str()
                };
                let temp = self
                    .remote_data
                    .get_file_list_from_assignment(Rc::clone(assignment), (path).into())
                    .await;
                pb.inc(1);
                temp
            }))
            .await
            .into_iter()
            .flatten()
            .collect();
        let mut result_c: Vec<Rc<RefCell<CourseFile>>> =
            join_all(self.pages.iter().map(|page| async move {
                let path = if !self.config.allow_term {
                    self.config.local_place.clone()
                } else {
                    self.config.local_place.clone()
                        + "/"
                        + page
                            .borrow()
                            .item
                            .borrow()
                            .module
                            .borrow()
                            .course
                            .borrow()
                            .term_name
                            .as_str()
                };
                let temp = self
                    .remote_data
                    .get_file_list_from_page(Rc::clone(page), (path).into())
                    .await;
                pb.inc(1);
                temp
            }))
            .await
            .into_iter()
            .flatten()
            .collect();
        pb.finish();
        result_a.append(result_b.as_mut());
        result_a.append(result_c.as_mut());
        result_a
    }
    fn calculate_files(&mut self) {
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
    fn download_files(&self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.download_files_helper());
    }
    async fn download_files_helper(&self) {
        if self.need_download_files.is_empty() {
            info!("No files need to download");
            return;
        }
        if ARGS.get().unwrap().yes
            || Confirm::new(&prompt!(
                "Do you want to download: {} MiB ?",
                self.download_size as f64 / 1024.0 / 1024.0
            ))
            .with_default(false)
            .prompt()
            .unwrap()
        {
            let m = MultiProgress::new();
            let pb = m.add(ProgressBar::new(self.download_size));
            info!("Download files...");
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta} {bytes_per_sec})")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));

            let result = join_all(self.need_download_files.iter().map(|file| async {
                let my_parent_path = &file.borrow().my_parent_path.clone();
                let url = &file.borrow().url.clone();
                let display_name = &file.borrow().display_name.clone();
                let size = file.borrow().size;
                debug!("Downloading: {:?}", file.borrow());
                self.remote_data
                    .download_file(my_parent_path, url, display_name, &pb, &m, size)
                    .await;
                debug!("Downloaded: {:?}", file.borrow());
            }));
            result.await;
            pb.finish_with_message("All downloaded");
        } else {
            info!("Do not download");
        }
    }
    fn update_files(&self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.update_files_helper())
    }
    async fn update_files_helper(&self) {
        if self.need_update_files.is_empty() {
            info!("No files need to update");
            return;
        }
        if ARGS.get().unwrap().yes
            || Confirm::new(&prompt!(
                "Do you want to update: {} MiB?",
                self.update_size as f64 / 1024.0 / 1024.0
            ))
            .with_default(false)
            .prompt()
            .unwrap()
        {
            info!("Update files...");
            let m = MultiProgress::new();
            let pb = m.add(ProgressBar::new(self.update_size));
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta} {bytes_per_sec})")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));

            let result = join_all(self.need_update_files.iter().map(|file| async {
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
                let res = fs::copy(my_full_path_new, my_full_path_old);
                match res {
                    Ok(_) => {}
                    Err(e) => {
                        error!("In updating[1] : {e}");
                        return;
                    }
                }
                let my_parent_path = file.borrow().my_parent_path.clone();
                let url = file.borrow().url.clone();
                let display_name = file.borrow().display_name.clone();
                let size = file.borrow().size;
                debug!("Updating: {:?}", file.borrow());
                self.remote_data
                    .download_file(&my_parent_path, &url, &display_name, &pb, &m, size)
                    .await;
                debug!("Updated: {:?}", file.borrow());
            }));
            result.await;
            pb.finish_with_message("all updated");
        } else {
            info!("Do not update");
        }
    }
}
