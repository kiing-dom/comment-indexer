mod core;

fn main() {
    let curr_dir = std::env::current_dir().expect("Failed to get current directory!");
    let file_paths = core::discover::find_source_files(&curr_dir, "java");
    let files = core::source::load_files(&file_paths);
    let comments = core::engine::extract_comments(&files);

    for comment in comments {
        println!("'{}':{}: {}", comment.file_name, comment.line, comment.text);
    }

    println!("found {} files", file_paths.len());
}