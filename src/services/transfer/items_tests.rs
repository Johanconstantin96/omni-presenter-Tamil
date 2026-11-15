use super::*;
use crate::core::models::{CustomTextItem, PlaylistItem, PlaylistItemKind, PresentationTheme, Song};
use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use std::fs;
use std::path::PathBuf;

struct TempDir(PathBuf);
impl TempDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("omni_test_{}_{}", name, std::process::id()));
        let _ = fs::create_dir_all(&path);
        Self(path)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn test_song_export_and_import() {
    let db = AppDb::open_in_memory().unwrap();
    let mut state = ControlPanelState::default();

    let song = Song {
        id: Some(1),
        title: "Graça Maravilhosa".into(),
        artist: "John Newton".into(),
        text: "Graça maravilhosa, quão doce é o som".into(),
        tags: "louvor,tradicional".into(),
        folder: "Hinos".into(),
    };

    let exported = export_song(&song).expect("Failed to export song");
    assert!(!exported.is_empty());

    let outcome = import_package_from_bytes(&exported, "song.opsong", &db, &mut state).expect("Import failed");
    match outcome {
        ImportOutcome::Song(title) => assert_eq!(title, "Graça Maravilhosa"),
        _ => panic!("Expected Song import outcome"),
    }

    let all_songs = db.get_all_songs().unwrap();
    let imported = all_songs.iter().find(|s| s.title == "Graça Maravilhosa").expect("Song not in DB");
    assert_eq!(imported.artist, "John Newton");
    assert_eq!(imported.text, "Graça maravilhosa, quão doce é o som");
    assert_eq!(imported.folder, "Hinos");
}

#[test]
fn test_text_export_and_import() {
    let db = AppDb::open_in_memory().unwrap();
    let mut state = ControlPanelState::default();

    let text = CustomTextItem {
        id: Some(42),
        title: "Avisos da Semana".into(),
        content: "Culto de oração na quarta-feira às 19:30".into(),
        folder: "Comunicados".into(),
    };

    let exported = export_text(&text).expect("Failed to export text");
    let outcome = import_package_from_bytes(&exported, "text.optext", &db, &mut state).expect("Import failed");
    match outcome {
        ImportOutcome::Text(title) => assert_eq!(title, "Avisos da Semana"),
        _ => panic!("Expected Text import outcome"),
    }

    let all_texts = db.get_all_custom_texts().unwrap();
    let imported = all_texts.iter().find(|t| t.title == "Avisos da Semana").expect("Text not in DB");
    assert_eq!(imported.content, "Culto de oração na quarta-feira às 19:30");
    assert_eq!(imported.folder, "Comunicados");
}

#[test]
fn test_design_export_and_import() {
    let db = AppDb::open_in_memory().unwrap();
    let mut state = ControlPanelState::default();

    let design = crate::core::design::DesignDocument::template(1);
    let design_name = design.name.clone();

    let exported = export_design(&design).expect("Failed to export design");
    let outcome = import_package_from_bytes(&exported, "design.opdesign", &db, &mut state).expect("Import failed");
    match outcome {
        ImportOutcome::Design(name) => assert_eq!(name, design_name),
        _ => panic!("Expected Design import outcome"),
    }

    let designs = db.load_designs().unwrap();
    assert!(designs.iter().any(|d| d.name == design_name));
}

#[test]
fn test_theme_export_and_font_detection() {
    let db = AppDb::open_in_memory().unwrap();
    let mut state = ControlPanelState::default();

    // 1. Theme with known font
    let theme_standard = PresentationTheme {
        id: "theme_std_1".into(),
        name: "Tema Padrão".into(),
        font_family: "Montserrat".into(),
        ..Default::default()
    };

    let bytes = export_theme(&theme_standard).expect("Failed to export theme");
    let outcome = import_package_from_bytes(&bytes, "theme.optheme", &db, &mut state).expect("Import failed");
    match outcome {
        ImportOutcome::Theme { theme, font_missing } => {
            assert_eq!(theme.name, "Tema Padrão");
            assert!(!font_missing, "Montserrat should be detected as installed");
        }
        _ => panic!("Expected Theme outcome"),
    }
    assert!(db.get_all_themes().unwrap().iter().any(|t| t.name == "Tema Padrão"));

    // 2. Theme with unknown font
    let theme_missing = PresentationTheme {
        id: "theme_missing_1".into(),
        name: "Tema Fonte Rara".into(),
        font_family: "FonteTotalmenteInexistenteXYZ123".into(),
        ..Default::default()
    };

    let bytes_missing = export_theme(&theme_missing).expect("Failed to export theme");
    let outcome_missing = import_package_from_bytes(&bytes_missing, "theme.optheme", &db, &mut state).expect("Import failed");
    match outcome_missing {
        ImportOutcome::Theme { theme, font_missing } => {
            assert_eq!(theme.name, "Tema Fonte Rara");
            assert!(font_missing, "Nonexistent font should be reported as missing");
        }
        _ => panic!("Expected Theme outcome"),
    }
}

#[test]
fn test_playlist_export_and_import_with_media() {
    let temp = TempDir::new("playlist_transfer");
    let db = AppDb::open_in_memory().unwrap();

    // Set up dummy media file on disk
    let media_file = temp.0.join("sample_video.mp4");
    fs::write(&media_file, b"FAKE_VIDEO_CONTENT_12345").unwrap();
    db.add_media_item("Sample Video", &media_file.to_string_lossy(), "video", "Mídias").unwrap();

    let song_id = db.add_song("Música Playlist", "Artista", "Verso 1").unwrap();
    let all = db.get_all_songs().unwrap();
    let song = all.into_iter().find(|s| s.id == Some(song_id)).unwrap();

    let items = vec![
        PlaylistItem {
            id: "item-1".into(),
            title: song.title.clone(),
            subtitle: song.artist.clone(),
            kind: PlaylistItemKind::Song,
            content: song.text.clone(),
            played: false,
        },
        PlaylistItem {
            id: "item-2".into(),
            title: "Sample Video".into(),
            subtitle: "Vídeo".into(),
            kind: PlaylistItemKind::Media,
            content: media_file.to_string_lossy().to_string(),
            played: false,
        },
    ];

    let exported = export_playlist("Culto Especial", &items, &db, true).expect("Failed to export playlist");
    assert!(!exported.is_empty());

    // Import into fresh DB and state
    let target_db = AppDb::open_in_memory().unwrap();
    let mut target_state = ControlPanelState::default();

    let outcome = import_package_from_bytes(&exported, "playlist.opplaylist", &target_db, &mut target_state).expect("Import failed");
    match outcome {
        ImportOutcome::Playlist { name, items_count, .. } => {
            assert_eq!(name, "Culto Especial");
            assert_eq!(items_count, 2);
        }
        _ => panic!("Expected Playlist outcome"),
    }

    assert_eq!(target_state.playlist.len(), 2);
    assert_eq!(target_state.playlist[0].title, "Música Playlist");
    assert_eq!(target_state.playlist[1].title, "Sample Video");
    assert!(target_db.get_all_songs().unwrap().iter().any(|s| s.title == "Música Playlist"));
}

#[test]
fn test_font_installed_and_custom_loader() {
    assert!(crate::core::models::fonts::is_font_installed("Montserrat"));
    assert!(crate::core::models::fonts::is_font_installed("Roboto"));
    assert!(crate::core::models::fonts::is_font_installed("Inter"));
    assert!(!crate::core::models::fonts::is_font_installed("FonteTotalmenteInventada98765"));

    // Invalid font bytes should return an error
    let invalid_bytes = b"NOT_A_TRUE_TYPE_FONT";
    let res = crate::core::models::fonts::load_custom_font_data(invalid_bytes);
    assert!(res.is_err());
}
