use chrono::Utc;
use rusqlite::{Connection, Result, params};

pub fn create(conn: &Connection, category_id: i64, session_name: Option<&str>) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    let session_name = session_name.unwrap_or("");

    conn.execute(
        r#"
        INSERT INTO sessions(
            category_id,
            session_name,
            started_at,
            ended_at,
            duration_seconds,
            note_markdown
        )
        VALUES(
            ?1,
            ?2,
            ?3,
            ?3,
            0,
            ''
        )
        "#,
        params![category_id, session_name, now],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn insert_record(
    conn: &Connection,
    category_id: i64,
    session_name: &str,
    started_at: &str,
    ended_at: &str,
    duration_seconds: i64,
    note_markdown: &str,
) -> Result<i64> {
    conn.execute(
        r#"
        INSERT INTO sessions(
            category_id,
            session_name,
            started_at,
            ended_at,
            duration_seconds,
            note_markdown
        )
        VALUES(?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        params![
            category_id,
            session_name,
            started_at,
            ended_at,
            duration_seconds,
            note_markdown,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn finish(
    conn: &Connection,
    session_id: i64,
    duration_seconds: i64,
    notes: &str,
    session_name: Option<&str>,
) -> Result<()> {
    let session_name = session_name.unwrap_or("");
    conn.execute(
        r#"
        UPDATE sessions
        SET
            ended_at=?2,
            duration_seconds=?3,
            note_markdown=?4,
            session_name=?5
        WHERE id=?1
        "#,
        params![
            session_id,
            Utc::now().to_rfc3339(),
            duration_seconds,
            notes,
            session_name,
        ],
    )?;

    Ok(())
}

pub fn update_notes(conn: &Connection, session_id: i64, notes: &str) -> Result<()> {
    conn.execute(
        r#"
        UPDATE sessions
        SET note_markdown = ?2
        WHERE id = ?1
        "#,
        params![session_id, notes],
    )?;

    Ok(())
}

pub fn all(conn: &Connection) -> Result<Vec<crate::models::Session>> {
    let mut stmt = conn.prepare(
        r#"SELECT id, category_id, session_name, started_at, ended_at, duration_seconds, note_markdown
           FROM sessions
           ORDER BY started_at DESC"#,
    )?;

    let sessions = stmt
        .query_map([], |row| {
            Ok(crate::models::Session {
                id: row.get(0)?,
                category_id: row.get(1)?,
                session_name: row.get(2)?,
                started_at: row.get(3)?,
                ended_at: row.get(4)?,
                duration_seconds: row.get(5)?,
                note_markdown: row.get(6)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();

    Ok(sessions)
}
