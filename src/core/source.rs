// handle loading file contents
use std::path::{ PathBuf };
use std::fs;

pub struct SourceFile {
    pub name: String,
    pub content: String,
}

pub fn load_files(paths: &[PathBuf]) -> Vec<SourceFile> {
    let mut files = Vec::new();
    
    for path in paths {
        if let Ok(content) = fs::read_to_string(path) {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            files.push(SourceFile { name, content });
        }
    }

    files
}