use crate::core::{
    design::DesignDocument,
    models::{CustomTextItem, PlaylistItem, PlaylistItemKind, PresentationTheme, Song, ThemeRecord},
    state::ControlPanelState,
};
use crate::services::db::AppDb;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

pub const EXT_SONG: &str = "opsong";
pub const EXT_TEXT: &str = "optext";
pub const EXT_DESIGN: &str = "opdesign";
pub const EXT_THEME: &str = "optheme";
pub const EXT_PLAYLIST: &str = "opplaylist";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SongPackage {
    pub format: String,
    pub version: u32,
    pub title: String,
    pub artist: String,
    pub text: String,
    #[serde(default)]
    pub tags: String,
    #[serde(default)]
    pub folder: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextPackage {
    pub format: String,
    pub version: u32,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub folder: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DesignPackage {
    pub format: String,
    pub version: u32,
    pub scene: DesignDocument,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemePackage {
    pub format: String,
    pub version: u32,
    pub theme: crate::core::models::ThemeRecord,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaylistPackage {
    pub format: String,
    pub version: u32,
    pub name: String,
    pub items: Vec<PlaylistItem>,
    #[serde(default)]
    pub songs: Vec<Song>,
    #[serde(default)]
    pub texts: Vec<CustomTextItem>,
    #[serde(default)]
    pub designs: Vec<DesignDocument>,
    #[serde(default)]
    pub themes: Vec<crate::core::models::ThemeRecord>,
    #[serde(default)]
    pub has_media: bool,
    #[serde(default)]
    pub media_files: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub enum ImportOutcome {
    Song(String),
    Text(String),
    Design(String),
    Theme {
        theme: Box<PresentationTheme>,
        font_missing: bool,
    },
    Playlist {
        name: String,
        items_count: usize,
        missing_fonts: Vec<String>,
    },
    Media(String),
}

impl ImportOutcome {
    pub fn description(&self, language: crate::core::i18n::Language) -> String {
        match self {
            Self::Song(title) => format!("{}: {title}", language.text("workspace-song-imported")),
            Self::Text(title) => format!("{}: {title}", language.text("workspace-text-imported")),
            Self::Design(name) => format!("{}: {name}", language.text("workspace-design-imported")),
            Self::Theme { theme, font_missing } => {
                if *font_missing {
                    format!("{}: {} ({})", language.text("workspace-theme-imported"), theme.name, language.text("workspace-font-missing-title"))
                } else {
                    format!("{}: {}", language.text("workspace-theme-imported"), theme.name)
                }
            }
            Self::Playlist { name, items_count, .. } => {
                format!("{}: {name} ({items_count} {})", language.text("workspace-playlist-imported"), language.text("workspace-items"))
            }
            Self::Media(title) => format!("{}: {title}", language.text("workspace-media-imported")),
        }
    }
}

pub fn export_song(song: &Song) -> Result<Vec<u8>, String> {
    let package = SongPackage {
        format: "omni-presenter-song".into(),
        version: 1,
        title: song.title.clone(),
        artist: song.artist.clone(),
        text: song.text.clone(),
        tags: song.tags.clone(),
        folder: song.folder.clone(),
    };
    serde_json::to_vec_pretty(&package).map_err(|e| e.to_string())
}

pub fn export_text(text: &CustomTextItem) -> Result<Vec<u8>, String> {
    let package = TextPackage {
        format: "omni-presenter-text".into(),
        version: 1,
        title: text.title.clone(),
        content: text.content.clone(),
        folder: text.folder.clone(),
    };
    serde_json::to_vec_pretty(&package).map_err(|e| e.to_string())
}

pub fn export_design(scene: &DesignDocument) -> Result<Vec<u8>, String> {
    let package = DesignPackage {
        format: "omni-presenter-design".into(),
        version: 1,
        scene: scene.clone(),
    };
    serde_json::to_vec_pretty(&package).map_err(|e| e.to_string())
}

pub fn export_theme(theme: &PresentationTheme) -> Result<Vec<u8>, String> {
    let package = ThemePackage {
        format: "omni-presenter-theme".into(),
        version: 1,
        theme: theme.into(),
    };
    serde_json::to_vec_pretty(&package).map_err(|e| e.to_string())
}

pub fn export_playlist(
    name: &str,
    items: &[PlaylistItem],
    db: &AppDb,
    include_media: bool,
) -> Result<Vec<u8>, String> {
    let all_songs = db.get_all_songs().unwrap_or_default();
    let all_texts = db.get_all_custom_texts().unwrap_or_default();
    let all_designs = db.load_designs().unwrap_or_default();
    let all_themes = db.get_all_themes().unwrap_or_default();

    let mut referenced_songs = Vec::new();
    let mut referenced_texts = Vec::new();
    let mut referenced_designs = Vec::new();

    for item in items {
        match item.kind {
            PlaylistItemKind::Song => {
                let found = all_songs.iter().find(|s| s.title == item.title);
                if let Some(song) = found
                    && !referenced_songs.iter().any(|s: &Song| s.id == song.id)
                {
                    referenced_songs.push(song.clone());
                }
            }
            PlaylistItemKind::Text => {
                let found = all_texts.iter().find(|t| t.title == item.title);
                if let Some(text) = found
                    && !referenced_texts.iter().any(|t: &CustomTextItem| t.id == text.id)
                {
                    referenced_texts.push(text.clone());
                }
            }
            PlaylistItemKind::Design => {
                let found = all_designs.iter().find(|d| d.id == item.content || d.name == item.title);
                if let Some(design) = found
                    && !referenced_designs.iter().any(|d: &DesignDocument| d.id == design.id)
                {
                    referenced_designs.push(design.clone());
                }
            }
            _ => {}
        }
    }

    let mut media_files = HashMap::new();
    let mut media_payloads: Vec<(String, PathBuf)> = Vec::new();

    if include_media {
        let all_media = db.get_all_media_items().unwrap_or_default();
        for (idx, item) in items.iter().enumerate() {
            if item.kind == PlaylistItemKind::Media {
                // Find path from content or media catalog
                let mut path = PathBuf::from(&item.content);
                if !path.is_file() {
                    let found = all_media.iter().find(|m| m.title == item.title);
                    if let Some(media) = found {
                        path = PathBuf::from(&media.file_path);
                    }
                }
                if path.is_file() {
                    let ext = path
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("bin");
                    let entry_name = format!("media/{}_{}.{}", idx, sanitize_filename(&item.title), ext);
                    media_files.insert(entry_name.clone(), item.content.clone());
                    media_payloads.push((entry_name, path));
                }
            }
        }
    }

    let package = PlaylistPackage {
        format: "omni-presenter-playlist".into(),
        version: 1,
        name: name.to_string(),
        items: items.to_vec(),
        songs: referenced_songs,
        texts: referenced_texts,
        designs: referenced_designs,
        themes: all_themes.into_iter().map(Into::into).collect(),
        has_media: include_media && !media_payloads.is_empty(),
        media_files,
    };

    // Always use a clean ZIP for playlist packages to ensure uniform handling
    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let manifest = serde_json::to_vec_pretty(&package).map_err(|e| e.to_string())?;
        zip.start_file("playlist.json", options).map_err(|e| e.to_string())?;
        zip.write_all(&manifest).map_err(|e| e.to_string())?;

        for (entry_name, source_path) in media_payloads {
            if let Ok(mut f) = File::open(&source_path) {
                zip.start_file(
                    entry_name,
                    options.compression_method(zip::CompressionMethod::Stored),
                )
                .map_err(|e| e.to_string())?;
                let _ = std::io::copy(&mut f, &mut zip);
            }
        }
        zip.finish().map_err(|e| e.to_string())?;
    }

    Ok(buffer.into_inner())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .take(64)
        .collect()
}

pub fn import_file_from_path(
    path: &Path,
    db: &AppDb,
    state: &mut ControlPanelState,
) -> Result<ImportOutcome, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let hint = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("import");
    import_package_from_bytes(&bytes, hint, db, state)
}

pub fn import_package_from_bytes(
    bytes: &[u8],
    hint_filename: &str,
    db: &AppDb,
    state: &mut ControlPanelState,
) -> Result<ImportOutcome, String> {
    // 1. Check if it's a ZIP file
    if bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
        let cursor = std::io::Cursor::new(bytes);
        if let Ok(mut archive) = ZipArchive::new(cursor) {
            let json_bytes = {
                if let Ok(mut file) = archive.by_name("playlist.json") {
                    let mut b = Vec::new();
                    let _ = file.read_to_end(&mut b);
                    Some(b)
                } else {
                    None
                }
            };
            if let Some(json_bytes) = json_bytes {
                let package: PlaylistPackage = serde_json::from_slice(&json_bytes).map_err(|e| e.to_string())?;
                return import_playlist_package(&package, Some(&mut archive), db, state);
            }
        }
    }

    // 2. Check if JSON
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) {
        if let Some(format) = value.get("format").and_then(|v| v.as_str()) {
            match format {
                "omni-presenter-song" | "omni-presenter-lyrics" => {
                    let pkg: SongPackage = serde_json::from_value(value).map_err(|e| e.to_string())?;
                    let _ = db.add_song_full(&pkg.title, &pkg.artist, &pkg.text, &pkg.folder, &pkg.tags);
                    state.reload_all_selection_items(db);
                    return Ok(ImportOutcome::Song(pkg.title));
                }
                "omni-presenter-text" => {
                    let pkg: TextPackage = serde_json::from_value(value).map_err(|e| e.to_string())?;
                    let _ = db.add_custom_text(&pkg.title, &pkg.content, &pkg.folder);
                    state.reload_all_selection_items(db);
                    return Ok(ImportOutcome::Text(pkg.title));
                }
                "omni-presenter-design" => {
                    let pkg: DesignPackage = serde_json::from_value(value).map_err(|e| e.to_string())?;
                    db.save_design(&pkg.scene).map_err(|e| e.to_string())?;
                    return Ok(ImportOutcome::Design(pkg.scene.name));
                }
                "omni-presenter-theme" => {
                    let pkg: ThemePackage = serde_json::from_value(value).map_err(|e| e.to_string())?;
                    let theme: PresentationTheme = pkg.theme.into();
                    let missing = !crate::core::models::fonts::is_font_installed(&theme.font_family);
                    if !missing {
                        db.save_theme(&theme).map_err(|e| e.to_string())?;
                        state.themes = db.get_all_themes().unwrap_or_default();
                    }
                    return Ok(ImportOutcome::Theme {
                        theme: Box::new(theme),
                        font_missing: missing,
                    });
                }
                "omni-presenter-playlist" => {
                    let pkg: PlaylistPackage = serde_json::from_value(value).map_err(|e| e.to_string())?;
                    return import_playlist_package(&pkg, None, db, state);
                }
                _ => {}
            }
        }

        // Direct model deserialization fallbacks
        if let Ok(record) = serde_json::from_value::<ThemeRecord>(value.clone()) {
            let theme: PresentationTheme = record.into();
            if !theme.id.is_empty() && !theme.name.is_empty() {
                let missing = !crate::core::models::fonts::is_font_installed(&theme.font_family);
                if !missing {
                    db.save_theme(&theme).map_err(|e| e.to_string())?;
                    state.themes = db.get_all_themes().unwrap_or_default();
                }
                return Ok(ImportOutcome::Theme {
                    theme: Box::new(theme),
                    font_missing: missing,
                });
            }
        }

        if let Some(design) = serde_json::from_value::<DesignDocument>(value.clone()).ok().filter(|d| !d.id.is_empty() && !d.name.is_empty()) {
            db.save_design(&design).map_err(|e| e.to_string())?;
            return Ok(ImportOutcome::Design(design.name));
        }

        if let Some(song) = serde_json::from_value::<Song>(value.clone()).ok().filter(|s| !s.title.is_empty() && !s.text.is_empty()) {
            let _ = db.add_song_full(&song.title, &song.artist, &song.text, &song.folder, &song.tags);
            state.reload_all_selection_items(db);
            return Ok(ImportOutcome::Song(song.title));
        }

        if let Some(text) = serde_json::from_value::<CustomTextItem>(value).ok().filter(|t| !t.title.is_empty() && !t.content.is_empty()) {
            let _ = db.add_custom_text(&text.title, &text.content, &text.folder);
            state.reload_all_selection_items(db);
            return Ok(ImportOutcome::Text(text.title));
        }
    }

    // 3. Fallback: check extension or text
    let lower_hint = hint_filename.to_lowercase();
    if lower_hint.ends_with(".opsong")
        || lower_hint.ends_with(".hinos")
        || lower_hint.ends_with(".hino")
        || lower_hint.ends_with(".letras")
        || lower_hint.ends_with(".lrc")
        || lower_hint.ends_with(".txt")
        || lower_hint.ends_with(".omnipresenter-lyrics")
    {
        let text = String::from_utf8_lossy(bytes).to_string();
        if !text.trim().is_empty() {
            let title = hint_filename
                .rsplit_once('.')
                .map(|(stem, _)| stem)
                .unwrap_or(hint_filename);
            let _ = db.add_song(title, "", &text);
            state.reload_all_selection_items(db);
            return Ok(ImportOutcome::Song(title.to_string()));
        }
    }

    if lower_hint.ends_with(".optext") || lower_hint.ends_with(".omnipresenter-text") {
        let text = String::from_utf8_lossy(bytes).to_string();
        if !text.trim().is_empty() {
            let title = hint_filename
                .rsplit_once('.')
                .map(|(stem, _)| stem)
                .unwrap_or(hint_filename);
            let _ = db.add_custom_text(title, &text, "");
            state.reload_all_selection_items(db);
            return Ok(ImportOutcome::Text(title.to_string()));
        }
    }

    Err(format!("Formato de arquivo não reconhecido para '{hint_filename}'"))
}

fn import_playlist_package(
    package: &PlaylistPackage,
    mut archive: Option<&mut ZipArchive<std::io::Cursor<&[u8]>>>,
    db: &AppDb,
    state: &mut ControlPanelState,
) -> Result<ImportOutcome, String> {
    // 1. Import referenced songs
    for song in &package.songs {
        let _ = db.add_song_full(&song.title, &song.artist, &song.text, &song.folder, &song.tags);
    }

    // 2. Import referenced texts
    for text in &package.texts {
        let _ = db.add_custom_text(&text.title, &text.content, &text.folder);
    }

    // 3. Import referenced designs
    for design in &package.designs {
        let _ = db.save_design(design);
    }

    // 4. Import referenced themes and check fonts
    let mut missing_fonts = Vec::new();
    for theme_record in &package.themes {
        let theme: PresentationTheme = theme_record.clone().into();
        if !crate::core::models::fonts::is_font_installed(&theme.font_family) {
            missing_fonts.push(theme.font_family.clone());
        }
        let _ = db.save_theme(&theme);
    }

    // 5. Extract media files if bundled
    let mut updated_items = package.items.clone();
    if let Some(archive) = archive.as_mut() {
        let media_dir = db
            .transfer_directory()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("imported-media");
        let _ = std::fs::create_dir_all(&media_dir);

        for (entry_name, _orig_path) in &package.media_files {
            if let Ok(mut entry) = archive.by_name(entry_name) {
                let file_name = entry_name.rsplit('/').next().unwrap_or("media.bin");
                let dest = media_dir.join(file_name);
                if let Ok(mut out) = File::create(&dest) {
                    let _ = std::io::copy(&mut entry, &mut out);
                    let title = file_name
                        .rsplit_once('.')
                        .map(|(s, _)| s)
                        .unwrap_or(file_name);
                    let _ = db.add_media_item(title, dest.to_string_lossy().as_ref(), "media", "");

                    // Update playlist items that match this entry or original path
                    for item in &mut updated_items {
                        if item.kind == PlaylistItemKind::Media && (item.content == *_orig_path || item.title == title) {
                            item.content = dest.to_string_lossy().to_string();
                        }
                    }
                }
            }
        }
    }

    // 6. Update state playlist
    state.playlist = updated_items;
    state.capture_profiles();
    let _ = db.save_settings(&state.settings);
    state.reload_all_selection_items(db);
    state.themes = db.get_all_themes().unwrap_or_default();

    Ok(ImportOutcome::Playlist {
        name: package.name.clone(),
        items_count: state.playlist.len(),
        missing_fonts,
    })
}

#[cfg(test)]
#[path = "items_tests.rs"]
mod items_tests;
