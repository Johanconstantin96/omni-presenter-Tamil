use crate::core::models::PlaylistItemKind;
use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use eframe::egui;

pub fn handle_playlist_export_action(
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
    ctx: &egui::Context,
) {
    if state.playlist.is_empty() {
        return;
    }
    let has_media = state.playlist.iter().any(|item| item.kind == PlaylistItemKind::Media);
    if has_media {
        state.playlist_export_media_modal = Some(crate::core::state::PlaylistExportModalState {
            playlist_name: state.settings.service().name.clone(),
        });
    } else if let Some(db) = db {
        let name = if state.settings.service().name.is_empty() {
            "playlist".to_string()
        } else {
            state.settings.service().name.clone()
        };
        let items = state.playlist.clone();
        let db_clone = db.clone();
        let ctx = ctx.clone();
        let lang = state.settings.language;
        crate::services::tasks::BackgroundTask::spawn_with_repaint(&ctx, move || {
            match crate::services::transfer::items::export_playlist(&name, &items, &db_clone, false) {
                Ok(bytes) => {
                    let filename = format!("{name}.opplaylist");
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name(&filename)
                        .add_filter("Ablaufplan (*.opplaylist)", &["opplaylist"])
                        .save_file()
                    {
                        let write_res = std::fs::write(&path, bytes);
                        if let Err(err) = write_res {
                            eprintln!("Fehler beim Speichern des Ablaufplans: {err}");
                        }
                    }
                }
                Err(err) => eprintln!("Fehler beim Exportieren des Ablaufplans: {err}"),
            }
            Ok(())
        });
        state.notify(lang.text("workspace-playlist-export-started"));
    }
}
