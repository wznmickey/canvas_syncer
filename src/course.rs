use serde_json::Value;
use std::rc::Rc;
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
    pub course: Rc<Course>
}
pub fn get_folder_from_json (x: &Value, c:Rc<Course>) -> Option<Folder> {
    Some(Folder {
        id: x["id"].as_i64()?,
        name: x["name"].as_str()?.to_string(),
        fullname: x["full_name"].as_str()?.to_string(),
        course:c
    })
}
