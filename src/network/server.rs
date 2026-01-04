use crate::core::PlayerId;
use crate::network::protocol::NetMessage;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub async fn start_server(addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Server started on {}", addr);

    let waiting_room: Arc<Mutex<Option<(TcpStream, String)>>> = Arc::new(Mutex::new(None));

    loop {
        let (socket, _) = listener.accept().await?;
        let waiting_room = Arc::clone(&waiting_room);

        tokio::spawn(async move {
            if let Err(e) = handle_new_connection(socket, waiting_room).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}

async fn handle_new_connection(
    mut socket: TcpStream,
    waiting_room: Arc<Mutex<Option<(TcpStream, String)>>>,
) -> anyhow::Result<()> {
    let mut reader = BufReader::new(&mut socket);
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    let msg: NetMessage = serde_json::from_str(&line)?;
    let player_name = if let NetMessage::Join { name } = msg {
        name
    } else {
        return Err(anyhow::anyhow!("Expected Join message"));
    };

    let mut lock = waiting_room.lock().await;
    if let Some((mut socket1, name1)) = lock.take() {
        drop(lock); // Release lock as soon as possible
        println!("Match found: {} vs {}", name1, player_name);

        // Setup game
        let board = crate::core::setup::setup_from_strings(
            &crate::core::setup::get_standard_mixed_setup(),
            true,
            true,
        );

        // Notify players
        send_msg(
            &mut socket1,
            &NetMessage::Welcome {
                player_id: PlayerId::Player1,
                board: board.clone(),
            },
        )
        .await?;
        send_msg(
            &mut socket1,
            &NetMessage::MatchFound {
                opponent_name: player_name.clone(),
            },
        )
        .await?;

        send_msg(
            &mut socket,
            &NetMessage::Welcome {
                player_id: PlayerId::Player2,
                board: board.clone(),
            },
        )
        .await?;
        send_msg(
            &mut socket,
            &NetMessage::MatchFound {
                opponent_name: name1.clone(),
            },
        )
        .await?;

        // Relay loop
        relay_game(socket1, socket, board).await?;
    } else {
        *lock = Some((socket, player_name));
    }
    Ok(())
}

async fn relay_game(
    mut s1: TcpStream,
    mut s2: TcpStream,
    board: crate::core::Board,
) -> anyhow::Result<()> {
    let (r1, w1) = s1.split();
    let (r2, w2) = s2.split();

    let board_state = Arc::new(Mutex::new(board));
    let next_player = Arc::new(Mutex::new(crate::core::PlayerId::Player1));

    let w1 = Arc::new(Mutex::new(w1));
    let w2 = Arc::new(Mutex::new(w2));

    let b1 = Arc::clone(&board_state);
    let p1 = Arc::clone(&next_player);
    let out1_1 = Arc::clone(&w1);
    let out2_1 = Arc::clone(&w2);

    let f1 = async move {
        let mut lines = BufReader::new(r1).lines();
        while let Some(line) = lines.next_line().await? {
            let msg: NetMessage = serde_json::from_str(&line)?;
            if let NetMessage::MakeMove { mv } = msg {
                let mut b = b1.lock().await;
                let mut p = p1.lock().await;
                if *p == crate::core::PlayerId::Player1 {
                    let moves = crate::logic::legal_moves(&b, *p);
                    if moves.contains(&mv) {
                        *b = crate::logic::apply_move(&b, &mv, *p);
                        *p = p.opponent();
                        let update = NetMessage::Update {
                            board: b.clone(),
                            last_move: Some(mv),
                            next_player: *p,
                        };
                        let json = serde_json::to_string(&update)? + "\n";
                        out1_1.lock().await.write_all(json.as_bytes()).await?;
                        out2_1.lock().await.write_all(json.as_bytes()).await?;
                    } else {
                        let err = NetMessage::Error {
                            message: "Illegal move".to_string(),
                        };
                        let json = serde_json::to_string(&err)? + "\n";
                        out1_1.lock().await.write_all(json.as_bytes()).await?;
                    }
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    };

    let b2 = Arc::clone(&board_state);
    let p2 = Arc::clone(&next_player);
    let out1_2 = Arc::clone(&w1);
    let out2_2 = Arc::clone(&w2);

    let f2 = async move {
        let mut lines = BufReader::new(r2).lines();
        while let Some(line) = lines.next_line().await? {
            let msg: NetMessage = serde_json::from_str(&line)?;
            if let NetMessage::MakeMove { mv } = msg {
                let mut b = b2.lock().await;
                let mut p = p2.lock().await;
                if *p == crate::core::PlayerId::Player2 {
                    let moves = crate::logic::legal_moves(&b, *p);
                    if moves.contains(&mv) {
                        *b = crate::logic::apply_move(&b, &mv, *p);
                        *p = p.opponent();
                        let update = NetMessage::Update {
                            board: b.clone(),
                            last_move: Some(mv),
                            next_player: *p,
                        };
                        let json = serde_json::to_string(&update)? + "\n";
                        out1_2.lock().await.write_all(json.as_bytes()).await?;
                        out2_2.lock().await.write_all(json.as_bytes()).await?;
                    } else {
                        let err = NetMessage::Error {
                            message: "Illegal move".to_string(),
                        };
                        let json = serde_json::to_string(&err)? + "\n";
                        out2_2.lock().await.write_all(json.as_bytes()).await?;
                    }
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    };

    tokio::select! {
        res = f1 => res,
        res = f2 => res,
    }
}

async fn send_msg(socket: &mut TcpStream, msg: &NetMessage) -> anyhow::Result<()> {
    let json = serde_json::to_string(msg)? + "\n";
    socket.write_all(json.as_bytes()).await?;
    Ok(())
}
