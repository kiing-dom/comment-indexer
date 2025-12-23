use regex::Regex;
use std::fs;
use std::env;

struct Comment<'a> {
    line: usize,
    text: &'a str,
}

fn parse_comments<'a>(s: &'a str) -> Vec<Comment< 'a>> {
    let re = Regex::new(r"//.*|/\*[\s\S]*?\*/").unwrap();
    let mut comments = Vec::new();

    for m in re.find_iter(s) {
        let start = m.start();
        let line_num = s[..start].matches("\n").count() + 1;

        comments.push(Comment {
            line: line_num,
            text: m.as_str()
        });
    }

    comments
}

fn main() {
    let contents = fs::read_to_string("example.java").expect("Could not read file");
    let comments = parse_comments(&contents);

    let curr_dir = env::current_dir().expect("Failed to get current directory");

    match fs::read_dir(curr_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("{}", entry.path().display())
                }
            }
        },
        Err(e) => eprintln!("Error reading directory {}", e),
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
        println!("{}: {}", comment.line, comment.text);
    }
}