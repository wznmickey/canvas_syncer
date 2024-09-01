use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Filter<T> {
    pub denies: Option<Vec<T>>,
    pub allows: Option<Vec<T>>,
}
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct ObjectFilter {
    pub name: Option<Filter<String>>,
    pub id: Option<Filter<i64>>,
    pub default: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct FileFilter {
    pub size_greater: Option<u64>,
    pub size_smaller: Option<u64>,
    pub type_filter: Option<Filter<String>>,
    pub default: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct CourseFilter {
    pub object_filter: Option<ObjectFilter>,
}
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct TermFilter {
    pub object_filter: Option<ObjectFilter>,
}
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Filters {
    pub course_filter: Option<CourseFilter>,
    pub term_filter: Option<TermFilter>,
    pub file_filter: Option<FileFilter>,
}


