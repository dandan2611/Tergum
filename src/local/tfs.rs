use std::fs;
use std::path::Path;

use log::info;

use crate::FILE_SPLITTER;

pub fn copy_dir(src: &str, dest: &str, ignore: &Vec<String>) {
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
            fs::create_dir_all(parent).unwrap();
            info!("Copying file: {} to {}", file_path, dest_path);
            fs_extra::file::copy(&file_path, &dest_path, &options).unwrap();
        }
    }
}