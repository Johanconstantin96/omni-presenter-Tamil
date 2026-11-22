use crate::core::state::ControlPanelState;
use crate::ui::modals::settings::components::{styled_button, ButtonTone};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use egui_phosphor::regular::*;

pub fn render_web_hub_modal(ctx: &egui::Context, state: &mut ControlPanelState) {
    if !state.show_web_hub_modal {
        return;
    }

    let theme = state.theme.clone();
    let language = state.settings.language;
    let mut copied_notification = false;

    let frame = egui::Frame::new()
        .fill(theme.bg_panel)
        .stroke(Stroke::new(1.0, theme.border_default))
        .corner_radius(16.0)
        .inner_margin(24)
        .shadow(egui::epaint::Shadow {
            offset: [0, 16],
            blur: 48,
            spread: 0,
            color: Color32::from_black_alpha(85),
        });

    let mut open = true;
    let modal = egui::Modal::new(egui::Id::unique("web_hub_modal"))
        .frame(frame)
        .backdrop_color(Color32::from_black_alpha(150))
        .show(ctx, |ui| {
            let max_w = 580.0_f32.min((ctx.content_rect().width() - 40.0).max(340.0));
            ui.set_width(max_w);
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 12.0);

            // Botão Fechar (X)
            let close_rect = egui::Rect::from_min_size(
                egui::pos2(ui.max_rect().right() - 20.0, ui.max_rect().top() - 2.0),
                Vec2::splat(22.0),
            );
            let close_resp = ui.interact(close_rect, ui.auto_id_with("close_web_hub_x"), egui::Sense::click());
            if close_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                ui.painter().circle_filled(close_rect.center(), 11.0, theme.bg_card_hover);
            }
            ui.painter().text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                X,
                egui::FontId::proportional(11.0),
                theme.text_muted,
            );
            if close_resp.on_hover_text(language.text("control-close-media")).clicked() {
                open = false;
            }

            // Cabeçalho
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(format!("{GLOBE}  {}", language.text("quick-web-hub-title")))
                        .size(20.0)
                        .strong()
                        .color(theme.text_primary),
                );
                ui.add_space(2.0);
                ui.label(
                    RichText::new(language.text("quick-web-hub-desc"))
                        .size(12.5)
                        .color(theme.text_secondary),
                );
            });

            ui.add_space(6.0);

            // Status do Servidor e Endereço IP local
            let is_running = state.settings_ui.server_running;
            let port = state.settings.server_port;
            let running_host = if state.settings.allow_lan && !state.settings_ui.lan_address.is_empty() {
                state.settings_ui.lan_address.as_str()
            } else {
                "127.0.0.1"
            };

            egui::Frame::new()
                .fill(if is_running { theme.bg_card } else { theme.bg_pill })
                .stroke(Stroke::new(1.0, if is_running { theme.accent.linear_multiply(0.4) } else { theme.border_default }))
                .corner_radius(10.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let status_color = if is_running { theme.status_success_fg } else { theme.status_warning_fg };
                        ui.painter().circle_filled(ui.cursor().min + egui::vec2(6.0, 10.0), 4.5, status_color);
                        ui.add_space(14.0);

                        ui.vertical(|ui| {
                            let status_text = if is_running {
                                language.format("quick-web-hub-server-running", &[("port", &port.to_string())])
                            } else {
                                language.text("quick-web-hub-server-off").to_string()
                            };
                            ui.label(RichText::new(status_text).size(12.5).strong().color(theme.text_primary));
                            ui.label(
                                RichText::new(format!("Netzwerkadresse: http://{running_host}:{port}"))
                                    .size(11.5)
                                    .color(theme.text_secondary),
                            );
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if styled_button(ui, Some(GEAR), language.text("quick-web-hub-config-network"), ButtonTone::Secondary, &theme).clicked() {
                                open = false;
                                state.settings_ui.open = true;
                                state.settings_ui.tab = crate::core::state::SettingsTab::Connections;
                            }
                        });
                    });
                });

            ui.add_space(6.0);

            // URLs base
            let auto_base = if state.settings.allow_lan
                && !state.settings_ui.discovery_url.is_empty()
                && state.settings_ui.discovery_url.ends_with("/control")
            {
                state
                    .settings_ui
                    .discovery_url
                    .strip_suffix("/control")
                    .map(|b| b.to_string())
            } else {
                None
            };

            let control_auto = auto_base.as_ref().map(|b| format!("{b}/control"));
            let control_ip = crate::services::web::http_url(running_host, port, "/control");

            let chat_auto = auto_base.as_ref().map(|b| format!("{b}/chat"));
            let chat_ip = crate::services::web::http_url(running_host, port, "/chat");

            let screen_h = ctx.content_rect().height();
            let max_scroll_h = (screen_h - 260.0).clamp(240.0, 600.0);

            egui::ScrollArea::vertical()
                .id_salt("web_hub_scroll")
                .max_height(max_scroll_h)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 10.0);

                    // Card 1: Controle Web
                    render_connection_item(
                        ui,
                        &ConnectionItem {
                            icon: DEVICE_MOBILE,
                            title: language.text("quick-web-hub-control"),
                            desc: language.text("quick-web-hub-control-desc"),
                            auto_url: control_auto,
                            ip_url: control_ip,
                            active: true,
                        },
                        &language,
                        &theme,
                        &mut copied_notification,
                    );

                    // Card 2: Chat da Equipe
                    render_connection_item(
                        ui,
                        &ConnectionItem {
                            icon: CHAT_TEARDROP_TEXT,
                            title: language.text("quick-web-hub-chat"),
                            desc: language.text("quick-web-hub-chat-desc"),
                            auto_url: chat_auto,
                            ip_url: chat_ip,
                            active: state.settings.chat_enabled,
                        },
                        &language,
                        &theme,
                        &mut copied_notification,
                    );

                    // Card 3: Telas Web
                    let screens: Vec<_> = state.settings.screens.iter().filter(|s| s.enabled).collect();
                    if screens.is_empty() {
                        egui::Frame::new()
                            .fill(theme.bg_card)
                            .stroke(Stroke::new(1.0, theme.border_default))
                            .corner_radius(12.0)
                            .inner_margin(16.0)
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        RichText::new(format!("{MONITOR}  {}", language.text("quick-web-hub-screens")))
                                            .size(14.0)
                                            .strong()
                                            .color(theme.text_primary),
                                    );
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(language.text("quick-web-hub-no-screens"))
                                            .size(12.0)
                                            .color(theme.text_secondary),
                                    );
                                    ui.add_space(8.0);
                                    if styled_button(ui, Some(PLUS), language.text("quick-web-hub-config-network"), ButtonTone::Secondary, &theme).clicked() {
                                        open = false;
                                        state.settings_ui.open = true;
                                        state.settings_ui.tab = crate::core::state::SettingsTab::Outputs;
                                    }
                                });
                            });
                    } else {
                        for screen in screens {
                            let slug = crate::core::preferences::screen_url_slug(screen, &state.settings.screens);
                            let screen_auto = auto_base.as_ref().map(|b| format!("{b}/screen/{slug}"));
                            let screen_ip = crate::services::web::http_url(running_host, port, &format!("/screen/{slug}"));
                            let title = format!("Web-Bildschirm: {}", screen.name);
                            render_connection_item(
                                ui,
                                &ConnectionItem {
                                    icon: MONITOR,
                                    title: &title,
                                    desc: language.text("quick-web-hub-screens-desc"),
                                    auto_url: screen_auto,
                                    ip_url: screen_ip,
                                    active: is_running,
                                },
                                &language,
                                &theme,
                                &mut copied_notification,
                            );
                        }
                    }
                });
        });

    if copied_notification {
        state.notify(language.text("workspace-copied"));
    }

    if !open || modal.should_close() {
        state.show_web_hub_modal = false;
    }
}

struct ConnectionItem<'a> {
    icon: &'a str,
    title: &'a str,
    desc: &'a str,
    auto_url: Option<String>,
    ip_url: String,
    active: bool,
}

fn render_connection_item(
    ui: &mut egui::Ui,
    item: &ConnectionItem<'_>,
    language: &crate::core::i18n::Language,
    theme: &AppTheme,
    copied_notification: &mut bool,
) {
    egui::Frame::new()
        .fill(theme.bg_card)
        .stroke(Stroke::new(1.0, theme.border_default))
        .corner_radius(12.0)
        .inner_margin(14.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // Linha superior: Ícone, Título e Badge Ativo/Inativo
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(item.icon)
                            .size(16.0)
                            .color(if item.active { theme.accent } else { theme.text_muted }),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(item.title)
                            .size(14.0)
                            .strong()
                            .color(theme.text_primary),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (badge_text, badge_fg, badge_bg) = if item.active {
                            ("Ativo", theme.status_success_fg, theme.status_success_bg)
                        } else {
                            ("Desativado", theme.text_muted, theme.bg_pill)
                        };
                        egui::Frame::new()
                            .fill(badge_bg)
                            .corner_radius(6.0)
                            .inner_margin(egui::vec2(8.0, 3.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(badge_text).size(11.0).strong().color(badge_fg));
                            });
                    });
                });

                ui.add_space(3.0);
                ui.label(
                    RichText::new(item.desc)
                        .size(11.5)
                        .color(theme.text_secondary),
                );

                ui.add_space(10.0);

                // Se houver endereço automático (.local), renderiza a linha de Descoberta Automática
                if let Some(auto_url) = &item.auto_url {
                    render_url_row(
                        ui,
                        theme,
                        language,
                        UrlRowArgs {
                            icon: GLOBE,
                            label: language.text("discovery-title"),
                            url: auto_url,
                            dialog_title: &format!("{} ({})", item.title, language.text("discovery-title")),
                        },
                        copied_notification,
                    );
                    ui.add_space(8.0);
                }

                // Renderiza a linha de Endereço IP Direto
                render_url_row(
                    ui,
                    theme,
                    language,
                    UrlRowArgs {
                        icon: WIFI_HIGH,
                        label: language.text("discovery-ip-alternative"),
                        url: &item.ip_url,
                        dialog_title: &format!("{} ({})", item.title, language.text("discovery-ip-alternative")),
                    },
                    copied_notification,
                );
            });
        });
}

struct UrlRowArgs<'a> {
    icon: &'a str,
    label: &'a str,
    url: &'a str,
    dialog_title: &'a str,
}

fn render_url_row(
    ui: &mut egui::Ui,
    theme: &AppTheme,
    language: &crate::core::i18n::Language,
    args: UrlRowArgs<'_>,
    copied_notification: &mut bool,
) {
    let icon = args.icon;
    let label = args.label;
    let url = args.url;
    let dialog_title = args.dialog_title;
    let now = ui.ctx().input(|i| i.time);
    let copy_id = ui.auto_id_with(("copied_url", url));
    let last_copied: Option<f64> = ui.ctx().data(|d| d.get_temp(copy_id));
    let is_copied = last_copied.is_some_and(|t| now - t < 2.0);
    if is_copied {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
    }

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{icon}  {label}"))
                .size(11.0)
                .color(theme.text_secondary),
        );
    });
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        let buttons_w = 152.0;
        let avail_w = ui.available_width();
        let pill_w = (avail_w - buttons_w - 6.0).max(100.0);

        let pill_border = if is_copied {
            theme.accent
        } else {
            theme.border_subtle
        };

        let pill_resp = egui::Frame::new()
            .fill(theme.bg_pill)
            .corner_radius(6.0)
            .inner_margin(egui::vec2(10.0, 7.0))
            .stroke(Stroke::new(1.0, pill_border))
            .show(ui, |ui| {
                let inner_w = (pill_w - 20.0).max(10.0);
                ui.set_width(inner_w);
                ui.set_max_width(inner_w);
                ui.add_sized(
                    [inner_w, 18.0],
                    egui::Label::new(
                        RichText::new(url)
                            .monospace()
                            .size(11.5)
                            .color(if is_copied { theme.accent } else { theme.text_primary }),
                    )
                    .truncate(),
                );
            });

        let click_area = ui.interact(pill_resp.response.rect, ui.auto_id_with(url), egui::Sense::click());
        if click_area.clicked() {
            crate::ui::copy_to_clipboard(ui.ctx(), url);
            ui.ctx().data_mut(|d| d.insert_temp(copy_id, now));
            *copied_notification = true;
        }
        click_area
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(if is_copied { language.text("workspace-copied") } else { language.text("quick-web-hub-copy") });

        // Botão Copiar
        let copy_icon = if is_copied { CHECK } else { COPY };
        let copy_tone = if is_copied { ButtonTone::Primary } else { ButtonTone::Secondary };
        if styled_button(ui, Some(copy_icon), "", copy_tone, theme)
            .on_hover_text(if is_copied { language.text("workspace-copied") } else { language.text("quick-web-hub-copy") })
            .clicked()
        {
            crate::ui::copy_to_clipboard(ui.ctx(), url);
            ui.ctx().data_mut(|d| d.insert_temp(copy_id, now));
            *copied_notification = true;
        }

        // Botão QR Code
        if styled_button(ui, Some(QR_CODE), "", ButtonTone::Secondary, theme)
            .on_hover_text(language.text("quick-web-hub-qr"))
            .clicked()
        {
            crate::ui::components::open_qr_dialog(ui.ctx(), dialog_title.to_string(), url.to_string());
        }

        // Botão Abrir no Navegador
        if styled_button(
            ui,
            Some(ARROW_SQUARE_OUT),
            language.text("quick-web-hub-open"),
            ButtonTone::Secondary,
            theme,
        )
        .on_hover_text(language.text("quick-web-hub-open"))
        .clicked()
        {
            crate::ui::open_browser_url(ui.ctx(), url);
        }
    });
}
