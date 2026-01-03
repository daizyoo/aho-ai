use crate::core::{Board, Move, PlayerId};
use crate::network::protocol::NetMessage;
use std::sync::mpsc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

pub struct NetworkClient {
    stream: TcpStream,
}

impl NetworkClient {
    pub async fn connect(addr: &str) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self { stream })
    }

    pub async fn run(
        &mut self,
        _player_id_tx: mpsc::Sender<PlayerId>,
        _board_tx: mpsc::Sender<Board>,
        move_tx: mpsc::Sender<Move>,
    ) -> anyhow::Result<()> {
        let (reader, mut writer) = self.stream.split();
        let mut lines = BufReader::new(reader).lines();

        // 1. Join
        let join = NetMessage::Join {
            name: "Player".to_string(),
        };
        let join_json = serde_json::to_string(&join)? + "\n";
        writer.write_all(join_json.as_bytes()).await?;

        // 2. Message loop
        while let Some(line) = lines.next_line().await? {
            let msg: NetMessage = serde_json::from_str(&line)?;
            match msg {
                NetMessage::Welcome { player_id, board } => {
                    println!("Joined as {:?}", player_id);
                    // In a full impl, we'd send these to the game loop
                    let _ = _player_id_tx.send(player_id);
                    let _ = _board_tx.send(board);
                }
                NetMessage::MatchFound { opponent_name } => {
                    println!("Match found! Opponent: {}", opponent_name);
                }
                NetMessage::Update {
                    board,
                    last_move: _,
                    next_player: _,
                } => {
                    let _ = _board_tx.send(board);
                }
                NetMessage::MakeMove { mv } => {
                    let _ = move_tx.send(mv);
                }
                NetMessage::GameOver { winner, reason } => {
                    println!("Game Over! Winner: {:?}, Reason: {}", winner, reason);
                    break;
                }
                NetMessage::Error { message } => {
                    eprintln!("Server Error: {}", message);
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub async fn send_move(&mut self, mv: Move) -> anyhow::Result<()> {
        let msg = NetMessage::MakeMove { mv };
        let json = serde_json::to_string(&msg)? + "\n";
        self.stream.write_all(json.as_bytes()).await?;
        Ok(())
    }
}
