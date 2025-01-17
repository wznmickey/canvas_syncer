use crate::structs::*;
use crate::util::get_file_link_from_html;
use crate::util::GetFromJson;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;
#[derive(Debug)]
#[allow(dead_code)]
pub struct Page {
    pub name: String,
    pub item: Rc<RefCell<Item>>,
    pub filelink: Vec<String>,
}
impl GetFromJson<Page, Rc<RefCell<Item>>, i32> for Page {
    fn get_from_json(x: &Value, c: Rc<RefCell<Item>>, _: i32) -> Option<Page> {
        if x["locked_for_user"].as_bool()? {
            println!(
                "{}",
                t!(
                    "Page %{name} is locked, skip it",
                    name = x["display_name"].as_str()?
                )
            );
            return None;
        }
        let filelink = get_file_link_from_html(x["body"].as_str()?);
        Some(Page {
            name: x["title"].as_str()?.to_string(),
            item: c,
            filelink,
        })
    }
}
