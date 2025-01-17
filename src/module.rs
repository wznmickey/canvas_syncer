use crate::structs::*;
use crate::util::GetFromJson;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;
#[derive(Debug)]
#[allow(dead_code)]
pub struct Module {
    pub id: i64,
    pub name: String,
    pub course: Rc<RefCell<Course>>,
    pub itemline: String,
}
impl GetFromJson<Module, Rc<RefCell<Course>>, i32> for Module {
    fn get_from_json(x: &Value, c: Rc<RefCell<Course>>, _: i32) -> Option<Module> {
        Some(Module {
            id: x["id"].as_i64()?,
            name: x["name"].as_str()?.to_string(),
            course: c,
            itemline: x["items_url"].as_str()?.to_string() + "?",
        })
    }
}
