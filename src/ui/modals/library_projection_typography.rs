use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Frame, Margin, RichText, Stroke};
use egui_phosphor::regular::{CHECK_CIRCLE, CIRCLE};

pub enum LineModePreviewKind {
    Wrap,
    Fit,
}

pub fn render_line_choice_card(
    ui: &mut egui::Ui,
    id_str: &str,
    is_selected: bool,
    title: &str,
    description: &str,
    preview_kind: LineModePreviewKind,
    tokens: &AppTheme,
) -> bool {
    let mut clicked = false;
    let card_id = ui.make_persistent_id(id_str);

    let (bg, stroke) = if is_selected {
        (
            tokens.accent_soft,
            Stroke::new(1.5, tokens.accent),
        )
    } else {
        (tokens.bg_pill, Stroke::new(1.0, tokens.border_default))
    };

    let frame_response = Frame::new()
        .fill(bg)
        .stroke(stroke)
        .corner_radius(8.0)
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 10.0;

            // 1. Mini visual projector screen preview in exact 16:9 proportion
            let box_width = ui.available_width();
            let box_height = (box_width * 9.0 / 16.0).round().max(50.0);
            ui.set_min_height(box_height + 84.0);

            let (box_rect, _) = ui.allocate_exact_size(
                egui::vec2(box_width, box_height),
                egui::Sense::hover(),
            );
            let painter = ui.painter();
            painter.rect_filled(box_rect, 6.0, egui::Color32::from_rgb(16, 20, 26));
            let box_border = if is_selected {
                tokens.accent.linear_multiply(0.5)
            } else {
                egui::Color32::from_rgb(42, 50, 62)
            };
            painter.rect_stroke(box_rect, 6.0, Stroke::new(1.0, box_border), egui::StrokeKind::Inside);

            let center = box_rect.center();
            match preview_kind {
                LineModePreviewKind::Wrap => {
                    painter.text(
                        egui::pos2(center.x, center.y - 12.0),
                        egui::Align2::CENTER_CENTER,
                        "Porque Ele vive, posso crer",
                        egui::FontId::proportional(12.5),
                        egui::Color32::WHITE,
                    );
                    painter.text(
                        egui::pos2(center.x, center.y + 12.0),
                        egui::Align2::CENTER_CENTER,
                        "no amanhã!",
                        egui::FontId::proportional(12.5),
                        egui::Color32::WHITE,
                    );
                }
                LineModePreviewKind::Fit => {
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        "Porque Ele vive, posso crer no amanhã!",
                        egui::FontId::proportional(8.5),
                        egui::Color32::WHITE,
                    );
                }
            }

            // 2. Title with radio icon
            ui.horizontal(|ui| {
                let icon = if is_selected { CHECK_CIRCLE } else { CIRCLE };
                let icon_color = if is_selected {
                    tokens.accent
                } else {
                    tokens.text_secondary
                };
                ui.label(RichText::new(icon).size(15.0).color(icon_color));

                ui.label(
                    RichText::new(title)
                        .size(12.5)
                        .strong()
                        .color(if is_selected {
                            tokens.accent
                        } else {
                            tokens.text_primary
                        }),
                );
            });

            // 3. Description
            ui.add(
                egui::Label::new(
                    RichText::new(description)
                        .size(10.5)
                        .color(tokens.text_secondary),
                )
                .wrap(),
            );
        });

    let rect = frame_response.response.rect;
    let interact = ui.interact(rect, card_id, egui::Sense::click());
    if interact.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        if !is_selected {
            ui.painter().rect_stroke(
                rect,
                8.0,
                Stroke::new(1.0, tokens.border_active),
                egui::StrokeKind::Inside,
            );
        }
    }
    if interact.clicked() {
        clicked = true;
    }

    clicked
}

pub fn render_typography_visual_options(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    _db: Option<&AppDb>,
    tokens: &AppTheme,
) {
    let current_wrap = state.settings.projection.lyrics.wrap_text;
    let current_fit = state.settings.projection.lyrics.auto_fit;

    let is_adaptar_selected = !current_wrap && current_fit;
    let is_quebrar_selected = !is_adaptar_selected;

    let mut new_selection = None;

    let prev_spacing_x = ui.spacing().item_spacing.x;
    ui.spacing_mut().item_spacing.x = 14.0;

    ui.columns(2, |columns| {
        // Coluna 1: Quebrar
        let wrap_title = crate::ui::i18n::text(&columns[0], "library-projection-lyrics-mode-wrap");
        let wrap_desc = crate::ui::i18n::text(&columns[0], "library-projection-lyrics-mode-wrap-desc");
        if render_line_choice_card(
            &mut columns[0],
            "line_mode_wrap",
            is_quebrar_selected,
            wrap_title,
            wrap_desc,
            LineModePreviewKind::Wrap,
            tokens,
        ) {
            new_selection = Some((true, false));
        }

        // Coluna 2: Adaptar
        let fit_title = crate::ui::i18n::text(&columns[1], "library-projection-lyrics-mode-fit");
        let fit_desc = crate::ui::i18n::text(&columns[1], "library-projection-lyrics-mode-fit-desc");
        if render_line_choice_card(
            &mut columns[1],
            "line_mode_fit",
            is_adaptar_selected,
            fit_title,
            fit_desc,
            LineModePreviewKind::Fit,
            tokens,
        ) {
            new_selection = Some((false, true));
        }
    });

    ui.spacing_mut().item_spacing.x = prev_spacing_x;

    if let Some((wrap, fit)) = new_selection {
        state.settings.projection.lyrics.wrap_text = wrap;
        state.settings.projection.lyrics.auto_fit = fit;
        state.setup_wizard.wrap_text = wrap;
        state.setup_wizard.auto_fit = fit;
        for theme in &mut state.themes {
            theme.options.wrap_text = wrap;
            theme.options.auto_fit = fit;
        }
    }
}
