use std::fs;
use std::fs::{File, FileType};
use std::path::Path;
use tempdir::TempDir;
use walkdir::WalkDir;
use rust_compiler_construction::compile;

fn main() {
    let tempdir = TempDir::new("cc-bench").unwrap();

    let path = Path::new("../programs/good");
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()).filter(|f| f.file_type().is_file()) {
        let content = fs::read_to_string(entry.path()).unwrap();
        compile(&content, &tempdir.path().join("output")).unwrap();

    }

}
