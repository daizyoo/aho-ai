use crossterm::{
    event::{self, Event, KeyCode},
    execute, terminal,
};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

pub mod selection;

pub async fn read_input_raw(default: &str, prompt: &str) -> anyhow::Result<String> {
    print!("{}: (Default: {}) \r\n> ", prompt, default);
    io::stdout().flush()?;

    let mut input = String::new();
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => {
                        print!("\r\n");
                        if input.is_empty() {
                            return Ok(default.to_string());
                        } else {
                            return Ok(input);
                        }
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                        print!("{}", c);
                        io::stdout().flush()?;
                    }
                    KeyCode::Backspace => {
                        if !input.is_empty() {
                            input.pop();
                            print!("\u{0008} \u{0008}");
                            io::stdout().flush()?;
                        }
                    }
                    KeyCode::Esc => {
                        return Err(anyhow::anyhow!("Canceled"));
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn select_kifu_file(dir: &str) -> anyhow::Result<Option<PathBuf>> {
    use std::fs;

    // Collect files from both directories with labels
    let mut files_with_labels: Vec<(String, PathBuf)> = Vec::new();

    // Add regular kifu files
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    files_with_labels.push((format!("[Game] {}", name), path));
                }
            }
        }
    }

    // Add self-play kifu files (support both subdirectories and legacy flat structure)
    let selfplay_dir = "selfplay_kifu";
    if let Ok(entries) = fs::read_dir(selfplay_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                // Search in board-type subdirectories
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.filter_map(|e| e.ok()) {
                        let sub_path = sub_entry.path();
                        if sub_path.extension().and_then(|s| s.to_str()) == Some("json") {
                            if let Some(board_type) = path.file_name().and_then(|s| s.to_str()) {
                                if let Some(name) = sub_path.file_name().and_then(|s| s.to_str()) {
                                    files_with_labels
                                        .push((format!("[{}] {}", board_type, name), sub_path));
                                }
                            }
                        }
                    }
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Legacy files in root directory
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    files_with_labels.push((format!("[AI vs AI] {}", name), path));
                }
            }
        }
    }

    // Sort by modified time (descending)
    files_with_labels.sort_by_key(|(_, p)| {
        fs::metadata(p)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    files_with_labels.reverse();

    if files_with_labels.is_empty() {
        println!("No kifu files found. Press any key to return.");
        loop {
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(_) = event::read()? {
                    break;
                }
            }
        }
        return Ok(None);
    }

    let mut selected_index = 0;

    loop {
        execute!(
            io::stdout(),
            terminal::Clear(terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;
        print!("Select Kifu to Replay (↑/↓ or j/k / Enter / q):\r\n");
        print!("------------------------------------------------\r\n");

        for (i, (label, _)) in files_with_labels.iter().enumerate() {
            if i == selected_index {
                print!("> {}\r\n", label);
            } else {
                print!("  {}\r\n", label);
            }
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        selected_index = selected_index.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if selected_index < files_with_labels.len() - 1 {
                            selected_index += 1;
                        }
                    }
                    KeyCode::Enter => {
                        return Ok(Some(files_with_labels[selected_index].1.clone()));
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(None);
                    }
                    _ => {}
                }
            }
        }
    }
}
