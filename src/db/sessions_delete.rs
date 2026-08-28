use rusqlite::{Connection, Result, params};

pub fn delete(conn: &Connection, session_id: i64) -> Result<()> {
    conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])?;
    Ok(())
}
