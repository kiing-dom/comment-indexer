use super::source::SourceFile;
use regex::Regex;

pub struct Comment<'a> {
    pub line: usize,
    pub text: &'a str,
    pub file_name: &'a str,
}

pub fn extract_comments<'a>(files: &'a [SourceFile]) -> Vec<Comment<'a>> {
    let re = Regex::new(r"//.*|/\*[\s\S]*?\*/").unwrap();
    let mut comments = Vec::new();

    for file in files {
        let mut line_offsets = vec![0];
        for (i, c) in file.content.char_indices() {
            if c == '\n' {
                line_offsets.push(i + 1);
            }
        }
        for comment_match in re.find_iter(&file.content) {
            let start = comment_match.start();
            let line_num = match line_offsets.binary_search(&start) {
                Ok(idx) => idx + 1,
                Err(idx) => idx,
            };

            comments.push(Comment {
                line: line_num,
                text: comment_match.as_str(),
                file_name: &file.name,
            });
        }
    }
    comments
}
