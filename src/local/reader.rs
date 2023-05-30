use std::fs;

use crate::FILE_SPLITTER;

pub static BACKUP_SRC: &str = ".backupsrc";
pub static BACKUP_IGNORE: &str = ".backupignore";
pub static BACKUP_DIR: &str = "backupdata";

pub fn load_src_files() -> Result<Vec<String>, std::io::Error> {
    let mut files: Vec<String> = Vec::new();
    let content = fs::read_to_string(BACKUP_SRC);

    match content {
        Ok(content) => {
            for line in content.lines() {
                files.push(line.to_string());
            }
            Ok(files)
        },
        Err(e) => Err(e)
    }
}

pub fn load_ignore_files(path: String) -> Vec<String> {
    let path = format!("{}{}{}", path, FILE_SPLITTER, BACKUP_IGNORE);
    let mut files: Vec<String> = Vec::new();
    let content_res = fs::read_to_string(path);

    match content_res {
        Ok(content) => {
            for line in content.lines() {
                files.push(line.to_string());
            }
        }
        Err(_) => { }
    };
    files
}