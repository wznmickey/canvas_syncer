use crate::assignment::Assignment;
use crate::folder::Folder;
use crate::page::Page;
use crate::util::GetFromJson;
use chrono::DateTime;
use chrono::Utc;
use serde_json::Value;
use std::cell::RefCell;
use std::cmp::max;
use std::rc::Rc;
use std::{fs, path::PathBuf};
pub enum FileStatus {
    Latest,
    NeedUpdate,
    NotExist,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct CourseFile {
    id: i64,
    pub display_name: String,
    pub filename: String,
    pub url: String,
    // pub folder: Rc<RefCell<Folder>>,
    pub content_type: String,
    pub size: u64,
    pub created_time: DateTime<Utc>,
    pub updated_time: DateTime<Utc>,
    pub modified_time: DateTime<Utc>,
    pub my_parent_path: PathBuf,
}
impl GetFromJson<CourseFile, Rc<RefCell<Folder>>, PathBuf> for CourseFile {
    fn get_from_json(x: &Value, f: Rc<RefCell<Folder>>, path: PathBuf) -> Option<CourseFile> {
        let temp = x["display_name"].as_str()?.to_string();
        if x["locked"].as_bool()? || x["locked_for_user"].as_bool()? {
            println!(
                "{}",
                t!(
                    "File %{name} is locked, skip it",
                    name = x["display_name"].as_str()?
                )
            );
            return None;
        }
        Some(CourseFile {
            id: x["id"].as_i64()?,
            my_parent_path: {
                path.join(&f.borrow().course.borrow().name)
                    .join(&f.borrow().fullname)
            },
            display_name: temp,
            filename: x["filename"].as_str()?.to_string(),
            // folder: f.clone(),
            url: x["url"].as_str()?.to_string(),
            content_type: x["content-type"].as_str()?.to_string(),
            size: x["size"].as_u64()?,
            created_time: DateTime::parse_from_str(x["created_at"].as_str()?, "%+")
                .ok()?
                .to_utc(),
            updated_time: DateTime::parse_from_str(x["updated_at"].as_str()?, "%+")
                .ok()?
                .to_utc(),
            modified_time: DateTime::parse_from_str(x["modified_at"].as_str()?, "%+")
                .ok()?
                .to_utc(),
        })
    }
}

impl GetFromJson<CourseFile, Rc<RefCell<Assignment>>, PathBuf> for CourseFile {
    fn get_from_json(x: &Value, f: Rc<RefCell<Assignment>>, path: PathBuf) -> Option<CourseFile> {
        let temp = x["display_name"].as_str()?.to_string();
        if x["locked"].as_bool()? || x["locked_for_user"].as_bool()? {
            println!(
                "{}",
                t!(
                    "File %{name} is locked, skip it",
                    name = x["display_name"].as_str()?
                )
            );
            return None;
        }
        Some(CourseFile {
            id: x["id"].as_i64()?,
            my_parent_path: {
                path.join(&f.borrow().course.borrow().name)
                    .join(&f.borrow().name)
            },
            display_name: temp,
            filename: x["filename"].as_str()?.to_string(),
            // folder: None,
            url: x["url"].as_str()?.to_string(),
            content_type: x["content-type"].as_str()?.to_string(),
            size: x["size"].as_u64()?,
            created_time: DateTime::parse_from_str(x["created_at"].as_str()?, "%+")
                .ok()?
                .to_utc(),
            updated_time: DateTime::parse_from_str(x["updated_at"].as_str()?, "%+")
                .ok()?
                .to_utc(),
            modified_time: DateTime::parse_from_str(x["modified_at"].as_str()?, "%+")
                .ok()?
                .to_utc(),
        })
    }
}

impl GetFromJson<CourseFile, Rc<RefCell<Page>>, PathBuf> for CourseFile {
    fn get_from_json(x: &Value, f: Rc<RefCell<Page>>, path: PathBuf) -> Option<CourseFile> {
        let temp = x["display_name"].as_str()?.to_string();
        if x["locked"].as_bool()? || x["locked_for_user"].as_bool()? {
            println!(
                "{}",
                t!(
                    "File %{name} is locked, skip it",
                    name = x["display_name"].as_str()?
                )
            );
            return None;
        }
        Some(CourseFile {
            id: x["id"].as_i64()?,
            my_parent_path: {
                path.join(
                    &f.borrow()
                        .item
                        .borrow()
                        .module
                        .borrow()
                        .course
                        .borrow()
                        .name,
                )
                .join(&f.borrow().item.borrow().module.borrow().name)
                .join(&f.borrow().item.borrow().name)
                .join(&f.borrow().name)
            },
            display_name: temp,
            filename: x["filename"].as_str()?.to_string(),
            // folder: None,
            url: x["url"].as_str()?.to_string(),
            content_type: x["content-type"].as_str()?.to_string(),
            size: x["size"].as_u64()?,
            created_time: DateTime::parse_from_str(x["created_at"].as_str()?, "%+")
                .ok()?
                .to_utc(),
            updated_time: DateTime::parse_from_str(x["updated_at"].as_str()?, "%+")
                .ok()?
                .to_utc(),
            modified_time: DateTime::parse_from_str(x["modified_at"].as_str()?, "%+")
                .ok()?
                .to_utc(),
        })
    }
}

impl CourseFile {
    pub fn get_status(&self) -> FileStatus {
        let x: Result<fs::Metadata, std::io::Error> =
            fs::metadata(self.my_parent_path.join(&self.display_name));
        match x {
            Ok(y) => {
                let mod_time = chrono::DateTime::<Utc>::from(y.modified().unwrap());
                let create_time = chrono::DateTime::<Utc>::from(y.created().unwrap());
                let newest_local_time = if mod_time > create_time {
                    mod_time
                } else {
                    create_time
                };
                let newest_remote_time = max(
                    max(self.created_time, self.modified_time),
                    self.updated_time,
                );
                if newest_local_time < newest_remote_time {
                    return FileStatus::NeedUpdate;
                }
                let local_size = y.len();
                let remote_size = self.size;
                if local_size != remote_size {
                    return FileStatus::NeedUpdate;
                }
                FileStatus::Latest
            }
            Err(_) => FileStatus::NotExist,
        }
    }
}
