use std::sync::{Arc, Mutex};
use sync_protocol::{EncryptedBlob, ServerMessage};

pub type DbPool = Arc<Mutex<rusqlite::Connection>>;

pub fn init_pool(path: &str) -> std::result::Result<DbPool, Box<dyn std::error::Error>> {
    let conn = rusqlite::Connection::open(path)?;

    // Create the updates table if it doesn't exist.
    // doc_id is renamed to sync_id (the SHA-256 hash location).
    conn.execute(
        "CREATE TABLE IF NOT EXISTS updates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sync_id TEXT NOT NULL,
            client_id TEXT NOT NULL,
            data BLOB NOT NULL
        )",
        [],
    )?;

    // Optimization: Index for faster lookups by sync_id + id
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_updates_sync_id_id ON updates (sync_id, id)",
        [],
    )?;

    Ok(Arc::new(Mutex::new(conn)))
}

/// Appends an encrypted blob to the log.
pub fn append_update(
    pool: &DbPool,
    sync_id: &str,
    client_id: &str,
    blob: &EncryptedBlob,
) -> anyhow::Result<i64> {
    let conn = pool.lock().unwrap();
    let data_bytes = serde_json::to_vec(blob)?;

    conn.execute(
        "INSERT INTO updates (sync_id, client_id, data) VALUES (?1, ?2, ?3)",
        (sync_id, client_id, &data_bytes),
    )?;

    let id = conn.last_insert_rowid();
    Ok(id)
}

/// Retrieves changes since a given sequence ID for a specific sync room.
pub fn get_changes_since(
    pool: &DbPool,
    sync_id: &str,
    last_sequence: i64,
) -> anyhow::Result<Vec<ServerMessage>> {
    let conn = pool.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, client_id, data FROM updates WHERE sync_id = ?1 AND id > ?2 ORDER BY id ASC",
    )?;

    let rows = stmt.query_map((sync_id, last_sequence), |row| {
        let id: i64 = row.get(0)?;
        let client_id: String = row.get(1)?;
        let data: Vec<u8> = row.get(2)?;
        let blob: EncryptedBlob = serde_json::from_slice(&data).expect("Corrupt DB JSON");

        Ok(ServerMessage::ChangeOccurred {
            sequence_id: id,
            sync_id: sync_id.to_string(), // Redundant but consistent
            source_client_id: client_id,
            payload: blob,
        })
    })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}
