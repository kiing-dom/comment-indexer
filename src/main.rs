mod core;
mod tui;

use clap::Parser;
use std::path::Path;
use core::search;

#[derive(Parser)]
#[command(name = "fuzc")]
#[command(about = "a fuzzy finder for code comments")]
struct Args {
    #[arg(long)]
    query: Option<String>, // Optional query for CLI mode

    #[arg(short, long, default_value = ".")]
    directory: String, // directory to search (default: current directory)

    #[arg(long)]
    strict: bool,

    #[arg(long)]
    cli: bool, // Flag to force CLI mode
}

fn main() {
    let args = Args::parse();
    
    // If --cli flag is used OR query is provided, run CLI mode
    if args.cli || args.query.is_some() {
        // CLI mode - need a query
        let query = match args.query {
            Some(q) => q,
            None => {
                eprintln!("Error: --query required for CLI mode");
                std::process::exit(1);
            }
        };
        
        let directory = Path::new(&args.directory);
        let search_mode = if args.strict {
            search::SearchMode::And
        } else {
            search::SearchMode::Or
        };
        
        let file_paths = core::discover::find_all_source_files(&directory);
        let files = core::source::load_files(&file_paths);
        let comments = core::engine::extract_comments(&files);

        let results = search::search(&comments, &query, search_mode);

        println!("Search results for {}: {} matches", &query, results.len());
        for result in results {
            println!("{}:{}: {}", result.file_name, result.line, result.text.trim());
        }
    } else {
        // Default: TUI mode
        if let Err(e) = tui::run_tui() {
            eprintln!("TUI error: {}", e);
            std::process::exit(1);
        }
    }
}