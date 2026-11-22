use crate::{
    core::state::ControlPanelState,
    services::{db::AppDb, transfer::items::export_playlist},
};
use eframe::egui::{self, Color32, RichText, Vec2};
use egui_phosphor::regular::{EXPORT, FILE, FILM_STRIP, X};

pub fn render_playlist_export_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) {
    let Some(export_state) = state.playlist_export_media_modal.clone() else {
        return;
    };

    let language = state.settings.language;
    let tokens = state.theme.clone();
    let mut choice = None;
    let mut cancel = false;

    let frame = egui::Frame::new()
        .fill(tokens.bg_panel)
        .stroke(egui::Stroke::new(1.0, tokens.border_default))
        .corner_radius(14.0)
        .inner_margin(20)
        .shadow(egui::epaint::Shadow {
            offset: [0, 12],
            blur: 40,
            spread: 0,
            color: Color32::from_black_alpha(70),
        });

    let modal = egui::Modal::new(egui::Id::unique("playlist_export_modal"))
        .frame(frame)
        .backdrop_color(Color32::from_black_alpha(120));

    let _ = modal.show(ctx, |ui| {
        ui.set_width(440.0_f32.min((ctx.content_rect().width() - 48.0).max(280.0)));
        ui.spacing_mut().item_spacing = Vec2::new(8.0, 12.0);

        // Header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(EXPORT)
                    .size(20.0)
                    .color(tokens.accent),
            );
            ui.label(
                RichText::new(language.text("workspace-include-media-title"))
                    .size(17.0)
                    .strong()
                    .color(tokens.text_primary),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if tokens
                    .icon_button(ui, X, language.text("common-close"), false, Vec2::splat(26.0))
                    .clicked()
                {
                    cancel = true;
                }
            });
        });

        ui.add_space(4.0);

        ui.label(
            RichText::new(language.text("workspace-include-media-prompt"))
                .size(13.5)
                .color(tokens.text_secondary),
        );

        ui.add_space(10.0);

        // Action choices
        ui.vertical_centered_justified(|ui| {
            let include_btn = ui.add(
                egui::Button::new(
                    RichText::new(format!("{FILM_STRIP}  {}", language.text("workspace-include-media-yes")))
                        .size(13.5)
                        .strong()
                        .color(tokens.text_on_accent),
                )
                .fill(tokens.accent)
                .min_size(Vec2::new(ui.available_width(), 38.0)),
            );
            if include_btn.clicked() {
                choice = Some(true);
            }

            ui.add_space(4.0);

            let exclude_btn = ui.add(
                egui::Button::new(
                    RichText::new(format!("{FILE}  {}", language.text("workspace-include-media-no")))
                        .size(13.0)
                        .color(tokens.text_primary),
                )
                .min_size(Vec2::new(ui.available_width(), 36.0)),
            );
            if exclude_btn.clicked() {
                choice = Some(false);
            }

            ui.add_space(4.0);

            if ui.button(language.text("common-cancel")).clicked() {
                cancel = true;
            }
        });
    });

    if cancel {
        state.playlist_export_media_modal = None;
    } else if let Some(include_media) = choice {
        state.playlist_export_media_modal = None;
        if let Some(db) = db {
            let name = if export_state.playlist_name.is_empty() {
                "playlist".to_string()
            } else {
                export_state.playlist_name
            };
            let items = state.playlist.clone();
            let db_clone = db.clone();
            let lang = language;

            crate::services::tasks::BackgroundTask::spawn_with_repaint(ctx, move || {
                match export_playlist(&name, &items, &db_clone, include_media) {
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
}
