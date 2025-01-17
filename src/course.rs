use crate::util::GetFromJson;
use serde_json::Value;

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
