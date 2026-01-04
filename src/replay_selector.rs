use crate::game::{replay::ReplayViewer, KifuData};
use crossterm::event::{self, Event, KeyCode};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

pub async fn run_replay_selector() -> anyhow::Result<()> {
    use crossterm::{execute, terminal};

    execute!(
        std::io::stdout(),
        terminal::Clear(terminal::ClearType::All),
        crossterm::cursor::MoveTo(0, 0)
    )?;

    print!("=== Self-Play Kifu Replay ===\r\n\r\n");

    // List available kifu files
    let kifu_dir = PathBuf::from("selfplay_kifu");

    if !kifu_dir.exists() {
        print!("Error: selfplay_kifu/ directory not found\r\n");
        print!("Run self-play first to generate kifu files\r\n\r\n");
        print!("Press any key to return to menu...\r\n");
        loop {
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(_) = event::read()? {
                    break;
                }
            }
        }
        return Ok(());
    }

    let mut kifu_files: Vec<PathBuf> = fs::read_dir(&kifu_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .collect();

    kifu_files.sort();

    if kifu_files.is_empty() {
        print!("No kifu files found in selfplay_kifu/\r\n");
        print!("Run self-play first to generate kifu files\r\n\r\n");
        print!("Press any key to return to menu...\r\n");
        loop {
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(_) = event::read()? {
                    break;
                }
            }
        }
        return Ok(());
    }

    print!("Available kifu files:\r\n\r\n");
    for (i, path) in kifu_files.iter().enumerate() {
        let filename = path.file_name().unwrap().to_str().unwrap();
        print!("{}. {}\r\n", i + 1, filename);
    }

    print!("\r\nSelect file (1-{}) or [q] to quit: ", kifu_files.len());
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut input = String::new();
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Enter => break,
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        input.push(c);
                        print!("{}", c);
                        std::io::Write::flush(&mut std::io::stdout())?;
                    }
                    KeyCode::Backspace => {
                        if !input.is_empty() {
                            input.pop();
                            print!("\u{0008} \u{0008}");
                            std::io::Write::flush(&mut std::io::stdout())?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if input.is_empty() {
        return Ok(());
    }

    let selection: usize = input.parse().unwrap_or(0);
    if selection == 0 || selection > kifu_files.len() {
        print!("\r\nInvalid selection\r\n");
        return Ok(());
    }

    let selected_file = &kifu_files[selection - 1];
    print!("\r\n\r\nLoading {}...\r\n", selected_file.display());

    // Load kifu file
    let kifu_json = fs::read_to_string(selected_file)?;
    let kifu_data: KifuData = serde_json::from_str(&kifu_json)?;

    print!("Loaded {} moves\r\n", kifu_data.moves.len());
    print!("Starting replay...\r\n\r\n");

    std::thread::sleep(Duration::from_millis(1000));

    // Start replay
    let mut viewer = ReplayViewer::new(kifu_data);
    viewer.run()?;

    Ok(())
}
