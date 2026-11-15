use crate::core::i18n::Language;
use crate::core::interface_theme::ThemeMode;
use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use crate::services::monitor::MonitorInfo;
use crate::ui::modals::settings::components::{appearance_card, card, styled_button, ButtonTone};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, RichText, Stroke};
use egui_phosphor::regular::{
    ARROW_LEFT, ARROW_RIGHT, CHECK, CHECK_CIRCLE, DESKTOP, GLOBE, HAND_WAVING, PAINT_BRUSH,
    TEXT_AA, X,
};

pub fn render_setup_wizard_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    monitors: &[MonitorInfo],
    db: Option<&AppDb>,
) -> Option<usize> {
    if !state.show_setup_wizard {
        return None;
    }

    let tokens = state.theme.clone();
    let language = state.settings.language;
    let mut monitor_target = None;

    let screen_rect = ctx.content_rect();
    let is_compact_height = screen_rect.height() < 680.0;
    let is_narrow_width = screen_rect.width() < 640.0;

    let margin_y = if screen_rect.height() < 580.0 {
        10
    } else if is_compact_height {
        14
    } else {
        22
    };
    let margin_x = if is_narrow_width { 14 } else { 22 };

    let frame = egui::Frame::new()
        .fill(tokens.bg_panel)
        .stroke(Stroke::new(1.0, tokens.border_default))
        .corner_radius(16.0)
        .inner_margin(egui::Margin::symmetric(margin_x, margin_y))
        .shadow(egui::epaint::Shadow {
            offset: [0, 14],
            blur: 48,
            spread: 0,
            color: Color32::from_black_alpha(75),
        });

    egui::Modal::new(egui::Id::unique("initial_setup_wizard_modal"))
        .frame(frame)
        .backdrop_color(Color32::from_black_alpha(140))
        .show(ctx, |ui| {
            let max_w = 620.0_f32.min((screen_rect.width() - 32.0).max(300.0));
            ui.set_width(max_w);

            let visuals = &mut ui.style_mut().visuals;
            visuals.widgets.inactive.weak_bg_fill = tokens.bg_pill;
            visuals.widgets.inactive.bg_fill = tokens.bg_pill;
            visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, tokens.border_default);
            visuals.widgets.hovered.weak_bg_fill = tokens.bg_card_hover;
            visuals.widgets.hovered.bg_fill = tokens.bg_card_hover;
            ui.spacing_mut().item_spacing = egui::vec2(8.0, if is_compact_height { 6.0 } else { 10.0 });

            // Close button (X) in top right corner
            let close_rect = egui::Rect::from_min_size(
                egui::pos2(ui.max_rect().right() - 20.0, ui.max_rect().top() - 2.0),
                egui::vec2(20.0, 20.0),
            );
            let close_resp = ui.interact(close_rect, ui.auto_id_with("close_setup_wizard_x"), egui::Sense::click());
            if close_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                ui.painter().circle_filled(close_rect.center(), 10.0, tokens.bg_card_hover);
            }
            ui.painter().text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                X,
                egui::FontId::proportional(11.0),
                tokens.text_muted,
            );
            if close_resp.on_hover_text(language.text("settings-wizard-skip")).clicked() {
                monitor_target = finish_wizard(state, db);
            }

            // Centered Stepper Navigation Header
            render_stepper_header(ui, state, &tokens, language);
            ui.add_space(if is_compact_height { 8.0 } else { 12.0 });

            // Ensure content area scrolls if window height is small, keeping header and footer pinned
            let header_and_footer_reserve = if is_compact_height { 110.0 } else { 135.0 };
            let body_max_h = (screen_rect.height() - (margin_y as f32 * 2.0 + header_and_footer_reserve)).max(160.0);

            egui::ScrollArea::vertical()
                .id_salt(("setup_wizard_step_scroll", state.setup_wizard.step))
                .max_height(body_max_h)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    match state.setup_wizard.step {
                        0 => render_step_welcome(ui, state, &tokens, language),
                        1 => render_step_appearance(ui, state, &tokens, language),
                        2 => render_step_screens(ui, state, monitors, &tokens, language),
                        3 => render_step_breaks(ui, state, &tokens, language),
                        _ => render_step_ready(ui, state, monitors, &tokens, language),
                    }
                });

            ui.add_space(if is_compact_height { 8.0 } else { 14.0 });

            // Bottom Navigation Footer (always visible and pinned)
            render_wizard_footer(ui, state, db, &mut monitor_target, &tokens, language);
        });

    monitor_target
}

fn render_stepper_header(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
    language: Language,
) {
    let steps = [
        (0, HAND_WAVING, language.text("settings-wizard-step-welcome")),
        (1, PAINT_BRUSH, language.text("settings-wizard-step-appearance")),
        (2, DESKTOP, language.text("settings-wizard-step-monitors")),
        (3, TEXT_AA, language.text("settings-wizard-step-breaks")),
        (4, CHECK_CIRCLE, language.text("settings-wizard-step-ready")),
    ];

    let current = state.setup_wizard.step;
    let available_w = ui.available_width();
    let compact_pills = available_w < 520.0;
    let gap = if compact_pills { 6.0 } else { 8.0 };

    let pill_widths: Vec<f32> = steps
        .iter()
        .map(|(step_idx, icon, label)| {
            let text = if compact_pills && *step_idx != current {
                (*icon).to_string()
            } else {
                format!("{icon} {label}")
            };
            let text_layout = ui.painter().layout_no_wrap(
                text,
                egui::FontId::proportional(11.0),
                tokens.text_primary,
            );
            let min_w = if compact_pills && *step_idx != current { 28.0 } else { 64.0 };
            (text_layout.size().x + if compact_pills && *step_idx != current { 12.0 } else { 20.0 }).max(min_w)
        })
        .collect();

    let total_w: f32 = pill_widths.iter().sum::<f32>() + gap * (steps.len() - 1) as f32;
    let offset_x = ((available_w - total_w) * 0.5).max(0.0);

    ui.horizontal(|ui| {
        if offset_x > 0.0 {
            ui.add_space(offset_x);
        }
        for (idx, (step_idx, icon, label)) in steps.iter().enumerate() {
            let is_active = *step_idx == current;
            let is_past = *step_idx < current;

            let (bg_color, text_color, border_color) = if is_active {
                (tokens.accent, tokens.text_on_accent, tokens.accent)
            } else if is_past {
                (tokens.accent_soft, tokens.text_secondary, tokens.border_default)
            } else {
                (tokens.bg_pill, tokens.text_muted, tokens.border_default)
            };

            let text = if compact_pills && !is_active {
                (*icon).to_string()
            } else {
                format!("{icon} {label}")
            };
            let pill_w = pill_widths[idx];
            let (rect, response) = ui.allocate_exact_size(egui::vec2(pill_w, 28.0), egui::Sense::click());

            if response.hovered() && !is_active {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if response.clicked() {
                state.setup_wizard.step = *step_idx;
            }

            ui.painter().rect(
                rect,
                14.0,
                if response.hovered() && !is_active { tokens.bg_card_hover } else { bg_color },
                Stroke::new(1.0, border_color),
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(11.0),
                text_color,
            );

            if compact_pills && !is_active {
                response.on_hover_text(*label);
            }

            if idx + 1 < steps.len() {
                ui.add_space(gap);
            }
        }
    });
}

fn render_step_welcome(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
    language: Language,
) {
    let is_compact_height = ui.available_height() < 420.0;
    ui.vertical_centered(|ui| {
        ui.add_space(if is_compact_height { 4.0 } else { 10.0 });
        let icon_size = if is_compact_height { 44.0 } else { 56.0 };
        let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(icon_size, icon_size), egui::Sense::hover());
        ui.painter().rect_filled(icon_rect, 12.0, tokens.accent_soft);
        ui.painter().text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            HAND_WAVING,
            egui::FontId::proportional(if is_compact_height { 24.0 } else { 30.0 }),
            tokens.accent,
        );
        ui.add_space(if is_compact_height { 4.0 } else { 8.0 });
        ui.label(
            RichText::new(language.text("settings-wizard-welcome"))
                .size(if is_compact_height { 19.0 } else { 22.0 })
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(2.0);
        ui.label(
            RichText::new(language.text("settings-wizard-welcome-desc"))
                .size(if is_compact_height { 11.5 } else { 13.0 })
                .color(tokens.text_secondary),
        );
        ui.add_space(if is_compact_height { 14.0 } else { 22.0 });
    });

    // Language selection card with ComboBox that scales gracefully to any number of languages
    card(ui, tokens, |ui| {
        let is_narrow = ui.available_width() < 440.0;
        if is_narrow {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let icon_dim = if is_compact_height { 26.0 } else { 30.0 };
                    let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(icon_dim, icon_dim), egui::Sense::hover());
                    ui.painter().rect_filled(icon_rect, 7.0, tokens.accent_soft);
                    ui.painter().text(
                        icon_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        GLOBE,
                        egui::FontId::proportional(if is_compact_height { 13.0 } else { 15.0 }),
                        tokens.accent,
                    );
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(language.text("settings-language"))
                                .size(if is_compact_height { 12.5 } else { 13.0 })
                                .strong()
                                .color(tokens.text_primary),
                        );
                        ui.label(
                            RichText::new(language.text("settings-language-hint"))
                                .size(if is_compact_height { 10.5 } else { 11.0 })
                                .color(tokens.text_secondary),
                        );
                    });
                });
                ui.add_space(6.0);
                let current_lang = state.settings.language;
                let selected_text = format!("{}  {}", current_lang.flag(), current_lang.name());
                let combo_w = ui.available_width();
                egui::ComboBox::from_id_salt("wizard_language_picker")
                    .selected_text(
                        RichText::new(selected_text)
                            .size(12.5)
                            .strong()
                            .color(tokens.text_primary),
                    )
                    .width(combo_w)
                    .show_ui(ui, |ui| {
                        for lang in Language::ALL {
                            let is_selected = state.settings.language == lang;
                            let label = format!("{}  {}", lang.flag(), lang.name());
                            if ui.selectable_value(&mut state.settings.language, lang, label).clicked() {
                                ui.ctx().request_repaint();
                            }
                            if is_selected {
                                ui.scroll_to_cursor(Some(egui::Align::Center));
                            }
                        }
                    });
            });
        } else {
            ui.horizontal(|ui| {
                let icon_dim = if is_compact_height { 28.0 } else { 32.0 };
                let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(icon_dim, icon_dim), egui::Sense::hover());
                ui.painter().rect_filled(icon_rect, 8.0, tokens.accent_soft);
                ui.painter().text(
                    icon_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    GLOBE,
                    egui::FontId::proportional(if is_compact_height { 14.0 } else { 16.0 }),
                    tokens.accent,
                );
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(language.text("settings-language"))
                            .size(if is_compact_height { 12.5 } else { 13.0 })
                            .strong()
                            .color(tokens.text_primary),
                    );
                    ui.label(
                        RichText::new(language.text("settings-language-hint"))
                            .size(if is_compact_height { 10.5 } else { 11.0 })
                            .color(tokens.text_secondary),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let current_lang = state.settings.language;
                    let selected_text = format!("{}  {}", current_lang.flag(), current_lang.name());
                    egui::ComboBox::from_id_salt("wizard_language_picker")
                        .selected_text(
                            RichText::new(selected_text)
                                .size(if is_compact_height { 12.0 } else { 12.5 })
                                .strong()
                                .color(tokens.text_primary),
                        )
                        .width(220.0)
                        .show_ui(ui, |ui| {
                            for lang in Language::ALL {
                                let is_selected = state.settings.language == lang;
                                let label = format!("{}  {}", lang.flag(), lang.name());
                                if ui.selectable_value(&mut state.settings.language, lang, label).clicked() {
                                    ui.ctx().request_repaint();
                                }
                                if is_selected {
                                    ui.scroll_to_cursor(Some(egui::Align::Center));
                                }
                            }
                        });
                });
            });
        }
    });

    ui.add_space(if is_compact_height { 8.0 } else { 14.0 });
}

fn render_step_appearance(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
    language: Language,
) {
    let is_compact_height = ui.available_height() < 420.0;
    ui.vertical_centered(|ui| {
        let icon_size = if is_compact_height { 36.0 } else { 44.0 };
        let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(icon_size, icon_size), egui::Sense::hover());
        ui.painter().rect_filled(icon_rect, 10.0, tokens.accent_soft);
        ui.painter().text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            PAINT_BRUSH,
            egui::FontId::proportional(if is_compact_height { 18.0 } else { 22.0 }),
            tokens.accent,
        );
        ui.add_space(if is_compact_height { 2.0 } else { 4.0 });
        ui.label(
            RichText::new(language.text("appearance-title"))
                .size(if is_compact_height { 17.0 } else { 19.0 })
                .strong()
                .color(tokens.text_primary),
        );
        ui.label(
            RichText::new(language.text("appearance-hint"))
                .size(if is_compact_height { 11.5 } else { 12.0 })
                .color(tokens.text_secondary),
        );
        ui.add_space(if is_compact_height { 8.0 } else { 14.0 });

        // Centered 3 equal-width mode choices
        let operator = state.settings.operator_mut();
        let selected_mode = if operator.theme_mode == ThemeMode::Legacy {
            if operator.dark_mode { ThemeMode::Dark } else { ThemeMode::Light }
        } else {
            operator.theme_mode
        };

        let modes = [
            (ThemeMode::System, "appearance-system", "💻"),
            (ThemeMode::Light, "appearance-light", "☀️"),
            (ThemeMode::Dark, "appearance-dark", "🌙"),
        ];

        let gap = 12.0;
        let total_avail = ui.available_width();
        let btn_w = ((total_avail - gap * (modes.len() - 1) as f32) / modes.len() as f32).max(100.0);
        let btn_h = if is_compact_height { 36.0 } else { 42.0 };

        ui.horizontal(|ui| {
            for (idx, (mode, label_key, icon)) in modes.iter().enumerate() {
                let is_sel = selected_mode == *mode;
                let (rect, response) = ui.allocate_exact_size(egui::vec2(btn_w, btn_h), egui::Sense::click());
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if response.clicked() {
                    let op = state.settings.operator_mut();
                    op.theme_mode = *mode;
                    if *mode != ThemeMode::System {
                        op.dark_mode = *mode == ThemeMode::Dark;
                    }
                    state.apply_profiles();
                }

                let fill = if is_sel {
                    tokens.accent_soft
                } else if response.hovered() {
                    tokens.bg_card_hover
                } else {
                    tokens.bg_pill
                };

                let stroke = if is_sel {
                    Stroke::new(1.5, tokens.accent)
                } else {
                    Stroke::new(1.0, tokens.border_default)
                };

                ui.painter().rect(rect, 8.0, fill, stroke, egui::StrokeKind::Inside);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{icon}  {}", language.text(label_key)),
                    egui::FontId::proportional(if is_compact_height { 11.5 } else { 12.5 }),
                    if is_sel { tokens.accent } else { tokens.text_primary },
                );

                if idx + 1 < modes.len() {
                    ui.add_space(gap);
                }
            }
        });
    });

    ui.add_space(if is_compact_height { 10.0 } else { 16.0 });

    // Palettes section
    card(ui, tokens, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(language.text("settings-palette"))
                    .size(12.5)
                    .strong()
                    .color(tokens.text_primary),
            );
        });
        ui.add_space(6.0);

        let mut catalogue = crate::core::interface_theme::builtins();
        catalogue.extend(state.settings.interface_themes.clone());
        for plugin in state.settings.plugins.iter().filter(|p| p.enabled) {
            catalogue.extend(plugin.package.interface_themes.clone());
        }

        let selected = state.settings.operator().interface_theme_id.clone();
        let mut choice = selected.clone();
        let dark = state.settings.operator().theme_mode.is_dark(
            ui.ctx().system_theme() == Some(egui::Theme::Dark),
            state.settings.operator().dark_mode,
        );

        let gap = 8.0;
        let available = ui.available_width();
        let columns = ((available + gap) / 140.0).floor().clamp(1.0, 4.0) as usize;
        let width = ((available - gap * (columns - 1) as f32) / columns as f32).max(1.0);

        for row in catalogue.chunks(columns) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for definition in row {
                    ui.push_id(&definition.id, |ui| {
                        let palette = AppTheme::from_definition(definition, dark);
                        if appearance_card(
                            ui,
                            &palette,
                            definition.localized_name_for_mode(language, dark),
                            choice == definition.id,
                            width,
                            tokens,
                        ) {
                            choice = definition.id.clone();
                        }
                    });
                }
            });
            ui.add_space(gap);
        }

        if choice != selected {
            state.settings.operator_mut().interface_theme_id = choice;
            state.apply_profiles();
        }
    });
}

#[path = "setup_wizard_steps.rs"]
mod steps;
use steps::{render_step_breaks, render_step_ready, render_step_screens};



fn render_wizard_footer(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
    monitor_target: &mut Option<usize>,
    tokens: &AppTheme,
    _language: Language,
) {
    ui.horizontal(|ui| {
        // Skip Button on the left
        if styled_button(
            ui,
            None,
            "settings-wizard-skip",
            ButtonTone::Ghost,
            tokens,
        )
        .clicked()
        {
            *monitor_target = finish_wizard(state, db);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let step = state.setup_wizard.step;
            let is_last = step >= 4;

            if is_last {
                if styled_button(
                    ui,
                    Some(CHECK),
                    "settings-wizard-finish",
                    ButtonTone::Primary,
                    tokens,
                )
                .clicked()
                {
                    *monitor_target = finish_wizard(state, db);
                }
            } else {
                if styled_button(
                    ui,
                    Some(ARROW_RIGHT),
                    "settings-wizard-next",
                    ButtonTone::Primary,
                    tokens,
                )
                .clicked()
                {
                    state.setup_wizard.step += 1;
                }
            }

            if step > 0
                && styled_button(
                    ui,
                    Some(ARROW_LEFT),
                    "settings-wizard-back",
                    ButtonTone::Secondary,
                    tokens,
                )
                .clicked()
            {
                state.setup_wizard.step -= 1;
            }
        });
    });
}

fn finish_wizard(
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) -> Option<usize> {
    state.settings.first_run_completed = true;
    state.show_setup_wizard = false;
    state.settings.service_mut().aspect_ratio = state.setup_wizard.presentation_aspect_ratio;
    if state.setup_wizard.presentation_aspect_ratio > 0.0 {
        state.presentation_aspect_ratio = state.setup_wizard.presentation_aspect_ratio;
    }
    state.dock_preview_bg = state.setup_wizard.dock_preview_bg;

    // Apply wrap_text and auto_fit to all presentation themes
    for theme in &mut state.themes {
        theme.options.wrap_text = state.setup_wizard.wrap_text;
        theme.options.auto_fit = state.setup_wizard.auto_fit;
        if let Some(db) = db {
            let _ = db.save_theme(theme);
        }
    }

    if let Some(db) = db {
        let _ = db.save_settings(&state.settings);
    }
    state.apply_profiles();
    state.apply_theme_to_live();

    let lang = state.settings.language;
    state.notify(lang.text("settings-wizard-ready-title").to_string());

    None
}

#[cfg(test)]
#[path = "setup_wizard_tests.rs"]
mod tests;
