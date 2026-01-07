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
    /// Scan directories for kifu files
    pub fn scan_directories(dirs: &[PathBuf]) -> Result<Self> {
        let mut files = Vec::new();

        for dir in dirs {
            if !dir.exists() {
                continue;
            }

            // Scan for .json files
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(info) = Self::extract_metadata(&path) {
                        files.push(info);
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        files.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(Self {
            files,
            selected_index: 0,
            scroll_offset: 0,
            visible_rows: 10,
        })
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
                format!(
                    "{} {}",
                    parts[parts.len() - 2],
                    parts[parts.len() - 1].trim_end_matches(".json")
                )
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

        println!("╔══════════════════════════════════════════════════════════════════╗\r");
        println!("║                   Kifu Replay - Select File                       ║\r");
        println!("╠══════════════════════════════════════════════════════════════════╣\r");
        println!("║                                                                    ║\r");

        if self.files.is_empty() {
            println!("║  No kifu files found in:                                          ║\r");
            println!("║    - kifu/                                                        ║\r");
            println!("║    - selfplay_results/                                            ║\r");
            println!("║                                                                    ║\r");
            println!("║  Generate some games first!                                       ║\r");
            println!("║                                                                    ║\r");
        } else {
            // Calculate visible range
            let start = self.scroll_offset;
            let end = (start + self.visible_rows).min(self.files.len());

            for i in start..end {
                let file = &self.files[i];
                let cursor = if i == self.selected_index { "▶" } else { " " };

                // Format: "▶ 20260107 214954 | ShogiOnly | P1 vs P2 | (64 moves)"
                let display = format!(
                    "{} {} | {:12} | {} vs {} ({} moves)",
                    cursor,
                    file.timestamp,
                    file.board_setup,
                    truncate(&file.player1, 10),
                    truncate(&file.player2, 10),
                    file.move_count
                );

                println!("║  {:<66} ║\r", truncate(&display, 66));
            }

            // Fill remaining visible rows
            for _ in (end - start)..self.visible_rows {
                println!(
                    "║                                                                    ║\r"
                );
            }
        }

        println!("║                                                                    ║\r");
        println!("╠══════════════════════════════════════════════════════════════════╣\r");

        if self.files.is_empty() {
            println!("║  [q] Back to Main Menu                                            ║\r");
        } else {
            println!(
                "║  {} files | [↑/↓] Navigate | [Enter] Select | [q] Back          ║\r",
                self.files.len()
            );
        }

        println!("╚══════════════════════════════════════════════════════════════════╝\r");

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
                Ok(Some(path)) => return Ok(Some(path)),
                Ok(None) => return Ok(None),
                Err(_) => continue, // "Continue" error means keep looping
            }
        }
    }
}

/// Truncate string to max length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
