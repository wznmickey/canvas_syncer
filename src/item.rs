use crate::module::Module;
// use crate::util::get_file_link_from_html;
use crate::util::GetFromJson;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;
#[derive(Debug)]
#[allow(dead_code)]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub module: Rc<RefCell<Module>>,
    pub url: String,
}
impl GetFromJson<Item, Rc<RefCell<Module>>, i32> for Item {
    fn get_from_json(x: &Value, c: Rc<RefCell<Module>>, _: i32) -> Option<Item> {
        Some(Item {
            id: x["id"].as_i64()?,
            name: x["title"].as_str()?.to_string(),
            module: c,
            url: x["url"].as_str()?.to_string() + "?",
        })
    }
}
