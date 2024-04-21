mod download;
mod course;
mod config;
mod account;

fn main() {
    
    let mut x = account::Account::new("/home/wznmickey/github/canvas_syncer/src/config.json");
    x.get_folders();
    x.create_folders();
    // x.get_files();
}
