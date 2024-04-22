use chrono::DateTime;
use chrono::Utc;
use serde_json::Value;
use std::cmp::max;
use std::{fs, path::PathBuf, rc::Rc};
pub enum FileStatus {
    Latest,
    NeedUpdate,
    NotExist,
}
#[derive(Debug)]
pub struct Course {
    pub id: i64,
    pub name: String,
    pub course_code: String,
    pub term_id: i64,
    pub term_name: String,
}
pub fn get_course_from_json(x: &Value) -> Option<Course> {
    Some(Course {
        id: x["id"].as_i64()?,
        name: x["name"].as_str()?.to_string(),
        course_code: x["course_code"].as_str()?.to_string(),
        term_id: x["enrollment_term_id"].as_i64()?,
        term_name: x["term"]["name"].as_str()?.to_string(),
    })
}

#[derive(Debug)]
pub struct Folder {
    id: i64,
    pub name: String,
    pub fullname: String,
    pub course: Rc<Course>,
    pub filelink: String,
}
pub fn get_folder_from_json(x: &Value, c: Rc<Course>) -> Option<Folder> {
    Some(Folder {
        id: x["id"].as_i64()?,
        name: x["name"].as_str()?.to_string(),
        fullname: x["full_name"].as_str()?.to_string(),
        course: c,
        filelink: x["files_url"].as_str()?.to_string() + "?",
    })
}

#[derive(Debug)]
pub struct File {
    id: i64,
    pub display_name: String,
    pub filename: String,
    pub url: String,
    pub folder: Rc<Folder>,
    pub content_type: String,
    pub size: u64,
    pub created_time: DateTime<Utc>,
    pub updated_time: DateTime<Utc>,
    pub modified_time: DateTime<Utc>,
    pub my_full_path: PathBuf,
}
pub fn get_file_from_json(x: &Value, f: Rc<Folder>, mut path: PathBuf) -> Option<File> {
    let temp = x["display_name"].as_str()?.to_string();
    Some(File {
        id: x["id"].as_i64()?,
        my_full_path: {
            path.push((&f.course.name).to_string() + " " + &f.fullname);
            path.push(&temp);
            path
        },
        display_name: temp,
        filename: x["filename"].as_str()?.to_string(),
        folder: f,
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
impl File {
    pub fn get_status(&self) -> FileStatus {
        let x = fs::metadata(self.my_full_path.clone());
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
