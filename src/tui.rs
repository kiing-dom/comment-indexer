use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::Frame;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    style::{Style, Color},
    Terminal,
};
use crate::core::engine::Comment;
use crate::core::search::SearchMode;

pub struct TuiState<'a> {
    query: String,
    results: Vec<&'a Comment<'a>>,
    selected: usize,
    should_quit: bool,
    strict_mode: bool,
}

pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    // setup
    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run_tui_loop(&mut terminal);

    disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run_tui_loop(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
   let directory = std::path::Path::new(".");
   let file_paths = crate::core::discover::find_all_source_files(&directory);
   let files = crate::core::source::load_files(&file_paths);
   let comments = crate::core::engine::extract_comments(&files);

   let mut state = TuiState {
        query: String::new(),
        results: Vec::new(),
        selected: 0,
        should_quit: false,
        strict_mode: false,
   };

   while !state.should_quit {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_key_event(& mut state, &comments, key);
            }
        }

        terminal.draw(|frame| {
            render_ui(frame, &state);
        })?;
   }
   
   Ok(())
}

fn handle_key_event<'a>(state: &mut TuiState<'a>, comments: &'a [Comment<'a>], key: KeyEvent) {
    // Only handle key press events, not release events
    if key.kind != KeyEventKind::Press {
        return;
    }
    
    match key.code {
        KeyCode::Esc => {
            state.should_quit = true;
        }
        KeyCode::Tab => {
            state.strict_mode = !state.strict_mode;
            update_search_results(state, comments);
        }
        KeyCode::Enter => {
            if let Some(comment) = state.results.get(state.selected) {
                open_in_editor(comment.file_path, comment.line);
            }
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.should_quit = true;
        }
        KeyCode::Backspace => {
            state.query.pop();
            update_search_results(state, comments);
        }
        KeyCode::Up => {
            if state.selected > 0 {
                state.selected -= 1;
            }
        }
        KeyCode::Down => {
            if state.selected < state.results.len().saturating_sub(1) {
                state.selected += 1;
            }
        }
        KeyCode::Char(c) => {
            state.query.push(c);
            update_search_results(state, comments);
        }
        _ => {}
    }
}

fn update_search_results<'a>(state: &mut TuiState<'a>, comments: &'a [Comment<'a>]) {
    let search_mode = if state.strict_mode {
        SearchMode::And 
    } else {
        SearchMode::Or
    };

    if state.query.is_empty() {
        state.results.clear();
    } else {
        state.results = crate::core::search::search(
            comments,
            &state.query,
            search_mode,
        );
    }

    state.selected = 0;
}

fn render_ui(frame: &mut Frame, state: &TuiState) {
    // splitting screen into 3 sections: input results, status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // search input box
            Constraint::Min(0), // results (takes up remaining space)
            Constraint::Length(1), // status line
        ])
        .split(frame.area());

    // render search input
    render_search_input(frame, chunks[0], state);
    // render Results
    render_results_list(frame, chunks[1], state);
    // render status line
    render_status_line(frame, chunks[2], state);
}

fn render_search_input(frame: &mut Frame, area: Rect, state: &TuiState) {
    let input = Paragraph::new(state.query.as_str())
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Search"));
        frame.render_widget(input, area);
}

fn render_results_list(frame: &mut Frame, area: Rect, state: &TuiState) {
    let items: Vec<ListItem> = state.results
        .iter()
        .map(|comment| {
            ListItem::new(format!("{}:{}: {}",
            comment.file_name,
            comment.line,
            comment.text.trim()))
        })
        .collect();

    let results_list = List::new(items)
        .block(Block::default()
        .borders(Borders::ALL)
        .title("Results"))
        .highlight_style(Style::default().bg(Color::Yellow));

    frame.render_stateful_widget(results_list, area, &mut ListState::default().with_selected(Some(state.selected)));
}

fn render_status_line(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, state: &TuiState) {
    let status = format!("{} matches | Tab: toggle strict | Enter: open in $EDITOR", state.results.len());
    let status_paragraph = Paragraph::new(status);
    frame.render_widget(status_paragraph, area);
}

fn open_in_editor(path: &std::path::PathBuf, line: usize) {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    let mut cmd = std::process::Command::new(&editor);

    // most editors support +<line> to jump to a line but vs code uses --goto file:line
    match editor.as_str() {
        "code" | "code-insiders" => { cmd.arg("--goto").arg(format!("{}:{}", path.display(), line)); }
        _ => { cmd.arg(format!("+{}", line)).arg(path); }
    };

    // leave TUI mode before switching to editor
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::ExecutableCommand::execute(
        &mut std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen
    );
    
    let _ = cmd.status();

    // restore TUI after editor exits
    let _ = crossterm::terminal::enable_raw_mode();
    let _ = crossterm::ExecutableCommand::execute(
        &mut std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen
    );
}