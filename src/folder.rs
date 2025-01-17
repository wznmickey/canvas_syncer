use crate::course::Course;
use crate::util::GetFromJson;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;
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
