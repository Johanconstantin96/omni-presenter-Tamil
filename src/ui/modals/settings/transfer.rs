use super::components::{self, ButtonTone};
use crate::{
    core::{models::Song, preferences::AppSettings, state::ControlPanelState},
    services::{db::AppDb, tasks::BackgroundTask, transfer::Package},
};
use eframe::egui::{self, RichText};
use egui_phosphor::regular::{
    CARET_DOWN, CARET_UP, CHECK, CHECK_SQUARE, MAGNIFYING_GLASS, MUSIC_NOTES, SQUARE, X,
};
use std::path::PathBuf;

#[path = "transfer_cards.rs"]
mod cards;

const ID: &str = "workspace_transfer_ui";
enum ResultData {
    Export(Package),
    Inspect(PathBuf, Package),
    Lyrics(Vec<Song>),
    Added,
    Example,
}
#[derive(Clone, Default)]
struct TransferUi {
    task: Option<BackgroundTask<Option<ResultData>>>,
    package: Option<(PathBuf, Package)>,
    songs: Vec<(bool, Song)>,
    show_preview_list: bool,
    filter_query: String,
    confirmed: bool,
    status: String,
    error: bool,
}
#[derive(Clone, Default)]
pub struct RestoreRequest {
    pub source: PathBuf,
}

fn get(ctx: &egui::Context) -> TransferUi {
    // Move large previews out instead of cloning potentially megabytes every frame.
    ctx.data_mut(|d| d.remove_temp(egui::Id::unique(ID)))
        .unwrap_or_default()
}
fn put(ctx: &egui::Context, value: TransferUi) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique(ID), value));
}

pub fn take_restore(ctx: &egui::Context) -> Option<RestoreRequest> {
    ctx.data_mut(|d| d.remove_temp(egui::Id::unique("workspace_restore_request")))
}

/// Always polled by the application, even if Settings has been closed.
pub fn poll(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    deck: &mut crate::core::slides::SlideDeck,
    db: &AppDb,
) {
    let mut data = get(ctx);
    if let Some(task) = &data.task {
        if let Some(result) = task.take() {
            data.task = None;
            data.error = false;
            match result {
                Ok(Some(ResultData::Export(package))) => {
                    data.status = state.settings.language.text("transfer-exported").into();
                    if !package.missing.is_empty() {
                        data.status.push_str(&format!(
                            "\n{}: {}",
                            state.settings.language.text("transfer-missing"),
                            package.missing.join(", ")
                        ));
                    }
                }
                Ok(Some(ResultData::Inspect(path, package))) => {
                    data.package = Some((path, package));
                    data.confirmed = false;
                }
                Ok(Some(ResultData::Lyrics(songs))) => {
                    data.songs = songs.into_iter().map(|s| (true, s)).collect();
                    data.show_preview_list = false;
                    data.filter_query.clear();
                }
                Ok(Some(ResultData::Added)) => {
                    deck.reload_from_db(db);
                    state.reload_all_selection_items(db);
                    data.songs.clear();
                    data.show_preview_list = false;
                    data.filter_query.clear();
                    data.status = state.settings.language.text("transfer-lyrics-added").into();
                }
                Ok(Some(ResultData::Example)) => {
                    data.status = state.settings.language.text("transfer-exported").into();
                }
                Ok(None) => {}
                Err(error) => {
                    data.error = true;
                    data.status = state.settings.language.text(&error).to_owned();
                }
            }
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
    put(ctx, data);
}

pub fn start_lyrics(ctx: &egui::Context) {
    let mut data = get(ctx);
    if data.task.is_some() {
        put(ctx, data);
        return;
    }
    data.status.clear();
    let language = crate::ui::i18n::language(ctx);
    data.task = Some(BackgroundTask::spawn_with_repaint(ctx, move || {
        let Some(paths) = rfd::FileDialog::new()
            .add_filter(
                "Lyrics",
                &[
                    "txt", "text", "md", "markdown", "lrc", "srt", "vtt", "pptx", "docx", "odt",
                    "odp",
                ],
            )
            .add_filter("All files", &["*"])
            .pick_files()
        else {
            return Ok(None);
        };
        if paths.len() > 10_000 {
            return Err("lyrics-too-large".into());
        }
        let mut songs = Vec::new();
        let mut total = 0;
        for path in paths {
            let song = crate::services::lyrics_import::read(&path)
                .map_err(|e| format!("{}: {}", path.display(), language.text(&e)))?;
            total += song.text.len();
            if total > 256 * 1024 * 1024 {
                return Err("lyrics-too-large".into());
            }
            songs.push(song);
        }
        Ok(Some(ResultData::Lyrics(songs)))
    }));
    put(ctx, data);
}

pub fn render(ui: &mut egui::Ui, draft: &AppSettings, db: Option<&AppDb>) {
    let language = draft.language;
    let Some(db) = db else {
        return;
    };
    let mut data = get(ui.ctx());
    let (theme, _) = crate::ui::theme::AppTheme::resolve(draft, ui.ctx());
    let busy = data.task.is_some();
    ui.add_enabled_ui(!busy, |ui| {
        let actions = cards::show(ui, language, &theme);
        if actions.export {
            let db = db.clone();
            let settings = draft.clone();
            data.status.clear();
            data.task = Some(BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
                let Some(path) = rfd::FileDialog::new()
                    .add_filter("Omni workspace ZIP", &["zip"])
                    .set_file_name("omni-presenter-backup.zip")
                    .save_file()
                else {
                    return Ok(None);
                };
                crate::services::transfer::export(&db, &settings, &path)
                    .map(|p| Some(ResultData::Export(p)))
            }));
        }
        if actions.import {
            let db = db.clone();
            data.status.clear();
            data.task = Some(BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
                let Some(path) = rfd::FileDialog::new()
                    .add_filter("Omni workspace ZIP", &["zip"])
                    .pick_file()
                else {
                    return Ok(None);
                };
                let package = crate::services::transfer::inspect(&path, &db)?;
                Ok(Some(ResultData::Inspect(path, package)))
            }));
        }
        if let Some((path, package)) = &data.package {
            ui.separator();
            ui.label(
                egui::RichText::new(path.file_name().unwrap_or_default().to_string_lossy())
                    .strong(),
            )
            .on_hover_text(path.display().to_string());
            egui::CollapsingHeader::new(language.text("transfer-package-details"))
                .id_salt("transfer_package_details")
                .show(ui, |ui| {
                    ui.label(format!("Omni Presenter {}", package.application_version));
                    for table in &package.tables {
                        ui.label(format!("{}: {}", table.name, table.rows.len()));
                    }
                });
            ui.label(format!(
                "{}: {}",
                language.text("transfer-assets"),
                package.assets.len()
            ));
            if !package.missing.is_empty() {
                ui.colored_label(
                    theme.status_warning_fg,
                    format!(
                        "{}: {}",
                        language.text("transfer-missing"),
                        package.missing.join(", ")
                    ),
                );
            }
            ui.colored_label(
                theme.status_warning_fg,
                language.text("transfer-restore-warning"),
            );
            ui.checkbox(&mut data.confirmed, language.text("transfer-confirm"));
            let mut cancel = false;
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(
                        data.confirmed,
                        egui::Button::new(language.text("transfer-restore")),
                    )
                    .clicked()
                {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            egui::Id::unique("workspace_restore_request"),
                            RestoreRequest {
                                source: path.clone(),
                            },
                        )
                    });
                    cancel = true;
                }
                if ui.button(language.text("common-cancel")).clicked() {
                    cancel = true;
                }
            });
            if cancel {
                data.package = None;
                data.confirmed = false;
            }
        }
        if actions.lyrics {
            put(ui.ctx(), std::mem::take(&mut data));
            start_lyrics(ui.ctx());
            data = get(ui.ctx());
        }
        if actions.example {
            data.task = Some(BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
                let Some(path) = rfd::FileDialog::new()
                    .set_file_name(crate::services::lyrics_import::example_filename(language))
                    .save_file()
                else {
                    return Ok(None);
                };
                std::fs::write(path, crate::services::lyrics_import::example(language))
                    .map_err(|e| e.to_string())?;
                Ok(Some(ResultData::Example))
            }));
        }
        if !data.songs.is_empty() {
            ui.add_space(4.0);
            components::card(ui, &theme, |ui| {
                let valid = data.songs.iter().any(|(selected, _)| *selected)
                    && data
                        .songs
                        .iter()
                        .filter(|(selected, _)| *selected)
                        .all(|(_, s)| !s.title.trim().is_empty() && !s.text.trim().is_empty());
                let selected_count = data.songs.iter().filter(|(selected, _)| *selected).count();
                let total_count = data.songs.len();

                // 1. Header with Icon, Title and Selected Count Badge
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 8.0, theme.accent_soft);
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        MUSIC_NOTES,
                        egui::FontId::proportional(18.0),
                        theme.accent,
                    );
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(language.text("transfer-lyrics-preview-title"))
                                    .size(14.0)
                                    .strong()
                                    .color(theme.text_primary),
                            );
                            let badge_text = format!("{selected_count} / {total_count}");
                            egui::Frame::new()
                                .fill(theme.accent_soft)
                                .corner_radius(10.0)
                                .inner_margin(egui::Margin::symmetric(8, 2))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(&badge_text)
                                            .size(11.0)
                                            .strong()
                                            .color(theme.accent),
                                    );
                                });
                        });
                        ui.label(
                            RichText::new(language.text("transfer-lyrics-preview"))
                                .size(11.5)
                                .color(theme.text_secondary),
                        );
                    });
                });

                ui.add_space(8.0);

                // 2. Action Buttons (Primary Add, Cancel, and Review List Toggle)
                let mut add_clicked = false;
                let mut cancel_clicked = false;

                ui.horizontal_wrapped(|ui| {
                    let add_label = format!(
                        "{} ({}/{})",
                        language.text("transfer-lyrics-add"),
                        selected_count,
                        total_count
                    );
                    let add_resp = ui
                        .add_enabled_ui(valid && selected_count > 0, |ui| {
                            components::styled_button(
                                ui,
                                Some(CHECK),
                                &add_label,
                                ButtonTone::Primary,
                                &theme,
                            )
                        })
                        .inner;
                    if add_resp.clicked() {
                        add_clicked = true;
                    }

                    if components::styled_button(
                        ui,
                        Some(X),
                        language.text("common-cancel"),
                        ButtonTone::Ghost,
                        &theme,
                    )
                    .clicked()
                    {
                        cancel_clicked = true;
                    }

                    let review_label = if data.show_preview_list {
                        language.text("transfer-lyrics-hide-list")
                    } else {
                        language.text("transfer-lyrics-show-list")
                    };
                    let review_icon = if data.show_preview_list {
                        CARET_UP
                    } else {
                        CARET_DOWN
                    };
                    if components::styled_button(
                        ui,
                        Some(review_icon),
                        review_label,
                        ButtonTone::Secondary,
                        &theme,
                    )
                    .clicked()
                    {
                        data.show_preview_list = !data.show_preview_list;
                    }
                });

                // 3. Collapsible / Expandable Preview List
                if data.show_preview_list {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // Selection tools & search bar
                    ui.horizontal_wrapped(|ui| {
                        if components::styled_button(
                            ui,
                            Some(CHECK_SQUARE),
                            language.text("common-select-all"),
                            ButtonTone::Secondary,
                            &theme,
                        )
                        .clicked()
                        {
                            for (selected, _) in data.songs.iter_mut() {
                                *selected = true;
                            }
                        }

                        if components::styled_button(
                            ui,
                            Some(SQUARE),
                            language.text("common-clear-selection"),
                            ButtonTone::Secondary,
                            &theme,
                        )
                        .clicked()
                        {
                            for (selected, _) in data.songs.iter_mut() {
                                *selected = false;
                            }
                        }

                        if total_count > 4 {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let query = &mut data.filter_query;
                                let search_width = (ui.available_width() - 8.0).min(320.0).max(200.0);
                                ui.add(
                                    egui::TextEdit::singleline(query)
                                        .hint_text(format!(
                                            "{} {}",
                                            MAGNIFYING_GLASS,
                                            language.text("transfer-lyrics-search-placeholder")
                                        ))
                                        .desired_width(search_width),
                                );
                            });
                        }
                    });

                    ui.add_space(6.0);

                    // Filtered indices
                    let query = data.filter_query.trim().to_lowercase();
                    let matching_indices: Vec<usize> = data
                        .songs
                        .iter()
                        .enumerate()
                        .filter(|(_, (_, song))| {
                            if query.is_empty() {
                                true
                            } else {
                                song.title.to_lowercase().contains(&query)
                                    || song.artist.to_lowercase().contains(&query)
                            }
                        })
                        .map(|(i, _)| i)
                        .collect();

                    let preview_limit = 100.min(matching_indices.len());

                    egui::ScrollArea::vertical()
                        .id_salt("transfer_lyrics_preview_scroll")
                        .max_height(480.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 6.0;
                            for &idx in &matching_indices[..preview_limit] {
                                let (selected, song) = &mut data.songs[idx];
                                ui.push_id(idx, |ui| {
                                    egui::Frame::new()
                                        .fill(theme.bg_pill)
                                        .stroke(egui::Stroke::new(1.0, theme.border_default))
                                        .corner_radius(8.0)
                                        .inner_margin(egui::Margin::symmetric(10, 6))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.checkbox(selected, "");
                                                ui.add(
                                                    egui::TextEdit::singleline(&mut song.title)
                                                        .desired_width(ui.available_width() - 30.0),
                                                );
                                            });
                                            egui::CollapsingHeader::new(
                                                RichText::new(language.text("transfer-lyrics-edit"))
                                                    .size(11.0)
                                                    .color(theme.text_secondary),
                                            )
                                            .id_salt(("transfer_song_edit", idx))
                                            .show(ui, |ui| {
                                                ui.spacing_mut().item_spacing.y = 4.0;
                                                ui.label(
                                                    RichText::new(language.text("transfer-artist"))
                                                        .size(11.0)
                                                        .color(theme.text_secondary),
                                                );
                                                ui.text_edit_singleline(&mut song.artist);
                                                ui.label(
                                                    RichText::new(language.text("transfer-folder"))
                                                        .size(11.0)
                                                        .color(theme.text_secondary),
                                                );
                                                ui.text_edit_singleline(&mut song.folder);
                                                ui.label(
                                                    RichText::new(language.text("transfer-tags"))
                                                        .size(11.0)
                                                        .color(theme.text_secondary),
                                                );
                                                ui.text_edit_singleline(&mut song.tags);
                                                ui.add(
                                                    egui::TextEdit::multiline(&mut song.text)
                                                        .desired_rows(6)
                                                        .desired_width(f32::INFINITY),
                                                );
                                            });
                                        });
                                });
                            }

                            if matching_indices.len() > preview_limit {
                                let limit_str = preview_limit.to_string();
                                let count_str = selected_count.to_string();
                                ui.label(language.format(
                                    "transfer-lyrics-preview-more",
                                    &[("limit", &limit_str), ("count", &count_str)],
                                ));
                            }
                        });
                }

                if add_clicked {
                    let db = db.clone();
                    let songs: Vec<_> = data
                        .songs
                        .iter()
                        .filter(|(selected, _)| *selected)
                        .map(|(_, s)| s.clone())
                        .collect();
                    data.task = Some(BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
                        db.import_lyrics(&songs)?;
                        Ok(Some(ResultData::Added))
                    }));
                }
                if cancel_clicked {
                    data.songs.clear();
                    data.show_preview_list = false;
                    data.filter_query.clear();
                }
            });
        }
    });
    cards::details(ui, language, &theme);
    if data.task.is_some() {
        ui.spinner();
        ui.label(language.text("transfer-working"));
    }
    if !data.status.is_empty() {
        if data.error {
            ui.colored_label(theme.status_danger, &data.status);
        } else {
            ui.label(&data.status);
        }
    }
    put(ui.ctx(), data);
}

#[cfg(test)]
#[path = "transfer_tests.rs"]
mod tests;
