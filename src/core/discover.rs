use std::fs;
use std::path::{Path, PathBuf};

pub fn find_all_source_files(dir: &Path) -> Vec<PathBuf> {
    let mut found_files = Vec::new();

    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        if should_skip_directory(dir_name) {
                            continue;
                        }
                    }

                    let mut sub_files = find_all_source_files(&path);
                    found_files.append(&mut sub_files);

                } else if path.is_file() {
                    if path.extension().is_some() {
                        found_files.push(path);
                    }
                }
            }
        }

        Err(e) => eprintln!("Error reading directory: {}", e),
    }

    found_files
}

fn should_skip_directory(dir_name: &str) -> bool {
    matches!(
        dir_name,
        "target"
            | "node_modules"
            | ".git"
            | "build"
            | "dist"
            | ".vscode"
            | ".idea"
            | "bin"
            | "obj"
            | "__pycache__"
    )
}
