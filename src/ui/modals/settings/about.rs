use crate::core::state::{ControlPanelState, SettingsTab};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Margin, RichText, Stroke};
use egui_phosphor::regular::{
    BOOK_OPEN, CHAT_CIRCLE_TEXT, HAND_WAVING, HEART, KEYBOARD, MONITOR, PROJECTOR_SCREEN,
    PUZZLE_PIECE,
};

use super::components::{card, card_header, section, styled_button, ButtonTone};

pub(super) fn about(ui: &mut egui::Ui, state: &mut ControlPanelState, theme: &AppTheme) {
    section(
        ui,
        "settings-about-title",
        "settings-application-information-and-installed-version",
        theme,
    );

    // --- Hero Card: Marca, Versão e Propósito ---
    card(ui, theme, |ui| {
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(52.0, 52.0), egui::Sense::hover());
            ui.painter().rect_filled(icon_rect, 12.0, theme.accent_soft);
            ui.painter().rect_stroke(
                icon_rect,
                12.0,
                Stroke::new(1.0, theme.accent.linear_multiply(0.3)),
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                PROJECTOR_SCREEN,
                egui::FontId::proportional(28.0),
                theme.accent,
            );

            ui.add_space(10.0);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Omni Presenter")
                            .size(18.0)
                            .strong()
                            .color(theme.text_primary),
                    );

                    // Versão dinâmica vinda do Cargo.toml
                    let version_text = concat!("v", env!("CARGO_PKG_VERSION"));
                    egui::Frame::new()
                        .fill(theme.accent_soft)
                        .corner_radius(6.0)
                        .stroke(Stroke::new(1.0, theme.accent.linear_multiply(0.25)))
                        .inner_margin(Margin::symmetric(8, 3))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(version_text)
                                    .size(11.5)
                                    .strong()
                                    .color(theme.accent),
                            );
                        });

                    // Tag de software livre / licença
                    egui::Frame::new()
                        .fill(theme.bg_pill)
                        .corner_radius(6.0)
                        .stroke(Stroke::new(1.0, theme.border_default))
                        .inner_margin(Margin::symmetric(8, 3))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(crate::ui::i18n::text(ui, "settings-about-license"))
                                    .size(11.0)
                                    .color(theme.text_secondary),
                            );
                        });
                });

                ui.add_space(4.0);

                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "settings-about-description"))
                        .size(12.5)
                        .color(theme.text_secondary),
                );

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if styled_button(
                        ui,
                        Some(BOOK_OPEN),
                        "settings-about-documentation",
                        ButtonTone::Secondary,
                        theme,
                    )
                    .clicked()
                    {
                        ui.ctx().open_url(egui::OpenUrl::new_tab("https://github.com/lewis/omni-presenter"));
                    }

                    if styled_button(
                        ui,
                        Some(CHAT_CIRCLE_TEXT),
                        "settings-about-report-issue",
                        ButtonTone::Ghost,
                        theme,
                    )
                    .clicked()
                    {
                        ui.ctx().open_url(egui::OpenUrl::new_tab("https://github.com/lewis/omni-presenter/issues"));
                    }
                });
            });
        });
    });

    ui.add_space(8.0);

    // --- Card: Assistente de Configuração Inicial ---
    card(ui, theme, |ui| {
        card_header(
            ui,
            HAND_WAVING,
            "settings-wizard-title",
            "settings-wizard-subtitle",
            theme,
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(crate::ui::i18n::text(
                ui,
                "settings-wizard-welcome-desc",
            ))
            .size(12.0)
            .color(theme.text_secondary),
        );
        ui.add_space(8.0);
        if styled_button(
            ui,
            Some(HAND_WAVING),
            "settings-wizard-reopen",
            ButtonTone::Primary,
            theme,
        )
        .clicked()
        {
            state.show_setup_wizard = true;
            state.setup_wizard.step = 0;
            state.settings_ui.open = false;
        }
    });

    ui.add_space(8.0);

    // --- Card: Atalhos Rápidos para o Operador ---
    card(ui, theme, |ui| {
        card_header(
            ui,
            KEYBOARD,
            "settings-about-quick-links",
            "settings-about-quick-links-desc",
            theme,
        );
        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

            if styled_button(ui, Some(MONITOR), "settings-outputs", ButtonTone::Secondary, theme).clicked() {
                state.settings_ui.tab = SettingsTab::Outputs;
            }

            if styled_button(ui, Some(KEYBOARD), "shortcuts-title", ButtonTone::Secondary, theme).clicked() {
                state.settings_ui.tab = SettingsTab::Shortcuts;
            }

            if styled_button(ui, Some(PUZZLE_PIECE), "plugins-title", ButtonTone::Secondary, theme).clicked() {
                state.settings_ui.tab = SettingsTab::Plugins;
            }
        });
    });

    ui.add_space(8.0);

    // --- Card: Comunidade & Agradecimento ---
    card(ui, theme, |ui| {
        card_header(
            ui,
            HEART,
            "settings-about-community",
            "settings-about-community-desc",
            theme,
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(crate::ui::i18n::text(ui, "settings-about-made-with"))
                .size(12.0)
                .italics()
                .color(theme.text_secondary),
        );
    });
}
