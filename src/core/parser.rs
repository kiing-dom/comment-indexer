pub enum Language {
    Java,

}

pub struct CommentMatch {
    pub start_byte: usize,
    pub end_byte: usize,
    pub text: String,
    pub comment_type: CommentType,
}

pub enum CommentType {
    SingleLine, // single line - language agnostic
    MultiLine, // multiline - language agnostic
}

pub fn extract_comments_from_content(content: &str, language: Language) {
    match language {
        Language::Java => { extract_java_comments(content); }
    }
}

enum JavaParseState {
    Code,
    SingleLineComment,
    MultiLineComment,
    StringLiteral,
    CharLiteral
}

fn extract_java_comments(content: &str) -> Vec<CommentMatch> {
    let mut comments = Vec::new();
    let mut state = JavaParseState::Code;
    let mut chars = content.char_indices().peekable();

    let mut comment_start: Option<usize> = None;
    let mut comment_text = String::new();
    
    while let Some((byte_pos, ch)) = chars.next() {
        match state {
            JavaParseState::Code => {
                match ch {
                    '/' => {
                        if let Some(&(_, next_ch)) = chars.peek() {
                            match next_ch {
                                '/' => {
                                    comment_start = Some(byte_pos);
                                    comment_text.clear();
                                    state = JavaParseState::SingleLineComment;
                                    chars.next();
                                },
                                '*' => {
                                    comment_start = Some(byte_pos);
                                    comment_text.clear();
                                    state = JavaParseState::MultiLineComment;
                                    chars.next();
                                },
                                _ => { continue }
                            }
                        }
                    },
                    '"' => { state = JavaParseState::StringLiteral },
                    '\'' => { state = JavaParseState::CharLiteral },
                    _ => { continue }
                }
            },
            JavaParseState::SingleLineComment => {
                if ch == '\n' {
                    let comment_match = CommentMatch {
                        start_byte: comment_start.unwrap(),
                        end_byte: byte_pos, // the curr pos after moving to new line
                        text: comment_text.clone(),
                        comment_type: CommentType::SingleLine,
                    };

                    comments.push(comment_match);
                    comment_text.clear();
                    state = JavaParseState::Code;
                }
            },
            JavaParseState::MultiLineComment => {
               match ch {
                 '*' => {
                    if let Some(&(_, next_ch)) = chars.peek() {
                        if next_ch == '/' {
                            let comment_match = CommentMatch {
                                start_byte: comment_start.unwrap(),
                                end_byte: byte_pos,
                                text: comment_text.clone(),
                                comment_type: CommentType::MultiLine,
                            };

                            comments.push(comment_match);
                            comment_text.clear();
                            state = JavaParseState::Code;
                            chars.next();
                        } else {
                            comment_text.push(ch);
                        }
                    }
                 },
                 
                 _ => {
                    comment_text.push(ch);
                 }
               } 
            },
            JavaParseState::StringLiteral => {
                match ch {
                    '\\' => { chars.next(); },
                    '"' => { state = JavaParseState::Code; },
                    _ => { /* do nothing (keep scanning) */ }
                }
            },
            JavaParseState::CharLiteral => {
                match ch {
                    '\\' => { chars.next(); },
                    '\'' => { JavaParseState::Code; },
                    _ => { /* do nothing (keep scanning) */}
                }
            }
        }
    }
    comments
}