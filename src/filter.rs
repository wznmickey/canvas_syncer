use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Filter<T> {
    pub denies: Option<Vec<T>>,
    pub allows: Option<Vec<T>>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectFilter {
    // default could be `allow` or `deny`.
    // If no default, `allow` will be used.
    // If one object is not in the filter, it will follow the default.
    // Either name or id could appear.
    // If both appear, **UB**.
    // If both none, all will follow default.
    //
    pub name: Option<Filter<String>>,
    pub id: Option<Filter<i64>>,
    pub default: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct FileFilter {
    pub size_greater: Option<u64>,
    pub size_smaller: Option<u64>,
    pub type_filter: Option<Filter<String>>,
    pub default: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CourseFilter {
    pub object_filter: Option<ObjectFilter>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct TermFilter {
    pub object_filter: Option<ObjectFilter>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Filters {
    pub course_filter: Option<CourseFilter>,
    pub term_filter: Option<TermFilter>,
    pub file_filter: Option<FileFilter>,
}

// true -> Allow
// false -> Deny
pub fn object_filter_check(f: &ObjectFilter, id: i64, name: &String) -> bool {
    let mut default_setting = "allow";
    if let Some(ref x) = f.default {
        default_setting = x
    }
    if default_setting == "deny" {
        if let Some(ref x) = f.id {
            if x.allows.is_some() && x.allows.as_ref().unwrap().contains(&id) {
                return true;
            }
        };
        if let Some(ref x) = f.name {
            if x.allows.is_some() && x.allows.as_ref().unwrap().contains(name) {
                return true;
            }
        };
        return false;
    }

    if default_setting == "allow" {
        if let Some(ref x) = f.id {
            if x.denies.is_some() && x.allows.as_ref().unwrap().contains(&id) {
                return false;
            }
        };
        if let Some(ref x) = f.name {
            if x.allows.is_some() && x.denies.as_ref().unwrap().contains(name) {
                return false;
            }
        };
        return true;
    }
    true
}
