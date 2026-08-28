use rusqlite::{Connection, Result, params};

pub fn delete(conn: &Connection, category_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM sessions WHERE category_id = ?1",
        params![category_id],
    )?;
    conn.execute("DELETE FROM categories WHERE id = ?1", params![category_id])?;
    Ok(())
}
