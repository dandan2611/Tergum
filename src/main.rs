use std::fs::{create_dir_all, metadata, remove_dir_all};
use std::path::Path;

use glob::glob;
use log::{debug, info};
use simple_logger::SimpleLogger;

use crate::local::reader::{BACKUP_DIR, BACKUP_IGNORE, load_ignore_files, load_src_files};
use crate::local::tfs::copy_dir;

mod local;

#[cfg(target_os = "windows")]
static FILE_SPLITTER: &str = "\\";
#[cfg(target_os = "linux")]
static FILE_SPLITTER: &str = "/";
#[cfg(target_os = "macos")]
static FILE_SPLITTER: &str = "/";

fn main() {
    SimpleLogger::new().init().unwrap();

    let mut files: Vec<String> = Vec::new();
    let mut ignore_files: Vec<String> = Vec::new();
    let src_globs = load_src_files();

    for g in src_globs {
        let gl = glob(&g).unwrap();
        for entry in gl {
            match entry {
                Ok(path) => {
                    let mut parent = path.parent();
                    while parent.is_some() {
                        debug!("Checking parent: {}", parent.unwrap().to_str().unwrap());
                        let parent_path = parent.unwrap();
                        let parent_str = parent_path.to_str().unwrap();
                        let ignore_file = format!("{}{}{}", parent_str, FILE_SPLITTER, BACKUP_IGNORE);
                        let ignore_meta = metadata(&ignore_file);
                        if ignore_meta.is_ok() {
                            info!("Found ignore file: {}", ignore_file);
                            let i = load_ignore_files(parent_str.to_string());
                            for ignore_file in i {
                                let ignore_file_glob = format!("{}{}{}", parent_str, FILE_SPLITTER, ignore_file);
                                let gl = glob(&ignore_file_glob).unwrap();
                                for entry in gl {
                                    match entry {
                                        Ok(path) => {
                                            if !ignore_files.contains(&path.to_str().unwrap().to_string()) {
                                                println!("Ignoring file: {}", path.to_str().unwrap());
                                                ignore_files.push(path.to_str().unwrap().to_string());
                                            }
                                        }
                                        Err(e) => println!("{:?}", e),
                                    }
                                }
                            }
                            break;
                        }
                        if (parent_str == ".") || (parent_str == "/") {
                            break;
                        }
                        parent = parent_path.parent();
                    }
                    if ignore_files.contains(&path.to_str().unwrap().to_string()) {
                        continue;
                    }
                    files.push(path.to_str().unwrap().to_string());
                }
                Err(e) => println!("{:?}", e),
            }
        }
    }

    let backup_meta = metadata(BACKUP_DIR);
    if backup_meta.is_ok() {
        remove_dir_all(BACKUP_DIR).unwrap();
    }
    create_dir_all(BACKUP_DIR).unwrap();
    // Copy files
    for file in files {
        let meta = metadata(&file).unwrap();
        if meta.is_dir() {
            copy_dir(&file, &format!("{}{}{}", BACKUP_DIR, FILE_SPLITTER, file), &ignore_files);
        } else {
            let options = fs_extra::file::CopyOptions::new();
            let final_path = format!("{}{}{}", BACKUP_DIR, FILE_SPLITTER, file);
            let path = Path::new(&final_path);
            let parent = path.parent().unwrap();
            create_dir_all(parent).unwrap();
            info!("legacy Copying file: {} to {}", file, final_path);
            if ignore_files.contains(&file) {
                continue;
            }
            fs_extra::file::copy(&file, &final_path, &options).unwrap();
        }
    }
}
