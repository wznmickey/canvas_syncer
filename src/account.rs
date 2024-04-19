use crate::config::*;
use crate::course::*;
use crate::download::*;
pub struct Account {
    config: Config,
    remote_data: RemoteData,
    course: Vec<Course>,
    downloadfolders: Vec<DownloadFolder>
}

impl Account {
    pub fn new(st: &str) -> Self {
        let config = Config::read_file(st);
        let remote_data = RemoteData::new(&config.key, &config.canvas_url);
        let course = remote_data.get_course_list();
        Self {
            config,
            remote_data,
            course,
            downloadfolders:Vec::new()
        }
    }

}
