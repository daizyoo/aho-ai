use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use crossterm::{execute, terminal};
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Lightweight kifu metadata for file selection
#[derive(Debug, Clone)]
pub struct KifuFileInfo {
    pub path: PathBuf,
    pub filename: String,
    pub board_setup: String,
    pub player1: String,
    pub player2: String,
    pub move_count: usize,
    pub timestamp: String,
}

/// Minimal structure for fast metadata parsing
#[derive(Deserialize)]
struct KifuMetadata {
    board_setup: String,
    player1_name: String,
    player2_name: String,
    moves: Vec<serde_json::Value>, // Don't parse full moves, just count
}

pub struct KifuSelector {
    files: Vec<KifuFileInfo>,
    selected_index: usize,
    scroll_offset: usize,
    visible_rows: usize,
}

impl KifuSelector {
    /// Scan directories for kifu files (recursively)
    pub fn scan_directories(dirs: &[PathBuf]) -> Result<Self> {
        let mut files = Vec::new();

        for dir in dirs {
            if !dir.exists() {
                continue;
            }

            // Recursively scan for .json files
            Self::scan_directory_recursive(dir, &mut files)?;
        }

        // Sort by timestamp (newest first)
        files.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(Self {
            files,
            selected_index: 0,
            scroll_offset: 0,
            visible_rows: 20,  // Increased from 10 to 20
        })
    }

    /// Try to extract timestamp from filename (e.g., "selfplay_results_20260107_214954.json")
    fn extract_timestamp_from_filename(filename: &str) -> Option<String> {
        if !filename.contains("_") {
            return None;
        }

        let parts: Vec<&str> = filename.split('_').collect();
        if parts.len() < 3 {
            return None;
        }

        let date_str = parts[parts.len() - 2];
        let time_str = parts[parts.len() - 1].trim_end_matches(".json");

        // Format: "20260107" -> "2026-01-07", "214954" -> "21:49:54"
        if date_str.len() == 8 && time_str.len() == 6 {
            Some(format!(
                "{}-{}-{} {}:{}:{}",
                &date_str[0..4],
                &date_str[4..6],
                &date_str[6..8],
                &time_str[0..2],
                &time_str[2..4],
                &time_str[4..6]
            ))
        } else if date_str.len() == 8 {
            // Date only
            Some(format!(
                "{}-{}-{}",
                &date_str[0..4],
                &date_str[4..6],
                &date_str[6..8]
            ))
        } else {
            Some(format!("{} {}", date_str, time_str))
        }
    }

    /// Try to extract timestamp from parent directory path (e.g., "selfplay_kifu/ShogiOnly/20260107_214954/")
    fn extract_timestamp_from_path(path: &Path) -> Option<String> {
        // Check parent directory for timestamp pattern
        if let Some(parent) = path.parent() {
            if let Some(dir_name) = parent.file_name().and_then(|n| n.to_str()) {
                // Try to extract from directory name
                if let Some(ts) = Self::extract_timestamp_from_filename(dir_name) {
                    return Some(ts);
                }

                // Check if it's just a date (YYYYMMDD)
                if dir_name.len() == 8 && dir_name.chars().all(|c| c.is_ascii_digit()) {
                    return Some(format!(
                        "{}-{}-{}",
                        &dir_name[0..4],
                        &dir_name[4..6],
                        &dir_name[6..8]
                    ));
                }

                // Check if it contains underscores (YYYYMMDD_HHMMSS)
                if dir_name.contains('_') {
                    let parts: Vec<&str> = dir_name.split('_').collect();
                    for part in parts {
                        if part.len() == 8 && part.chars().all(|c| c.is_ascii_digit()) {
                            // Found date part
                            return Some(format!(
                                "{}-{}-{}",
                                &part[0..4],
                                &part[4..6],
                                &part[6..8]
                            ));
                        }
                    }
                }
            }
        }
        None
    }

    /// Recursively scan a directory for JSON kifu files
    fn scan_directory_recursive(dir: &Path, files: &mut Vec<KifuFileInfo>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                Self::scan_directory_recursive(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Extract metadata from JSON files
                if let Ok(info) = Self::extract_metadata(&path) {
                    files.push(info);
                }
            }
        }
        Ok(())
    }

    /// Extract metadata without loading full moves
    fn extract_metadata(path: &Path) -> Result<KifuFileInfo> {
        let content = fs::read_to_string(path)?;
        let metadata: KifuMetadata = serde_json::from_str(&content)?;

        // Extract timestamp from filename (e.g., "selfplay_results_20260107_214954.json")
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let timestamp = if filename.contains("_") {
            // Extract date/time from filename
            let parts: Vec<&str> = filename.split('_').collect();
            if parts.len() >= 3 {
                let date_str = parts[parts.len() - 2];
                let time_str = parts[parts.len() - 1].trim_end_matches(".json");

                // Format: "20260107" -> "2026-01-07", "214954" -> "21:49:54"
                if date_str.len() == 8 && time_str.len() == 6 {
                    format!(
                        "{}-{}-{} {}:{}:{}",
                        &date_str[0..4],
                        &date_str[4..6],
                        &date_str[6..8],
                        &time_str[0..2],
                        &time_str[2..4],
                        &time_str[4..6]
                    )
                } else {
                    format!("{} {}", date_str, time_str)
                }
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        };

        Ok(KifuFileInfo {
            path: path.to_path_buf(),
            filename,
            board_setup: metadata.board_setup,
            player1: metadata.player1_name,
            player2: metadata.player2_name,
            move_count: metadata.moves.len(),
            timestamp,
        })
    }

    /// Render the file selection UI
    pub fn render(&self) -> Result<()> {
        execute!(
            io::stdout(),
            terminal::Clear(terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;

        println!("╔══════════════════════════════════════════════════════════════════════════════════════════╗\r");
        println!("║                              Kifu Replay - Select File                                     ║\r");
        println!("╠══════════════════════════════════════════════════════════════════════════════════════════╣\r");
        println!("║                                                                                            ║\r");

        if self.files.is_empty() {
            println!("║  No kifu files found in:                                                               ║\r");
            println!("║    - kifu/                                                                             ║\r");
            println!("║    - selfplay_results/                                                                 ║\r");
            println!("║    - selfplay_kifu/                                                                    ║\r");
            println!("║                                                                                            ║\r");
            println!("║  Generate some games first!                                                            ║\r");
            println!("║                                                                                            ║\r");
        } else {
            // Calculate visible range
            let start = self.scroll_offset;
            let end = (start + self.visible_rows).min(self.files.len());

            for i in start..end {
                let file = &self.files[i];
                let cursor = if i == self.selected_index { "▶" } else { " " };

                // Format: "▶ 2026-01-07 21:49:54 | ... "
                let display = format!(
                    "{} {} | {} | {} vs {} ({} moves)",
                    cursor,
                    file.timestamp,
                    truncate(&file.board_setup, 15),
                    truncate(&file.player1, 12),
                    truncate(&file.player2, 12),
                    file.move_count
                );

                println!("║  {:<90} ║\r", truncate(&display, 90));
            }

            // Fill remaining visible rows
            for _ in (end - start)..self.visible_rows {
                println!(
                    "║                                                                                            ║\r"
                );
            }
        }

        println!("║                                                                                            ║\r");
        println!("╠══════════════════════════════════════════════════════════════════════════════════════════╣\r");

        if self.files.is_empty() {
            println!("║  [q] Back to Main Menu                                                                     ║\r");
        } else {
            println!(
                "║  {} files | [↑/↓] Navigate | [Enter] Select | [q] Back                                  ║\r",
                self.files.len()
            );
        }

        println!("╚══════════════════════════════════════════════════════════════════════════════════════════╝\r");

        Ok(())
    }

    /// Handle user input, returns Some(PathBuf) if file selected, None if quit
    pub fn handle_input(&mut self) -> Result<Option<PathBuf>> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                    KeyCode::Enter => {
                        if !self.files.is_empty() {
                            return Ok(Some(self.files[self.selected_index].path.clone()));
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                            self.update_scroll();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if self.selected_index < self.files.len().saturating_sub(1) {
                            self.selected_index += 1;
                            self.update_scroll();
                        }
                    }
                    _ => {}
                }
            }
        }

        Err(anyhow::anyhow!("Continue"))
    }

    /// Update scroll offset to keep cursor visible
    fn update_scroll(&mut self) {
        // Keep cursor in view
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = self.selected_index - self.visible_rows + 1;
        }
    }

    /// Run the selector loop
    pub fn run(&mut self) -> Result<Option<PathBuf>> {
        loop {
            self.render()?;

            match self.handle_input() {
                Ok(Some(path)) => {
                    // Clear screen before returning
                    execute!(
                        std::io::stdout(),
                        terminal::Clear(terminal::ClearType::All),
                        crossterm::cursor::MoveTo(0, 0)
                    )?;
                    return Ok(Some(path));
                }
                Ok(None) => {
                    // Clear screen before returning
                    execute!(
                        std::io::stdout(),
                        terminal::Clear(terminal::ClearType::All),
                        crossterm::cursor::MoveTo(0, 0)
                    )?;
                    return Ok(None);
                }
                Err(_) => continue, // "Continue" error means keep looping
            }
        }
    }
}

/// Truncate string to max length (respects UTF-8 character boundaries)
fn truncate(s: &str, max_len: usize) -> String {
    let char_count: usize = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else {
        // Take max_len - 1 characters and append ellipsis
        let truncated: String = s.chars().take(max_len.saturating_sub(1)).collect();
        format!("{}…", truncated)
    }
}
