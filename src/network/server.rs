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
        relay_game(socket1, socket).await?;
    } else {
        *lock = Some((socket, player_name));
    }
    Ok(())
}

async fn relay_game(mut s1: TcpStream, mut s2: TcpStream) -> anyhow::Result<()> {
    let (r1, mut w1) = s1.split();
    let (r2, mut w2) = s2.split();

    let f1 = async {
        let mut lines = BufReader::new(r1).lines();
        while let Some(line) = lines.next_line().await? {
            w2.write_all((line + "\n").as_bytes()).await?;
        }
        Ok::<(), anyhow::Error>(())
    };

    let f2 = async {
        let mut lines = BufReader::new(r2).lines();
        while let Some(line) = lines.next_line().await? {
            w1.write_all((line + "\n").as_bytes()).await?;
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
