use rusqlite::{Connection, Result};
use std::path::Path;
use std::sync::{Arc, Mutex};

mod chat;
mod design;
mod media;
mod schema;
mod seed;
mod settings;
mod slides;
mod songs;
mod themes;
pub mod transfer;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct AppDb {
    conn: Arc<Mutex<Connection>>,
}

impl AppDb {
    /// Opens or creates the SQLite database at the given path
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        db.seed_defaults_if_empty()?;
        db.load_persisted_fonts();
        Ok(db)
    }

    /// Opens an in-memory database (useful for tests)
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        db.seed_defaults_if_empty()?;
        Ok(db)
    }

    /// Retorna o diretório persistente para fontes customizadas
    pub fn fonts_directory(&self) -> std::result::Result<std::path::PathBuf, String> {
        let path: String = self
            .conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT file FROM pragma_database_list WHERE name='main'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let parent = if path.is_empty() {
            std::env::current_dir().map_err(|e| e.to_string())?
        } else {
            std::path::Path::new(&path)
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf()
        };
        let dir = parent.join("fonts");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(dir)
    }

    /// Carrega fontes customizadas presentes no diretório de fontes
    pub fn load_persisted_fonts(&self) {
        if let Ok(dir) = self.fonts_directory() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                        if ext.eq_ignore_ascii_case("ttf") || ext.eq_ignore_ascii_case("otf") {
                            let _ = crate::core::models::fonts::load_custom_font_file(&path);
                        }
                    }
                }
            }
        }
    }
}
