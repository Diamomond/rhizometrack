pub mod categories;
pub mod categories_delete;
pub mod schema;
pub mod sessions;
pub mod sessions_delete;
pub mod settings;
pub mod stats;

use rusqlite::Connection;
use std::path::Path;

pub fn open_database(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;

    // Use the schema bundled at project root via the schema module
    conn.execute_batch(crate::db::schema::SCHEMA)?;

    // Migrate existing databases by adding any missing columns
    if let Err(e) =
        conn.execute_batch("ALTER TABLE sessions ADD COLUMN session_name TEXT NOT NULL DEFAULT '';")
    {
        let msg = e.to_string();
        if !msg.contains("duplicate column name") && !msg.contains("already exists") {
            return Err(e);
        }
    }

    Ok(conn)
}
