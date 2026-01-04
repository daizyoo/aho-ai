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
        let sanitized = Self::sanitize_addr(addr);
        // 10秒のタイムアウトを設定
        let connect_fut = TcpStream::connect(&sanitized);
        let stream = tokio::time::timeout(std::time::Duration::from_secs(10), connect_fut)
            .await
            .map_err(|_| anyhow::anyhow!("Connection timed out: {}", sanitized))??;

        Ok(Self { stream })
    }

    pub fn sanitize_addr(addr: &str) -> String {
        let mut s = addr.trim().to_string();

        // プロトコルスキームの除去 (xxx://)
        if let Some(pos) = s.find("://") {
            s = s[(pos + 3)..].to_string();
        }

        // 末尾のパスやスラッシュを除去
        if let Some(pos) = s.find('/') {
            s.truncate(pos);
        }

        // ポート番号の自動付与 (IPv6を考慮せず、単純に ':' が無ければ付与)
        if !s.contains(':') {
            s.push_str(":8080");
        }
        s
    }

    pub async fn run(
        &mut self,
        player_id_tx: mpsc::Sender<PlayerId>,
        board_tx: mpsc::Sender<(Board, PlayerId)>,
        remote_move_tx: mpsc::Sender<Move>,
        mut local_move_rx: tokio_mpsc::UnboundedReceiver<Move>,
    ) -> anyhow::Result<()> {
        let (reader, mut writer) = self.stream.split();
        let mut lines = BufReader::new(reader).lines();
        let mut my_id: Option<PlayerId> = None;

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
                            my_id = Some(player_id);
                            let _ = player_id_tx.send(player_id);
                            let _ = board_tx.send((board, PlayerId::Player1));
                        }
                        NetMessage::MatchFound { opponent_name: _ } => {
                        }
                        NetMessage::Update { board, last_move, next_player } => {
                            // 相手の指し手のみをUI表示用に流す
                            // (自分が指した直後のUpdateでは next_player != me なのでスルーされる)
                            if let Some(mv) = last_move {
                                if let Some(me) = my_id {
                                    if next_player == me {
                                        let _ = remote_move_tx.send(mv);
                                    }
                                }
                            }
                            let _ = board_tx.send((board, next_player));
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
