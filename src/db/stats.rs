use rusqlite::{Connection, Result};

pub struct CategoryStats {
    pub category_id: i64,
    pub total_seconds: i64,
}

pub fn category_totals(conn: &Connection) -> Result<Vec<CategoryStats>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            category_id,
            SUM(duration_seconds)
        FROM sessions
        GROUP BY category_id
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(CategoryStats {
            category_id: row.get(0)?,
            total_seconds: row.get(1)?,
        })
    })?;

    rows.collect()
}
