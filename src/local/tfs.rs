use std::fs;
use std::fs::{DirEntry, File};
use std::path::Path;
use chrono::NaiveDateTime;
use flate2::Compression;
use flate2::read::GzEncoder;

use glob::glob;
use log::{error, info};
use crate::FILE_SPLITTER;

use crate::local::reader;
use crate::local::reader::{BACKUP_DIR, BACKUP_IGNORE, BACKUP_SRC};
use crate::types::Ctx;

pub static BACKUP_GROUP_DIR: &str = "backups";

pub fn copy_dir(src: &str, dest: &str, ignore: &Vec<String>) {
    let dir = fs::read_dir(src).unwrap();
    let files = dir.filter(|f| {
        let file = f.as_ref().unwrap();
        let file_path = file.path();
        let file_path_str = file_path.to_str().unwrap();
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
            fs::create_dir_all(parent).unwrap();
            fs_extra::file::copy(&file_path, &dest_path, &options).unwrap();
        }
    }
}

pub fn copy_files(_ctx: &Ctx) -> Result<(), ()> {
    let mut files: Vec<String> = Vec::new();
    let mut ignore_files: Vec<String> = Vec::new();
    let src_files_res = reader::load_src_files();
    let src_globs: Vec<String>;

    match src_files_res {
        Ok(src_files) => {
            src_globs = src_files;
        },
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                error!("No src files found. Please create a {} file in the current directory.", BACKUP_SRC);
            } else {
                error!("Error reading src files: {}", e);
                return Err(());
            }
            return Err(());
        }
    }

    for g in src_globs {
        let gl = glob(&g).unwrap();
        for entry in gl {
            match entry {
                Ok(path) => {
                    let mut parent = path.parent();
                    while parent.is_some() {
                        let parent_path = parent.unwrap();
                        let parent_str = parent_path.to_str().unwrap();
                        let ignore_file = format!("{}{}{}", parent_str, FILE_SPLITTER, BACKUP_IGNORE);
                        let ignore_meta = fs::metadata(&ignore_file);
                        if ignore_meta.is_ok() {
                            let i = reader::load_ignore_files(parent_str.to_string());
                            for ignore_file in i {
                                let ignore_file_glob = format!("{}{}{}", parent_str, FILE_SPLITTER, ignore_file);
                                let gl = glob(&ignore_file_glob).unwrap();
                                for entry in gl {
                                    match entry {
                                        Ok(path) => {
                                            if !ignore_files.contains(&path.to_str().unwrap().to_string()) {
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

    let backup_meta = fs::metadata(BACKUP_DIR);
    if backup_meta.is_ok() {
        fs::remove_dir_all(BACKUP_DIR).unwrap();
    }
    fs::create_dir_all(BACKUP_DIR).unwrap();
    // Copy files
    for file in &files {
        let meta = fs::metadata(&file).unwrap();
        if meta.is_dir() {
            copy_dir(&file, &format!("{}{}{}", BACKUP_DIR, FILE_SPLITTER, file), &ignore_files);
        } else {
            let options = fs_extra::file::CopyOptions::new();
            let final_path = format!("{}{}{}", BACKUP_DIR, FILE_SPLITTER, file);
            let path = Path::new(&final_path);
            let parent = path.parent().unwrap();
            fs::create_dir_all(parent).unwrap();
            if ignore_files.contains(&file) {
                continue;
            }
            fs_extra::file::copy(&file, &final_path, &options).unwrap();
        }
    }
    info!("Copy complete!");
    Ok(())
}

fn prune_old_local_backups(context: &Ctx) {
    let max_count = context.config.rotate_count;

    if max_count == 0 {
        info!("Rotation disabled. Skipping deletion of old backups!");
        return;
    }
    let backup_group_meta = fs::metadata(BACKUP_GROUP_DIR);
    if backup_group_meta.is_err() {
        return;
    }

    let mut files: Vec<DirEntry> = Vec::new();
    for entry in fs::read_dir(BACKUP_GROUP_DIR).unwrap() {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        if file_type.is_file() {
            files.push(entry);
        }
    }

    let mut backup_files: Vec<String> = Vec::new();
    for file in &files {
        let file_name = file.file_name();
        let file_name_str = file_name.to_str().unwrap();
        if file_name_str.ends_with(format!("-backup{}", context.config.archive_format).as_str()) {
            backup_files.push(file_name_str.to_string());
        }
    }

    info!("Found {} backups", files.len());

    // Sort backup files by time
    if !context.config.timestamp_prefix {
        backup_files.sort_by(|a, b| {
            let a_time = a.split(format!("-backup{}", context.config.archive_format).as_str()).collect::<Vec<&str>>()[0];
            let b_time = b.split(format!("-backup{}", context.config.archive_format).as_str()).collect::<Vec<&str>>()[0];
            let a_time = NaiveDateTime::parse_from_str(a_time, "%Y-%m-%d-%H-%M-%S").unwrap();
            let b_time = NaiveDateTime::parse_from_str(b_time, "%Y-%m-%d-%H-%M-%S").unwrap();
            a_time.cmp(&b_time)
        });
    } else {
        backup_files.sort_by(|a, b| {
            let a_time = a.split(format!("-backup{}", context.config.archive_format).as_str()).collect::<Vec<&str>>()[0];
            let b_time = b.split(format!("-backup{}", context.config.archive_format).as_str()).collect::<Vec<&str>>()[0];
            a_time.cmp(&b_time)
        });
    }
    backup_files.reverse();

    if context.config.dry_run {
        let to_remove = if backup_files.len() > max_count {
            backup_files.len() - max_count
        } else {
            0
        };
        info!("Dry run: Would remove {} backups", to_remove);
        return;
    }

    // Remove old backups
    let mut count = 0;
    for file in backup_files {
        if count >= max_count {
            let file_path = format!("{}{}{}", BACKUP_GROUP_DIR, FILE_SPLITTER, file);
            info!("Removing old backup: {}", file_path);
            fs::remove_file(file_path).unwrap();
        }
        count += 1;
    }

    info!("Prune complete!");
}

pub fn compress_backup(context: &Ctx) -> Result<(), ()> {
    // If BACKUP_GROUP_DIR does not exist, create it
    let backup_group_meta = fs::metadata(BACKUP_GROUP_DIR);
    if backup_group_meta.is_err() {
        fs::create_dir_all(BACKUP_GROUP_DIR).unwrap();
    }

    // Compress backup
    let archive_name = format!("{}{}{}", BACKUP_GROUP_DIR, FILE_SPLITTER, context.backup_filename);
    info!("Creating archive {}", archive_name);
    let tar_gz = File::create(archive_name).unwrap();
    let encoder = GzEncoder::new(&tar_gz, Compression::default());
    let mut tar = tar::Builder::new(encoder);
    tar.append_dir_all("data", BACKUP_DIR).expect("Error compressing backup");
    info!("Backup compressed!");

    // Remove backup dir
    fs::remove_dir_all(BACKUP_DIR).expect("Error removing backup dir");

    // Prune old backups
    prune_old_local_backups(&context);
    Ok(())
}
