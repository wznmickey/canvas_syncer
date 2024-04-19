use crate::download::*;
use serde_json::Error;
use serde_json::Value;

#[derive(Debug)]
pub struct Course {
    id: i64,
    name: String,
    course_code: String,
    term_id: i64,
    term_name: String
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
pub struct DownloadFolder {
    name: String,
    local_path: String,
    remote_url: String,
}