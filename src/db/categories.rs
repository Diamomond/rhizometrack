use crate::models::Category;
use chrono::Utc;
use rusqlite::{Connection, Result, params};

pub fn create(conn: &Connection, name: &str, icon: Option<&str>) -> Result<i64> {
    conn.execute(
        r#"
        INSERT INTO categories(
            name,
            icon,
            created_at
        )
        VALUES(?1, ?2, ?3)
        "#,
        params![name, icon, Utc::now().to_rfc3339()],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn all(conn: &Connection) -> Result<Vec<Category>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            id,
            name,
            icon,
            created_at
        FROM categories
        ORDER BY name
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            icon: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    rows.collect()
}
