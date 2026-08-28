use rusqlite::{Connection, Result};
use std::collections::HashMap;

use crate::models::{Category, ExportPackage};

pub trait Repository {
    fn all_categories(&self) -> Result<Vec<Category>>;
    fn create_category(&self, name: &str, icon: Option<&str>) -> Result<i64>;
    fn create_session(&self, category_id: i64, session_name: Option<&str>) -> Result<i64>;
    fn finish_session(
        &self,
        session_id: i64,
        duration_seconds: i64,
        notes: &str,
        session_name: Option<&str>,
    ) -> Result<()>;
    fn insert_session_record(
        &self,
        category_id: i64,
        session_name: &str,
        started_at: &str,
        ended_at: &str,
        duration_seconds: i64,
        note_markdown: &str,
    ) -> Result<i64>;
    fn category_totals(&self) -> Result<Vec<crate::db::stats::CategoryStats>>;
    fn all_sessions(&self) -> Result<Vec<crate::models::Session>>;
    fn export_data(&self) -> Result<ExportPackage>;
    fn import_data(&self, data: ExportPackage) -> Result<()>;
    fn delete_category(&self, category_id: i64) -> Result<()>;
    fn delete_session(&self, session_id: i64) -> Result<()>;
    fn update_session_notes(&self, session_id: i64, notes: &str) -> Result<()>;

    // Theme persistence
    fn get_theme(&self) -> Result<Option<String>>;
    fn set_theme(&self, theme: &str) -> Result<()>;
}

pub struct SqliteRepository {
    conn: Connection,
}

impl SqliteRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

impl Repository for SqliteRepository {
    fn all_categories(&self) -> Result<Vec<Category>> {
        crate::db::categories::all(&self.conn)
    }

    fn create_category(&self, name: &str, icon: Option<&str>) -> Result<i64> {
        crate::db::categories::create(&self.conn, name, icon)
    }

    fn create_session(&self, category_id: i64, session_name: Option<&str>) -> Result<i64> {
        crate::db::sessions::create(&self.conn, category_id, session_name)
    }

    fn finish_session(
        &self,
        session_id: i64,
        duration_seconds: i64,
        notes: &str,
        session_name: Option<&str>,
    ) -> Result<()> {
        crate::db::sessions::finish(
            &self.conn,
            session_id,
            duration_seconds,
            notes,
            session_name,
        )
    }

    fn insert_session_record(
        &self,
        category_id: i64,
        session_name: &str,
        started_at: &str,
        ended_at: &str,
        duration_seconds: i64,
        note_markdown: &str,
    ) -> Result<i64> {
        crate::db::sessions::insert_record(
            &self.conn,
            category_id,
            session_name,
            started_at,
            ended_at,
            duration_seconds,
            note_markdown,
        )
    }

    fn export_data(&self) -> Result<ExportPackage> {
        Ok(ExportPackage {
            categories: self.all_categories()?,
            sessions: self.all_sessions()?,
        })
    }

    fn import_data(&self, data: ExportPackage) -> Result<()> {
        let existing = self.all_categories()?;
        let mut name_to_id = existing
            .into_iter()
            .map(|c| (c.name.clone(), c.id))
            .collect::<HashMap<_, _>>();

        let categories = data.categories;
        let mut old_to_new_id = HashMap::new();
        for category in categories.iter() {
            let new_id = if let Some(&existing_id) = name_to_id.get(&category.name) {
                existing_id
            } else {
                let created_id = self.create_category(&category.name, category.icon.as_deref())?;
                name_to_id.insert(category.name.clone(), created_id);
                created_id
            };
            old_to_new_id.insert(category.id, new_id);
        }

        for session in data.sessions.into_iter() {
            let category_id = old_to_new_id
                .get(&session.category_id)
                .copied()
                .unwrap_or(session.category_id);
            let _ = self.insert_session_record(
                category_id,
                &session.session_name,
                &session.started_at,
                &session.ended_at,
                session.duration_seconds,
                &session.note_markdown,
            )?;
        }

        Ok(())
    }

    fn category_totals(&self) -> Result<Vec<crate::db::stats::CategoryStats>> {
        crate::db::stats::category_totals(&self.conn)
    }

    fn all_sessions(&self) -> Result<Vec<crate::models::Session>> {
        crate::db::sessions::all(&self.conn)
    }

    fn delete_category(&self, category_id: i64) -> Result<()> {
        crate::db::categories_delete::delete(&self.conn, category_id)
    }

    fn delete_session(&self, session_id: i64) -> Result<()> {
        crate::db::sessions_delete::delete(&self.conn, session_id)
    }

    fn update_session_notes(&self, session_id: i64, notes: &str) -> Result<()> {
        crate::db::sessions::update_notes(&self.conn, session_id, notes)
    }

    fn get_theme(&self) -> Result<Option<String>> {
        crate::db::settings::get(&self.conn, "theme")
    }

    fn set_theme(&self, theme: &str) -> Result<()> {
        crate::db::settings::set(&self.conn, "theme", theme)
    }
}
