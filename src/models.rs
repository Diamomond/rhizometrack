use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: i64,
    pub category_id: i64,
    pub session_name: String,
    pub started_at: String,
    pub ended_at: String,
    pub duration_seconds: i64,
    pub note_markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPackage {
    pub categories: Vec<Category>,
    pub sessions: Vec<Session>,
}
