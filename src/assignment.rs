use crate::course::Course;
use crate::util::get_file_link_from_html;
use crate::util::GetFromJson;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;
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
