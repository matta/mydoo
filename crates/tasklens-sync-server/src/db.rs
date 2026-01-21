use std::sync::{Arc, Mutex};
use tasklens_sync_protocol::ServerMessage;

pub(crate) type DbPool = Arc<Mutex<rusqlite::Connection>>;

pub(crate) fn init_pool(path: &str) -> std::result::Result<DbPool, Box<dyn std::error::Error>> {
    let conn = rusqlite::Connection::open(path)?;

    // Create the updates table if it doesn't exist.
    // doc_id is renamed to discovery_key (the SHA-256 hash location).
    conn.execute(
        "CREATE TABLE IF NOT EXISTS updates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            discovery_key TEXT NOT NULL,
            client_id TEXT NOT NULL,
            data BLOB NOT NULL
        )",
        [],
    )?;

    // Optimization: Index for faster lookups by discovery_key + id
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_updates_discovery_key_id ON updates (discovery_key, id)",
        [],
    )?;

    Ok(Arc::new(Mutex::new(conn)))
}

/// Appends a raw payload blob to the log.
pub(crate) fn append_update(
    pool: &DbPool,
    discovery_key: &str,
    client_id: &str,
    payload: &[u8],
) -> anyhow::Result<i64> {
    let conn = pool.lock().unwrap();

    conn.execute(
        "INSERT INTO updates (discovery_key, client_id, data) VALUES (?1, ?2, ?3)",
        (discovery_key, client_id, payload),
    )?;

    let id = conn.last_insert_rowid();
    Ok(id)
}

/// Retrieves changes since a given sequence ID for a specific discovery key.
pub(crate) fn get_changes_since(
    pool: &DbPool,
    discovery_key: &str,
    last_sequence: i64,
) -> anyhow::Result<Vec<ServerMessage>> {
    let conn = pool.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, client_id, data FROM updates WHERE discovery_key = ?1 AND id > ?2 ORDER BY id ASC",
    )?;

    let rows = stmt.query_map((discovery_key, last_sequence), |row| {
        let id: i64 = row.get(0)?;
        let client_id: String = row.get(1)?;
        let data: Vec<u8> = row.get(2)?;
        // Payload is now just raw bytes, no internal structure assumed by server

        Ok(ServerMessage::ChangeOccurred {
            sequence_id: id,
            discovery_key: discovery_key.to_string(), // Redundant but consistent
            source_client_id: client_id,
            payload: data,
        })
    })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tasklens_sync_protocol::ServerMessage;

    #[test]
    fn test_append_and_retrieve_discovery_key() {
        let db_dir = tempfile::tempdir().unwrap();
        let db_path = db_dir.path().join("test.db");
        let pool = init_pool(db_path.to_str().unwrap()).unwrap();

        let discovery_key = "key_123";
        let client_id = "client_abc";
        let payload = vec![1, 2, 3, 4, 255]; // Arbitrary bytes

        let _ = append_update(&pool, discovery_key, client_id, &payload).unwrap();

        let changes = get_changes_since(&pool, discovery_key, 0).unwrap();
        assert_eq!(changes.len(), 1);

        match &changes[0] {
            ServerMessage::ChangeOccurred {
                discovery_key: key,
                source_client_id,
                payload: data,
                ..
            } => {
                assert_eq!(key, discovery_key);
                assert_eq!(source_client_id, client_id);
                assert_eq!(data, &payload);
            }
        }
    }
}
