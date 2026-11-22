use crate::core::i18n::Language;
use crate::core::state::ControlPanelState;
use crate::services::monitor::MonitorInfo;
use crate::ui::modals::settings::components::{card, setting_toggle};
use crate::ui::modals::settings::screen_card::{render_screen_card, ScreenCardAction, ScreenCardProps};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, RichText, Stroke};
use egui_phosphor::regular::{CHECK, DESKTOP, MONITOR, TEXT_AA};

pub fn render_step_screens(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    monitors: &[MonitorInfo],
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
            DESKTOP,
            egui::FontId::proportional(if is_compact_height { 18.0 } else { 22.0 }),
            tokens.accent,
        );
        ui.add_space(if is_compact_height { 2.0 } else { 4.0 });
        ui.label(
            RichText::new(language.text("settings-presentation-screens"))
                .size(if is_compact_height { 17.0 } else { 19.0 })
                .strong()
                .color(tokens.text_primary),
        );
        ui.label(
            RichText::new(language.text("settings-send-the-presentation-to-all-enabled-screens-above-click-a"))
                .size(if is_compact_height { 11.5 } else { 12.0 })
                .color(tokens.text_secondary),
        );
        ui.add_space(if is_compact_height { 6.0 } else { 10.0 });
    });

    card(ui, tokens, |ui| {
        if monitors.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(language.text("workspace-no-monitors-detected"))
                        .color(tokens.text_secondary)
                        .size(12.0),
                );
                ui.add_space(6.0);
            });
        } else {
            egui::ScrollArea::horizontal()
                .id_salt("wizard_physical_screens")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;
                        for (index, monitor) in monitors.iter().enumerate() {
                            let enabled = state.settings.physical_monitor_enabled(&monitor.name);
                            let label = (index + 1).to_string();
                            let res_str = format!("{} × {}", monitor.resolution.0, monitor.resolution.1);

                            let (_resp, action) = render_screen_card(
                                ui,
                                ScreenCardProps {
                                    id_salt: &format!("wiz_screen_{}", monitor.name),
                                    label: &label,
                                    name: &monitor.display_name,
                                    subtitle: Some(&res_str),
                                    selected: enabled,
                                    enabled: enabled && state.settings.physical_output_enabled,
                                    is_virtual: false,
                                    theme: tokens,
                                },
                            );

                            if matches!(action, ScreenCardAction::Select | ScreenCardAction::ToggleEnabled) {
                                state.settings.toggle_physical_monitor(&monitor.name);
                                if state.settings.physical_monitor_enabled(&monitor.name) {
                                    state.settings.monitor_name = monitor.name.clone();
                                }
                            }
                        }
                    });
                });
        }

        ui.add_space(6.0);
        setting_toggle(
            ui,
            &mut state.settings.physical_output_enabled,
            "workspace-open-the-projection-window-on-the-monitor",
            "settings-send-the-presentation-to-all-enabled-screens-above-click-a",
            tokens,
        );
        setting_toggle(
            ui,
            &mut state.settings.persistent_physical_output,
            "settings-keep-the-presentation-window-ready",
            "settings-keep-the-output-ready-show-the-standby-wallpaper-when-no",
            tokens,
        );
        setting_toggle(
            ui,
            &mut state.settings.suspend_projection_while_configuring,
            "settings-suspend-projection-while-configuring",
            "settings-suspend-projection-while-configuring-desc",
            tokens,
        );
    });

    ui.add_space(8.0);

    // Aspect ratio selection - Centered horizontal row
    card(ui, tokens, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{MONITOR}  {}", language.text("settings-screen-aspect-ratio")))
                    .size(12.5)
                    .strong()
                    .color(tokens.text_primary),
            );
        });
        ui.add_space(6.0);

        #[derive(Clone, Copy, PartialEq)]
        enum AspectOption {
            Monitor,
            Wide16_9,
            Wide16_10,
            Standard4_3,
            Ultra21_9,
        }

        let detected_monitor = monitors
            .iter()
            .find(|m| state.settings.physical_monitor_enabled(&m.name))
            .or_else(|| monitors.iter().find(|m| m.name == state.settings.monitor_name))
            .or_else(|| monitors.first());
        let detected_ratio = detected_monitor
            .map(|m| if m.resolution.0 > 0 && m.resolution.1 > 0 { m.resolution.0 as f32 / m.resolution.1 as f32 } else { 16.0 / 9.0 })
            .unwrap_or(16.0 / 9.0);

        let current_aspect = state.setup_wizard.presentation_aspect_ratio;
        let selected_opt = if current_aspect <= 0.05 {
            AspectOption::Monitor
        } else if (current_aspect - 16.0 / 9.0).abs() < 0.05 {
            AspectOption::Wide16_9
        } else if (current_aspect - 16.0 / 10.0).abs() < 0.05 {
            AspectOption::Wide16_10
        } else if (current_aspect - 4.0 / 3.0).abs() < 0.05 {
            AspectOption::Standard4_3
        } else if (current_aspect - 21.0 / 9.0).abs() < 0.05 {
            AspectOption::Ultra21_9
        } else {
            AspectOption::Monitor
        };

        let draw_btn = |ui: &mut egui::Ui, w: f32, h: f32, is_sel: bool, text: &str| -> egui::Response {
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::click());
            if resp.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
            let fill = if is_sel { tokens.accent_soft } else if resp.hovered() { tokens.bg_card_hover } else { tokens.bg_pill };
            let stroke = Stroke::new(if is_sel { 1.5 } else { 1.0 }, if is_sel { tokens.accent } else { tokens.border_default });
            ui.painter().rect(rect, 8.0, fill, stroke, egui::StrokeKind::Inside);
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, text, egui::FontId::proportional(11.5), if is_sel { tokens.accent } else { tokens.text_primary });
            resp
        };

        let monitor_label = if let Some(m) = detected_monitor {
            if m.resolution.0 > 0 && m.resolution.1 > 0 {
                format!("{MONITOR}  {} ({}×{})", language.text("settings-aspect-monitor"), m.resolution.0, m.resolution.1)
            } else {
                format!("{MONITOR}  {}", language.text("settings-aspect-monitor"))
            }
        } else {
            format!("{MONITOR}  {}", language.text("settings-aspect-monitor"))
        };
        if draw_btn(ui, ui.available_width(), 36.0, selected_opt == AspectOption::Monitor, &monitor_label).clicked() {
            state.setup_wizard.presentation_aspect_ratio = 0.0;
            state.presentation_aspect_ratio = detected_ratio;
            state.settings.service_mut().aspect_ratio = 0.0;
        }

        ui.add_space(6.0);

        let aspect_list = [
            (AspectOption::Wide16_9, "16:9 Breitbild", 16.0 / 9.0),
            (AspectOption::Wide16_10, "16:10", 16.0 / 10.0),
            (AspectOption::Standard4_3, "4:3 Klassisch", 4.0 / 3.0),
            (AspectOption::Ultra21_9, "21:9 Ultrabreit", 21.0 / 9.0),
        ];

        let gap = 8.0;
        let total_avail = ui.available_width();
        let btn_w = ((total_avail - gap * (aspect_list.len() - 1) as f32) / aspect_list.len() as f32).max(80.0);

        ui.horizontal(|ui| {
            for (idx, (opt, label, ratio_val)) in aspect_list.iter().enumerate() {
                if draw_btn(ui, btn_w, 36.0, selected_opt == *opt, label).clicked() {
                    state.setup_wizard.presentation_aspect_ratio = *ratio_val;
                    state.presentation_aspect_ratio = *ratio_val;
                    state.settings.service_mut().aspect_ratio = *ratio_val;
                }
                if idx + 1 < aspect_list.len() {
                    ui.add_space(gap);
                }
            }
        });
    });
}

pub fn render_step_breaks(
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
            TEXT_AA,
            egui::FontId::proportional(if is_compact_height { 18.0 } else { 22.0 }),
            tokens.accent,
        );
        ui.add_space(if is_compact_height { 2.0 } else { 4.0 });
        ui.label(
            RichText::new(language.text("settings-wizard-step-breaks"))
                .size(if is_compact_height { 17.0 } else { 19.0 })
                .strong()
                .color(tokens.text_primary),
        );
        ui.label(
            RichText::new("Automatische Formatierung und Zeilenumbrüche für Liedtexte und Bibelverse einstellen.")
                .size(if is_compact_height { 11.5 } else { 12.0 })
                .color(tokens.text_secondary),
        );
        ui.add_space(if is_compact_height { 6.0 } else { 10.0 });
    });

    card(ui, tokens, |ui| {
        setting_toggle(
            ui,
            &mut state.setup_wizard.wrap_text,
            "settings-wizard-auto-wrap",
            "settings-wizard-auto-wrap-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.setup_wizard.auto_fit,
            "settings-wizard-auto-fit",
            "settings-wizard-auto-fit-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.setup_wizard.dock_preview_bg,
            "settings-thumbnails-with-theme-background",
            "settings-show-the-theme-background-colors-and-fonts-in-stanza-cards",
            tokens,
        );
    });
}

pub fn render_step_ready(
    ui: &mut egui::Ui,
    state: &ControlPanelState,
    monitors: &[MonitorInfo],
    tokens: &AppTheme,
    language: Language,
) {
    let is_compact_height = ui.available_height() < 420.0;
    ui.vertical_centered(|ui| {
        ui.add_space(if is_compact_height { 2.0 } else { 6.0 });
        let icon_size = if is_compact_height { 40.0 } else { 52.0 };
        let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(icon_size, icon_size), egui::Sense::hover());
        ui.painter().rect_filled(icon_rect, 13.0, tokens.status_success_bg);
        ui.painter().text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            CHECK,
            egui::FontId::proportional(if is_compact_height { 22.0 } else { 28.0 }),
            tokens.status_success_fg,
        );
        ui.add_space(if is_compact_height { 4.0 } else { 6.0 });
        ui.label(
            RichText::new(language.text("settings-wizard-ready-title"))
                .size(if is_compact_height { 18.0 } else { 21.0 })
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(2.0);
        ui.label(
            RichText::new(language.text("settings-wizard-ready-desc"))
                .size(12.5)
                .color(tokens.text_secondary),
        );
        ui.add_space(14.0);
    });

    // Clean, structured summary card
    card(ui, tokens, |ui| {
        let lang_name = match state.settings.language {
            Language::Portuguese => "Português 🇧🇷",
            Language::English => "English 🇺🇸",
            Language::Spanish => "Español 🇪🇸",
            Language::German => "Deutsch 🇩🇪",
        };

        let screen_name = if monitors.is_empty() {
            "Hauptmonitor".to_string()
        } else if let Some(first) = monitors.iter().find(|m| state.settings.physical_monitor_enabled(&m.name)) {
            first.display_name.clone()
        } else {
            "Projektion im Fenster".to_string()
        };

        let aspect_name = if state.setup_wizard.presentation_aspect_ratio <= 0.05 {
            language.text("settings-aspect-monitor")
        } else if (state.setup_wizard.presentation_aspect_ratio - 16.0 / 9.0).abs() < 0.05 {
            "16:9 Breitbild"
        } else if (state.setup_wizard.presentation_aspect_ratio - 16.0 / 10.0).abs() < 0.05 {
            "16:10"
        } else if (state.setup_wizard.presentation_aspect_ratio - 4.0 / 3.0).abs() < 0.05 {
            "4:3 Klassisch"
        } else if (state.setup_wizard.presentation_aspect_ratio - 21.0 / 9.0).abs() < 0.05 {
            "21:9 Ultrabreit"
        } else {
            "16:9 Breitbild"
        };

        let rows = [
            ("🌐  Sprache der Oberfläche", lang_name),
            ("🖥️  Projektionsausgabe", screen_name.as_str()),
            ("📐  Seitenverhältnis", aspect_name),
            ("🔄  Automatischer Zeilenumbruch", if state.setup_wizard.wrap_text { "Aktiviert" } else { "Deaktiviert" }),
            ("🔍  Automatische Schriftgröße", if state.setup_wizard.auto_fit { "Aktiviert" } else { "Deaktiviert" }),
        ];

        for (idx, (label, val)) in rows.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(RichText::new(*label).size(12.0).color(tokens.text_secondary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::Frame::new()
                        .fill(tokens.bg_pill)
                        .corner_radius(6.0)
                        .inner_margin(egui::vec2(10.0, 3.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(*val).size(12.0).strong().color(tokens.text_primary));
                        });
                });
            });
            if idx + 1 < rows.len() {
                ui.add_space(4.0);
            }
        }
    });

    ui.add_space(10.0);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new("Tipp: Du kannst alle Optionen oben jederzeit in den Einstellungen ändern.")
                .size(11.5)
                .color(tokens.text_secondary),
        );
    });
}
