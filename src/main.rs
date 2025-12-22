use regex::Regex;
use std::fs;

struct Comment {
    text: String,
}

fn main() {
    let contents = fs::read_to_string("example.java").unwrap();
    let re = Regex::new(r"//.*|/\*[\s\S]*?\*/").unwrap();

    for m in re.find_iter(&contents) {
        print!("---{}", m.as_str());
    }
}
