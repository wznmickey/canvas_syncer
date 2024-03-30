mod download;
mod my_key;
mod course;
// use futures::executor::block_on;
use crate::download::RemoteData;
fn main() {
    // println!("Hello, world!");
    let x = RemoteData::new();
    let mycourses=x.get_course_list();
    // for course in mycourses{
    //     course.download_files();
    // }
}
