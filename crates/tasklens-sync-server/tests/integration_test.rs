use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tasklens_sync_protocol::{ClientMessage, EncryptedBlob, ServerMessage};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::protocol::Message,
};

/// Connects to the WebSocket server and performs a handshake.
/// Retries for up to 60 seconds to allow the server to start (and compile if needed).
/// Connects to the WebSocket server and performs a handshake.
async fn connect_client(
    port: u16,
    sync_id: &str,
    client_id: &str,
    last_seq: i64,
) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
    let url = format!("ws://127.0.0.1:{}/sync", port);

    let mut attempt = 0;
    let mut ws_stream = loop {
        match connect_async(&url).await {
            Ok((stream, _)) => break stream,
            Err(e) => {
                attempt += 1;
                if attempt > 120 {
                    // 60 seconds (500ms * 120)
                    panic!("Failed to connect to {} after retries: {}", url, e);
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    };

    // Send Hello
    let hello = ClientMessage::Hello {
        client_id: client_id.to_string(),
        sync_id: sync_id.to_string(),
        last_sequence: last_seq,
    };
    let json = serde_json::to_string(&hello).unwrap();
    ws_stream
        .send(Message::Text(json.into()))
        .await
        .expect("Failed to send Hello");

    ws_stream
}

fn get_free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

#[tokio::test]
async fn test_sync_flow() {
    let port = get_free_port();
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_updates.db");
    let db_url = db_path.to_str().expect("Failed to convert path to string");

    let mut server_process = tokio::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "tasklens-sync-server",
            "--",
            "-d",
            db_url,
            "-p",
            &port.to_string(),
        ])
        .current_dir("..") // Run from root workspace
        .kill_on_drop(true)
        .spawn()
        .expect("Failed to spawn server");

    let sync_id = "test_room_1";
    let client_a_id = "client_a";
    let client_b_id = "client_b";

    // 2. Client A connects
    let mut ws_a = connect_client(port, sync_id, client_a_id, 0).await;

    // 3. Client B connects
    let mut ws_b = connect_client(port, sync_id, client_b_id, 0).await;

    // 4. Client A pushes a change
    let blob = EncryptedBlob {
        nonce: [0u8; 24],
        ciphertext: vec![1, 2, 3, 4],
    };
    let push_msg = ClientMessage::SubmitChange {
        sync_id: sync_id.to_string(),
        payload: blob.clone(),
    };
    ws_a.send(Message::Text(
        serde_json::to_string(&push_msg).unwrap().into(),
    ))
    .await
    .expect("Failed to send push");

    // 5. Client B should receive the broadcast
    let msg = ws_b
        .next()
        .await
        .expect("Stream closed")
        .expect("Error receiving");
    let Message::Text(text) = msg else {
        panic!("Expected text message");
    };
    let server_msg: ServerMessage = serde_json::from_str(&text).unwrap();
    match server_msg {
        ServerMessage::ChangeOccurred {
            source_client_id,
            payload,
            ..
        } => {
            assert_eq!(source_client_id, client_a_id);
            assert_eq!(payload.ciphertext, vec![1, 2, 3, 4]);
        }
    }

    // 6. Persistence Check: Client C connects later
    let mut ws_c = connect_client(port, sync_id, "client_c", 0).await;
    let msg = ws_c
        .next()
        .await
        .expect("Stream closed")
        .expect("Error receiving");
    match msg {
        Message::Text(text) => {
            let server_msg: ServerMessage = serde_json::from_str(&text).unwrap();
            let ServerMessage::ChangeOccurred { payload, .. } = server_msg;
            assert_eq!(payload.ciphertext, vec![1, 2, 3, 4]);
        }
        _ => panic!("Expected text message"),
    }

    // Cleanup
    server_process.kill().await.unwrap();
}

#[tokio::test]
async fn test_room_isolation() {
    let port = get_free_port();
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_updates_iso.db");
    let db_url = db_path.to_str().expect("Failed to convert path to string");

    let mut server_process = tokio::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "tasklens-sync-server",
            "--",
            "-d",
            db_url,
            "-p",
            &port.to_string(),
        ])
        .current_dir("..")
        .kill_on_drop(true)
        .spawn()
        .expect("Failed to spawn server");

    let room_1 = "room_1";
    let room_2 = "room_2";

    // Client A in Room 1
    let mut ws_a = connect_client(port, room_1, "client_a", 0).await;

    // Client B in Room 2
    let mut ws_b = connect_client(port, room_2, "client_b", 0).await;

    // Client A pushes change to Room 1
    let blob = EncryptedBlob {
        nonce: [0u8; 24],
        ciphertext: vec![10, 20, 30],
    };
    let push_msg = ClientMessage::SubmitChange {
        sync_id: room_1.to_string(),
        payload: blob,
    };
    ws_a.send(Message::Text(
        serde_json::to_string(&push_msg).unwrap().into(),
    ))
    .await
    .expect("Failed to send push");

    // Client B should NOT receive anything (wait briefly to verify absence)
    // We use a timeout to assert no message arrives
    match tokio::time::timeout(Duration::from_millis(500), ws_b.next()).await {
        Ok(Some(msg)) => {
            panic!("Client B received message unexpectedly: {:?}", msg);
        }
        Ok(None) => {
            // Stream closed? unexpected but means no message
        }
        Err(_) => {
            // Timeout reached! This is good, means no message received.
        }
    }

    server_process.kill().await.unwrap();
}
