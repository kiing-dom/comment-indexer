use std::path::Path;

pub enum Language {
    Java,
    Python,
    JavaScript,
    TypeScript,
    Rust
}

pub struct CommentMatch {
    pub start_byte: usize,
    pub end_byte: usize,
    #[allow(dead_code)]
    pub text: String,
    #[allow(dead_code)]
    pub comment_type: CommentType,
}

pub enum CommentType {
    SingleLine, // single line - language agnostic
    MultiLine, // multiline - language agnostic
}

pub fn extract_comments_from_content(content: &str, language: Language) -> Vec<CommentMatch> {
    match language {
        Language::Java => extract_java_comments(content),
        Language::Python => extract_python_comments(content),
        Language::JavaScript => extract_javascript_comments(content),
        Language::TypeScript => extract_javascript_comments(content), // Same as JS
        Language::Rust => extract_java_comments(content), // Same as Java
    }
}

enum JavaParseState {
    Code,
    SingleLineComment,
    MultiLineComment,
    StringLiteral,
    CharLiteral
}

enum PythonParseState {
    Code,
    SingleLineComment,
    StringLiteral,
    TripleQuotedString,
}

pub fn detect_language(file_path: &Path) -> Option<Language> {
    match file_path.extension()?.to_str()? {
        "java" => Some(Language::Java),
        "rs" => Some(Language::Rust),
        "py" => Some(Language::Python),
        "js" => Some(Language::JavaScript),
        "ts" | "tsx" => Some(Language::TypeScript),
        _ => None,
    }
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
                    '\'' => { state = JavaParseState::Code; },
                    _ => { /* do nothing (keep scanning) */}
                }
            }
        }
    }
    comments
}

fn extract_python_comments(content: &str) -> Vec<CommentMatch> {
    let mut comments = Vec::new();
    let mut state = PythonParseState::Code;
    let mut chars = content.char_indices().peekable();

    let mut comment_start: Option<usize> = None;
    let mut comment_text = String::new();

    while let Some((byte_pos, ch)) = chars.next() {
        match state {
            PythonParseState::Code => {
                match ch {
                    '#' => {
                        comment_start = Some(byte_pos);
                        comment_text.clear();
                        state = PythonParseState::SingleLineComment;
                    },
                    '"' | '\'' => {
                        // Check for triple quotes
                        if let Some(&(_, next_ch)) = chars.peek() {
                            if next_ch == ch {
                                chars.next(); // consume second quote
                                if let Some(&(_, third_ch)) = chars.peek() {
                                    if third_ch == ch {
                                        chars.next(); // consume third quote
                                        state = PythonParseState::TripleQuotedString;
                                        continue;
                                    }
                                }
                            }
                        }
                        state = PythonParseState::StringLiteral;
                    },
                    _ => {}
                }
            },
            PythonParseState::SingleLineComment => {
                if ch == '\n' {
                    if let Some(start) = comment_start {
                        comments.push(CommentMatch {
                            start_byte: start,
                            end_byte: byte_pos,
                            text: comment_text.clone(),
                            comment_type: CommentType::SingleLine,
                        });
                    }
                    comment_text.clear();
                    comment_start = None;
                    state = PythonParseState::Code;
                } else {
                    comment_text.push(ch);
                }
            },
            PythonParseState::StringLiteral => {
                match ch {
                    '"' | '\'' => state = PythonParseState::Code,
                    '\\' => { chars.next(); }, // Skip escaped character
                    _ => {}
                }
            },
            PythonParseState::TripleQuotedString => {
                if ch == '"' || ch == '\'' {
                    // Check for end of triple quotes
                    if let Some(&(_, next_ch)) = chars.peek() {
                        if next_ch == ch {
                            chars.next();
                            if let Some(&(_, third_ch)) = chars.peek() {
                                if third_ch == ch {
                                    chars.next();
                                    state = PythonParseState::Code;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Handle comment at end of file
    if let Some(start) = comment_start {
        comments.push(CommentMatch {
            start_byte: start,
            end_byte: content.len(),
            text: comment_text,
            comment_type: CommentType::SingleLine,
        });
    }

    comments
}

fn extract_javascript_comments(content: &str) -> Vec<CommentMatch> {
    // JavaScript and TypeScript use same comment syntax as Java: // and /* */
    extract_java_comments(content)
}
