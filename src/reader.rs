use std::fs;

pub fn read_file_content(path: String) -> String {
    fs::read_to_string(path).expect("File not found")
}