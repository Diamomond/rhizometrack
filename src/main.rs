mod app;
mod db;
mod models;
mod repository;
mod xp;

use std::path::PathBuf;

fn main() {
    let db_path = get_database_path();
    let conn = match crate::db::open_database(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open database {}: {}", db_path.display(), e);
            std::process::exit(1);
        }
    };
    let repo = crate::repository::SqliteRepository::new(conn);
    if let Err(e) = crate::app::run(repo) {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
}

fn get_database_path() -> PathBuf {
    let base_dir = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            if cfg!(target_os = "macos") {
                std::env::var_os("HOME")
                    .map(|home| PathBuf::from(home).join("Library/Application Support"))
            } else {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            }
        })
        .unwrap_or_else(|| PathBuf::from("."));

    let app_dir = base_dir.join("rhizometrack");
    if let Err(err) = std::fs::create_dir_all(&app_dir) {
        eprintln!(
            "Could not create data directory {}: {}",
            app_dir.display(),
            err
        );
    }

    app_dir.join("rhizometrack.db")
}
