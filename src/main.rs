use regex::Regex;
use std::fs;
use std::env;

struct Comment<'a> {
    line: usize,
    text: &'a str,
    file_name: &'a str,
}

struct SourceFile {
    name: String,
    content: String,
}

fn parse_comments<'a>(file_name: &'a str, s: &'a str) -> Vec<Comment< 'a>> {
    let re = Regex::new(r"//.*|/\*[\s\S]*?\*/").unwrap();
    let mut comments = Vec::new();
    let mut starts = vec![0];

    for (i, c) in s.char_indices() {
        if c == '\n' {
            starts.push(i + 1);
            dbg!(file_name, &starts);
        }
    }

    for m in re.find_iter(s) {
        let start = m.start();
        let line_num = match starts.binary_search(&start) {
            Ok(idx) => idx + 1,
            Err(idx) => idx,
        };

        comments.push(Comment {
            line: line_num,
            text: m.as_str(),
            file_name,
        });
    }

    comments
}

fn main() {
    let mut comments = Vec::new();
    let mut files = Vec::new();

    let curr_dir = env::current_dir().expect("Failed to get current directory");

    match fs::read_dir(&curr_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {

                let path = entry.path();

                if path.is_file() {
                    if path.extension().and_then(|s| s.to_str()) == Some("java") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            files.push(SourceFile{
                                name: path.to_string_lossy().into_owned(),
                                content: content,
                            });
                        }
                    }
                }
            }
        },
        Err(e) => eprintln!("Error reading directory {}", e),
    }

    for file in &files {
        
        let found = parse_comments(&file.name, &file.content);
        for c in found {
            comments.push(Comment{
                line: c.line,
                text: c.text,
                file_name: &file.name
            });
        }
    }

    // print lines found
    if comments.len() == 0 {
        println!("0 comments found")
    } else if comments.len() == 1 {
        println!("{} comment found", comments.len())
    } else {
        println!("{} comments found", comments.len())
    }

    // print each comment
    for comment in comments {
        println!("{}, line {}: {}", comment.file_name, comment.line, comment.text,);
    }
}