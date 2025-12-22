use regex::Regex;
use std::fs;

struct Comment {
    line: usize,
    text: String,
}

fn main() {
    let contents = fs::read_to_string("example.java").unwrap();
    let re = Regex::new(r"//.*|/\*[\s\S]*?\*/").unwrap();

    let mut comments = Vec::new();

    for m in re.find_iter(&contents) {
        let start = m.start();
        let line = contents[..start].matches("\n").count() + 1;
        comments.push(Comment {
            text: m.as_str().to_string(),
            line: line
        });
    }

    println!("{} comments found", comments.len());
    for comment in comments {
        print!("Line {}: {}", comment.line, comment.text);
    }
}
