use crate::util::get_file_link_from_html;
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

pub trait GetFromJson<T, A, B> {
    fn get_from_json(x: &Value, a: A, b: B) -> Option<T>;
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Course {
    pub id: i64,
    pub name: String,
    pub course_code: String,
    pub term_id: i64,
    pub term_name: String,
}
impl GetFromJson<Course, i32, i32> for Course {
    fn get_from_json(x: &Value, _: i32, _: i32) -> Option<Course> {
        Some(Course {
            id: x["id"].as_i64()?,
            name: x["name"].as_str()?.to_string(),
            course_code: x["course_code"].as_str()?.to_string(),
            term_id: x["enrollment_term_id"].as_i64()?,
            term_name: x["term"]["name"].as_str()?.to_string(),
        })
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Folder {
    id: i64,
    pub name: String,
    pub fullname: String,
    pub course: Rc<RefCell<Course>>,
    pub filelink: String,
}
impl GetFromJson<Folder, Rc<RefCell<Course>>, i32> for Folder {
    fn get_from_json(x: &Value, c: Rc<RefCell<Course>>, _: i32) -> Option<Folder> {
        if x["locked"].as_bool()? || x["locked_for_user"].as_bool()? {
            println!(
                "{}",
                t!(
                    "Folder %{name} is locked, skip it",
                    name = x["display_name"].as_str()?
                )
            );
            return None;
        }
        Some(Folder {
            id: x["id"].as_i64()?,
            name: x["name"].as_str()?.to_string(),
            fullname: x["full_name"].as_str()?.to_string(),
            course: c,
            filelink: x["files_url"].as_str()?.to_string() + "?",
        })
    }
}
#[derive(Debug)]
#[allow(dead_code)]
pub struct Assignment {
    pub id: i64,
    pub name: String,
    pub course: Rc<RefCell<Course>>,
    pub description: String,
    pub filelink: Vec<String>,
}
impl GetFromJson<Assignment, Rc<RefCell<Course>>, i32> for Assignment {
    fn get_from_json(x: &Value, c: Rc<RefCell<Course>>, _: i32) -> Option<Assignment> {
        if x["locked_for_user"].as_bool()? {
            println!(
                "{}",
                t!(
                    "Assignment %{name} is locked, skip it",
                    name = x["display_name"].as_str()?
                )
            );
            return None;
        }
        let filelink = get_file_link_from_html(x["description"].as_str()?);
        Some(Assignment {
            id: x["id"].as_i64()?,
            name: x["name"].as_str()?.to_string(),
            course: c,
            description: x["description"].as_str()?.to_string(),
            filelink,
        })
    }
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
