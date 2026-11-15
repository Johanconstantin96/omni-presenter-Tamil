use crate::{
    core::{
        models::fonts::{get_all_font_names, load_custom_font_file},
        state::ControlPanelState,
    },
    render::render_theme_lyrics,
    services::db::AppDb,
};
use eframe::egui::{self, Color32, RichText, Vec2};
use egui_phosphor::regular::{CHECK, FILE_ARROW_UP, MAGNIFYING_GLASS, WARNING, X};

pub fn render_font_resolution_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) {
    let Some(mut pending) = state.pending_font_theme.clone() else {
        return;
    };

    let language = state.settings.language;
    let tokens = state.theme.clone();
    let mut save_and_close = false;
    let mut close = false;
    let mut use_default = false;

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

    let modal = egui::Modal::new(egui::Id::unique("font_resolution_modal"))
        .frame(frame)
        .backdrop_color(Color32::from_black_alpha(120));

    let _ = modal.show(ctx, |ui| {
        ui.set_width(460.0_f32.min((ctx.content_rect().width() - 48.0).max(280.0)));
        ui.spacing_mut().item_spacing = Vec2::new(8.0, 10.0);

        // Header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(WARNING)
                    .size(20.0)
                    .color(tokens.status_warning_fg),
            );
            ui.label(
                RichText::new(language.text("workspace-font-missing-title"))
                    .size(17.0)
                    .strong()
                    .color(tokens.text_primary),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if tokens
                    .icon_button(ui, X, language.text("common-close"), false, Vec2::splat(26.0))
                    .clicked()
                {
                    close = true;
                }
            });
        });

        ui.add_space(4.0);

        let desc = language.format(
            "workspace-font-missing-prompt",
            &[
                ("font", &pending.missing_font),
                ("theme", &pending.theme.name),
            ],
        );
        ui.label(
            RichText::new(desc)
                .size(13.0)
                .color(tokens.text_secondary),
        );

        if let Some(status) = &pending.font_file_status {
            ui.colored_label(tokens.accent, status);
        }

        ui.add_space(4.0);

        // Botão para carregar arquivo TTF/OTF do disco
        let font_file_task_id = ui.make_persistent_id("font_file_picker_task");
        let pending_task = ui
            .ctx()
            .data_mut(|d| d.get_temp::<crate::services::tasks::BackgroundTask<Option<std::path::PathBuf>>>(font_file_task_id));

        if let Some(task) = &pending_task {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(33));
            if let Some(result) = task.take() {
                ui.ctx().data_mut(|d| d.remove::<crate::services::tasks::BackgroundTask<Option<std::path::PathBuf>>>(font_file_task_id));
                if let Ok(Some(path)) = result {
                    match load_custom_font_file(&path) {
                        Ok(family) => {
                            if let Some(dir) = db.and_then(|d| d.fonts_directory().ok()) {
                                let filename = path.file_name().unwrap_or_default();
                                let dest = dir.join(filename);
                                let _ = std::fs::copy(&path, &dest);
                            }
                            pending.font_file_status = Some(format!("Fonte '{family}' carregada com sucesso!"));
                            pending.selected_replacement = family;
                        }
                        Err(err) => {
                            pending.font_file_status = Some(format!("Erro ao carregar fonte: {err}"));
                        }
                    }
                }
            }
        }

        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    pending_task.is_none(),
                    egui::Button::new(RichText::new(format!("{FILE_ARROW_UP}  {}", language.text("workspace-load-font-file")))),
                )
                .clicked()
            {
                let task = crate::services::tasks::BackgroundTask::spawn_with_repaint(ui.ctx(), || {
                    Ok(rfd::FileDialog::new()
                        .add_filter("Fontes (*.ttf, *.otf)", &["ttf", "otf"])
                        .pick_file())
                });
                ui.ctx().data_mut(|d| d.insert_temp(font_file_task_id, task));
            }
        });

        ui.separator();

        // Campo de busca e seleção de fonte existente
        ui.label(
            RichText::new(language.text("workspace-select-replacement-font"))
                .size(12.5)
                .strong()
                .color(tokens.text_primary),
        );

        ui.horizontal(|ui| {
            ui.label(RichText::new(MAGNIFYING_GLASS).color(tokens.text_muted));
            ui.add(
                egui::TextEdit::singleline(&mut pending.search_query)
                    .hint_text(language.text("workspace-filter-fonts"))
                    .desired_width(ui.available_width() - 8.0),
            );
        });

        let all_fonts = get_all_font_names();
        let query_clean = pending.search_query.trim().to_lowercase();
        let filtered: Vec<&String> = all_fonts
            .iter()
            .filter(|f| query_clean.is_empty() || f.to_lowercase().contains(&query_clean))
            .collect();

        egui::ScrollArea::vertical()
            .max_height(140.0)
            .id_salt("font_replacement_scroll")
            .show(ui, |ui| {
                for font_name in filtered {
                    let is_selected = font_name.eq_ignore_ascii_case(&pending.selected_replacement);
                    let resp = ui.selectable_label(
                        is_selected,
                        RichText::new(font_name.as_str()).size(13.0),
                    );
                    if resp.clicked() {
                        pending.selected_replacement = font_name.clone();
                    }
                }
            });

        ui.add_space(6.0);

        // Preview da letra usando o tema com a fonte selecionada
        ui.label(
            RichText::new(language.text("workspace-font-preview"))
                .size(11.5)
                .color(tokens.text_muted),
        );
        let preview_h = 75.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), preview_h), egui::Sense::hover());
        let mut preview_theme = pending.theme.clone();
        preview_theme.font_family = pending.selected_replacement.clone();
        render_theme_lyrics(
            ui.painter(),
            rect,
            &preview_theme,
            "Porque Deus amou o mundo de tal maneira...",
        );

        ui.add_space(8.0);

        // Footer buttons
        ui.horizontal(|ui| {
            if ui
                .button(language.text("workspace-use-default-font"))
                .clicked()
            {
                use_default = true;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(format!("{CHECK}  {}", language.text("workspace-apply-and-save")))
                                .color(tokens.text_on_accent),
                        )
                        .fill(tokens.accent),
                    )
                    .clicked()
                {
                    save_and_close = true;
                }

                if ui.button(language.text("common-cancel")).clicked() {
                    close = true;
                }
            });
        });
    });

    if use_default {
        pending.theme.font_family = "Montserrat".into();
        if let Some(db) = db {
            let _ = db.save_theme(&pending.theme);
            state.themes = db.get_all_themes().unwrap_or_default();
            state.notify(language.format(
                "workspace-theme-saved",
                &[("name", &pending.theme.name)],
            ));
        }
        state.pending_font_theme = None;
    } else if save_and_close {
        pending.theme.font_family = pending.selected_replacement.clone();
        if let Some(db) = db {
            let _ = db.save_theme(&pending.theme);
            state.themes = db.get_all_themes().unwrap_or_default();
            state.notify(language.format(
                "workspace-theme-saved",
                &[("name", &pending.theme.name)],
            ));
        }
        state.pending_font_theme = None;
    } else if close {
        state.pending_font_theme = None;
    } else {
        state.pending_font_theme = Some(pending);
    }
}
