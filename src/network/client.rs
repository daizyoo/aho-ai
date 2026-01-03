use crate::core::{Board, Move, PlayerId};
use crate::network::protocol::NetMessage;
use std::sync::mpsc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc as tokio_mpsc;

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
        player_id_tx: mpsc::Sender<PlayerId>,
        board_tx: mpsc::Sender<Board>,
        remote_move_tx: mpsc::Sender<Move>,
        mut local_move_rx: tokio_mpsc::UnboundedReceiver<Move>,
    ) -> anyhow::Result<()> {
        let (reader, mut writer) = self.stream.split();
        let mut lines = BufReader::new(reader).lines();

        // 1. Join
        let join = NetMessage::Join {
            name: "Player".to_string(),
        };
        let join_json = serde_json::to_string(&join)? + "\n";
        writer.write_all(join_json.as_bytes()).await?;

        // 2. Relay loop
        loop {
            tokio::select! {
                // Incoming from network
                line_res = lines.next_line() => {
                    let line = line_res?.ok_or_else(|| anyhow::anyhow!("Connection closed by server"))?;
                    let msg: NetMessage = serde_json::from_str(&line)?;
                    match msg {
                        NetMessage::Welcome { player_id, board } => {
                            let _ = player_id_tx.send(player_id);
                            let _ = board_tx.send(board);
                        }
                        NetMessage::MatchFound { opponent_name: _ } => {
                        }
                        NetMessage::Update { board, last_move: _, next_player: _ } => {
                            let _ = board_tx.send(board);
                        }
                        NetMessage::MakeMove { mv } => {
                            let _ = remote_move_tx.send(mv);
                        }
                        NetMessage::GameOver { winner: _, reason: _ } => {
                            break;
                        }
                        NetMessage::Error { message } => {
                            eprintln!("Server Error: {}", message);
                        }
                        _ => {}
                    }
                }
                // Outgoing to network
                local_mv_opt = local_move_rx.recv() => {
                    if let Some(mv) = local_mv_opt {
                        let msg = NetMessage::MakeMove { mv };
                        let json = serde_json::to_string(&msg)? + "\n";
                        writer.write_all(json.as_bytes()).await?;
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
