mod reader;

use std::fs;
use std::fs::{copy, create_dir, create_dir_all, metadata, read_dir, remove_dir, remove_dir_all, remove_file};
use std::path::Path;
use fs_extra::dir;
use fs_extra::dir::{copy_with_progress};
use glob::glob;
use log::{debug, info};
use simple_logger::SimpleLogger;
use crate::reader::read_file_content;

static BACKUP_SRC: &str = ".backupsrc";
static BACKUP_IGNORE: &str = ".backupignore";
static BACKUP_DIR: &str = "backup";

#[cfg(target_os = "windows")]
static FILE_SPLITTER: &str = "\\";
#[cfg(target_os = "linux")]
static FILE_SPLITTER: &str = "/";
#[cfg(target_os = "macos")]
static FILE_SPLITTER: &str = "/";

fn load_src_files() -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    let content = read_file_content(BACKUP_SRC.to_string());

    for line in content.lines() {
        files.push(line.to_string());
    }
    files
}

fn load_ignore_file(path: String) -> Vec<String> {
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

fn copy_dir(src: &str, dest: &str, mut ignore: &Vec<String>) {
    // Print ignore files
    for i in ignore {
        println!("Ignore file: {}", i);
    }
    let dir = fs::read_dir(src).unwrap();
    let files = dir.filter(|f| {
        let file = f.as_ref().unwrap();
        let file_path = file.path();
        let file_path_str = file_path.to_str().unwrap();
        info!("Checking file: {}", file_path_str);
        !ignore.contains(&file_path_str.to_string())
    });

    for file in files {
        let file = file.unwrap();
        let file_type = file.file_type().unwrap();
        let file_name = file.file_name();
        let file_name_str = file_name.to_str().unwrap();
        let file_path = format!("{}{}{}", src, FILE_SPLITTER, file_name_str);
        let dest_path = format!("{}{}{}", dest, FILE_SPLITTER, file_name_str);
        if file_type.is_dir() {
            copy_dir(&file_path, &dest_path, ignore);
        } else {
            let options = fs_extra::file::CopyOptions::new();
            let path = Path::new(&dest_path);
            let parent = path.parent().unwrap();
            create_dir_all(parent).unwrap();
            info!("Copying file: {} to {}", file_path, dest_path);
            fs_extra::file::copy(&file_path, &dest_path, &options).unwrap();
        }
    }
}

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
                            let i = load_ignore_file(parent_str.to_string());
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
