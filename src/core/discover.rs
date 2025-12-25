use std::path::{Path, PathBuf};
use std::fs;

pub fn find_source_files(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let mut found_files = Vec::new();

    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() {
                    if path.extension().and_then(|s| s.to_str()) == Some(extension) {
                        found_files.push(path);
                    }
                }
            }
        },

        Err(e) => eprintln!("Error reading directory: {}", e),
    }

    found_files
}