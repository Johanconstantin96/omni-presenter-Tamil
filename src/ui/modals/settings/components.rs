use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Margin, RichText, Stroke};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonTone {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

pub fn apply_settings_style(ui: &mut egui::Ui, theme: &AppTheme) {
    let visuals = &mut ui.style_mut().visuals;

    // Inactive widgets (buttons, comboboxes, drag values, inputs)
    visuals.widgets.inactive.weak_bg_fill = theme.bg_pill;
    visuals.widgets.inactive.bg_fill = theme.bg_pill;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, theme.border_default);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(7);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme.text_primary);

    // Hovered widgets
    let hover_fill = theme.bg_card_hover.lerp_to_gamma(theme.accent_soft, 0.35);
    visuals.widgets.hovered.weak_bg_fill = hover_fill;
    visuals.widgets.hovered.bg_fill = hover_fill;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.25, theme.accent);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(7);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, theme.accent);

    // Active (pressed/clicking) widgets
    visuals.widgets.active.weak_bg_fill = theme.bg_card_hover;
    visuals.widgets.active.bg_fill = theme.bg_card_hover;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, theme.accent);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(7);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, theme.accent);

    // Open (e.g. open ComboBox or dropdown menu)
    visuals.widgets.open.weak_bg_fill = theme.bg_card_hover;
    visuals.widgets.open.bg_fill = theme.bg_card_hover;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, theme.accent);
    visuals.widgets.open.corner_radius = egui::CornerRadius::same(7);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, theme.accent);

    // Noninteractive background
    visuals.widgets.noninteractive.bg_fill = theme.bg_pill;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, theme.border_default);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(7);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, theme.text_secondary);

    // Selection & Popup windows (dropdown menus)
    visuals.selection.bg_fill = theme.accent_soft;
    visuals.selection.stroke = Stroke::new(1.0, theme.accent);
    visuals.window_fill = theme.bg_panel;
    visuals.window_stroke = Stroke::new(1.0, theme.border_default);
    visuals.panel_fill = theme.bg_panel;
    visuals.menu_corner_radius = egui::CornerRadius::same(8);
    visuals.window_corner_radius = egui::CornerRadius::same(8);

    // Button and widget padding
    ui.spacing_mut().button_padding = egui::vec2(10.0, 6.0);
}

pub fn styled_button(
    ui: &mut egui::Ui,
    icon: Option<&str>,
    text: &str,
    tone: ButtonTone,
    theme: &AppTheme,
) -> egui::Response {
    let text = crate::ui::i18n::text(ui, text);
    let label = if let Some(ic) = icon {
        if text.trim().is_empty() {
            ic.to_string()
        } else {
            format!("{ic}  {text}")
        }
    } else {
        text.to_string()
    };

    let font_size = if icon.is_some() && text.trim().is_empty() { 13.5 } else { 12.5 };
    let font_id = egui::FontId::proportional(font_size);
    let is_empty_text = text.trim().is_empty();
    let min_w = if is_empty_text { 32.0 } else { 0.0 };
    let min_h = 32.0;

    let dummy_galley = ui.painter().layout_no_wrap(label.clone(), font_id.clone(), Color32::WHITE);
    let pad_x = if is_empty_text { 0.0 } else { 12.0 };
    let desired_size = egui::vec2((dummy_galley.size().x + pad_x * 2.0).max(min_w), min_h);

    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    let hover = ui.ctx().animate_bool(response.id.with("hov"), response.hovered());
    let press = ui.ctx().animate_bool(response.id.with("prs"), response.is_pointer_button_down_on());

    let (bg, stroke, fg) = match tone {
        ButtonTone::Primary => {
            let base_bg = theme.accent;
            let bg = if press > 0.05 {
                base_bg.linear_multiply(0.88)
            } else {
                base_bg.lerp_to_gamma(Color32::WHITE, hover * 0.15)
            };
            let stroke = if hover > 0.05 {
                Stroke::new(1.0, theme.text_on_accent.linear_multiply(0.45 * hover))
            } else {
                Stroke::NONE
            };
            (bg, stroke, theme.text_on_accent)
        }
        ButtonTone::Secondary => {
            let bg = if press > 0.05 {
                theme.accent_soft
            } else {
                theme.bg_pill.lerp_to_gamma(theme.bg_card_hover, hover)
            };
            let border_color = if press > 0.05 {
                theme.accent
            } else {
                theme.border_default.lerp_to_gamma(theme.accent, hover)
            };
            let text_color = if press > 0.05 {
                theme.accent
            } else {
                theme.text_primary.lerp_to_gamma(theme.accent, hover * 0.75)
            };
            (bg, Stroke::new(1.0 + hover * 0.25, border_color), text_color)
        }
        ButtonTone::Danger => {
            let base_bg = theme.status_danger.linear_multiply(0.12);
            let bg = if press > 0.05 {
                theme.status_danger.linear_multiply(0.35)
            } else {
                base_bg.lerp_to_gamma(theme.status_danger.linear_multiply(0.25), hover)
            };
            let stroke = Stroke::new(
                1.0 + hover * 0.25,
                theme.status_danger.linear_multiply(0.3 + hover * 0.7),
            );
            (bg, stroke, theme.status_danger)
        }
        ButtonTone::Ghost => {
            let bg = if press > 0.05 {
                theme.accent_soft
            } else {
                Color32::TRANSPARENT.lerp_to_gamma(theme.accent_soft.linear_multiply(0.7), hover)
            };
            let stroke = if hover > 0.05 {
                Stroke::new(1.0, theme.accent.linear_multiply(0.35 * hover))
            } else {
                Stroke::NONE
            };
            let text_color = theme.text_secondary.lerp_to_gamma(theme.accent, hover);
            (bg, stroke, text_color)
        }
    };

    let draw_rect = rect.shrink(press * 0.5);
    ui.painter().rect(draw_rect, 7.0, bg, stroke, egui::StrokeKind::Inside);

    let real_galley = ui.painter().layout_no_wrap(label, font_id, fg);
    let text_pos = egui::pos2(
        draw_rect.center().x - real_galley.size().x * 0.5,
        draw_rect.center().y - real_galley.size().y * 0.5,
    );
    ui.painter().galley(text_pos, real_galley, fg);

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn segmented_picker<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    selected: &mut T,
    options: &[(T, &str)],
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let total_w = ui.available_width();
    let height = 34.0;
    let (container_rect, _) = ui.allocate_exact_size(egui::vec2(total_w, height), egui::Sense::hover());

    ui.painter().rect_filled(container_rect, 8.0, theme.bg_pill);
    ui.painter().rect_stroke(
        container_rect,
        8.0,
        Stroke::new(1.0, theme.border_default),
        egui::StrokeKind::Inside,
    );

    let count = options.len() as f32;
    if count == 0.0 {
        return false;
    }

    let pad = 3.0;
    let seg_w = (container_rect.width() - pad * 2.0) / count;
    let seg_h = container_rect.height() - pad * 2.0;

    for (i, (val, label)) in options.iter().enumerate() {
        let label = crate::ui::i18n::text(ui, label);
        let seg_rect = egui::Rect::from_min_size(
            egui::pos2(container_rect.left() + pad + i as f32 * seg_w, container_rect.top() + pad),
            egui::vec2(seg_w, seg_h),
        );

        let is_selected = *selected == *val;
        let id = ui.auto_id_with(("seg_picker", i));
        let response = ui.interact(seg_rect, id, egui::Sense::click());

        if response.clicked() && !is_selected {
            *selected = *val;
            changed = true;
        }

        let hover = ui.ctx().animate_bool(id.with("hover"), response.hovered() && !is_selected);
        let active = ui.ctx().animate_bool(id.with("active"), is_selected);

        if active > 0.0 {
            ui.painter().rect_filled(seg_rect, 6.0, theme.bg_card_active);
            ui.painter().rect_stroke(
                seg_rect,
                6.0,
                Stroke::new(1.0, theme.accent),
                egui::StrokeKind::Inside,
            );
        } else if hover > 0.0 {
            ui.painter().rect_filled(
                seg_rect,
                6.0,
                theme.bg_card_hover.linear_multiply(hover),
            );
        }

        let text_color = if is_selected {
            theme.accent
        } else {
            theme.text_secondary.lerp_to_gamma(theme.text_primary, hover)
        };

        let font_id = if is_selected {
            egui::FontId::proportional(12.0)
        } else {
            egui::FontId::proportional(11.5)
        };

        ui.painter().text(
            seg_rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            font_id,
            text_color,
        );

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }

    changed
}

pub fn text_input(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    value: &mut String,
    hint: &str,
    width: Option<f32>,
    theme: &AppTheme,
) -> egui::Response {
    let desired_w = width.unwrap_or_else(|| ui.available_width());
    let height = 32.0;

    let input_id = ui.make_persistent_id(id_salt).with("inp");
    let is_focused = ui.ctx().memory(|m| m.has_focus(input_id));

    let stroke_c = if is_focused {
        theme.accent
    } else {
        theme.border_default
    };

    let frame = egui::Frame::new()
        .fill(theme.bg_pill)
        .stroke(Stroke::new(1.0, stroke_c))
        .corner_radius(6.0)
        .inner_margin(Margin::symmetric(10, 6));

    let edit = egui::TextEdit::singleline(value)
        .id(input_id)
        .frame(frame)
        .hint_text(RichText::new(crate::ui::i18n::text(ui, hint)).color(theme.text_muted).size(12.0))
        .font(egui::FontId::proportional(12.0))
        .margin(Margin::symmetric(10, 6))
        .align(egui::Align2::LEFT_CENTER);

    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(desired_w, height),
        egui::Sense::click(),
    );

    let edit_resp = ui.put(rect, edit);
    if resp.clicked() && !edit_resp.has_focus() {
        edit_resp.request_focus();
    }
    edit_resp
}

pub(super) fn card_header(
    ui: &mut egui::Ui,
    icon: &str,
    title: &str,
    subtitle: &str,
    theme: &AppTheme,
) {
    let title = crate::ui::i18n::text(ui, title);
    let subtitle = crate::ui::i18n::text(ui, subtitle);
    ui.horizontal(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::hover());
        ui.painter().rect_filled(icon_rect, 7.0, theme.accent_soft);
        ui.painter().text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(16.0),
            theme.accent,
        );
        ui.vertical(|ui| {
            ui.label(
                RichText::new(title)
                    .size(13.5)
                    .strong()
                    .color(theme.text_primary),
            );
            if !subtitle.is_empty() {
                ui.label(
                    RichText::new(subtitle)
                        .size(11.0)
                        .color(theme.text_secondary),
                );
            }
        });
    });
}

pub(super) fn section(ui: &mut egui::Ui, title: &str, subtitle: &str, theme: &AppTheme) {
    let language = crate::ui::i18n::language(ui.ctx());
    let title = language.text(title);
    let subtitle = language.text(subtitle);
    ui.add_space(8.0);
    ui.label(
        RichText::new(title)
            .size(15.0)
            .strong()
            .color(theme.text_primary),
    );
    ui.label(
        RichText::new(subtitle)
            .size(12.0)
            .color(theme.text_secondary),
    );
    ui.add_space(10.0);
}

pub fn card(ui: &mut egui::Ui, theme: &AppTheme, contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(theme.bg_panel)
        .stroke(Stroke::new(1.0, theme.border_default))
        .corner_radius(10.0)
        .inner_margin(14)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 9.0;
            contents(ui);
        });
    ui.add_space(8.0);
}

pub fn setting_toggle(
    ui: &mut egui::Ui,
    value: &mut bool,
    title: &str,
    description: &str,
    theme: &AppTheme,
) {
    let language = crate::ui::i18n::language(ui.ctx());
    let title = language.text(title);
    let description = language.text(description);
    let has_desc = !description.trim().is_empty();
    let text_width = (ui.available_width() - 80.0).max(80.0);
    let detail = if has_desc {
        Some(ui.painter().layout(
            description.into(),
            egui::FontId::proportional(11.5),
            theme.text_secondary,
            text_width,
        ))
    } else {
        None
    };

    let height = if let Some(ref d) = detail {
        (38.0 + d.size().y).max(52.0f32)
    } else {
        38.0
    };

    let (rect, mut response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::click(),
    );
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::Role::CheckBox, ui.is_enabled(), *value, title)
    });

    let hover = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());
    let active = ui.ctx().animate_bool(response.id.with("on"), *value);

    let bg = if hover > 0.0 {
        theme.bg_panel.lerp_to_gamma(theme.bg_card_hover, hover * 0.8)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 6.0, bg);

    if has_desc {
        ui.painter().text(
            rect.min + egui::vec2(8.0, 8.0),
            egui::Align2::LEFT_TOP,
            title,
            egui::FontId::proportional(13.0),
            theme.text_primary,
        );
        if let Some(d) = detail {
            ui.painter().galley(
                rect.min + egui::vec2(8.0, 27.0),
                d,
                theme.text_secondary,
            );
        }
    } else {
        ui.painter().text(
            egui::pos2(rect.min.x + 8.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            title,
            egui::FontId::proportional(13.0),
            theme.text_primary,
        );
    }

    let switch = egui::Rect::from_center_size(
        egui::pos2(rect.right() - 26.0, rect.center().y),
        egui::vec2(38.0, 22.0),
    );
    let switch_bg = theme.bg_pill.lerp_to_gamma(theme.accent, active);
    ui.painter().rect_filled(switch, 11.0, switch_bg);
    ui.painter().rect_stroke(
        switch,
        11.0,
        Stroke::new(1.0, theme.border_default.lerp_to_gamma(theme.accent, active)),
        egui::StrokeKind::Inside,
    );

    let thumb_center = egui::pos2(switch.left() + 11.0 + 16.0 * active, switch.center().y);
    ui.painter().circle_filled(thumb_center, 8.0, Color32::WHITE);

    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            6.0,
            Stroke::new(1.0, theme.accent),
            egui::StrokeKind::Inside,
        );
    }
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(description);
}

pub(crate) fn appearance_card(
    ui: &mut egui::Ui,
    palette: &AppTheme,
    label: &str,
    selected: bool,
    width: f32,
    theme: &AppTheme,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 114.0), egui::Sense::click());
    let hover = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());
    let active = ui
        .ctx()
        .animate_bool(response.id.with("selected"), selected);
    ui.painter().rect_filled(
        rect,
        9.0,
        theme
            .bg_panel
            .lerp_to_gamma(theme.accent_soft, active * 0.65),
    );
    ui.painter().rect_stroke(
        rect,
        9.0,
        Stroke::new(
            1.0 + active,
            theme
                .border_default
                .lerp_to_gamma(theme.accent, active.max(hover * 0.5)),
        ),
        egui::StrokeKind::Inside,
    );
    let preview = egui::Rect::from_min_max(
        rect.min + egui::vec2(10.0, 10.0),
        rect.max - egui::vec2(10.0, 32.0),
    );
    ui.painter().rect_filled(preview, 5.0, palette.bg_app);
    ui.painter().rect_filled(
        egui::Rect::from_min_size(preview.min, egui::vec2(preview.width(), 12.0)),
        4.0,
        palette.bg_panel,
    );
    ui.painter()
        .circle_filled(preview.min + egui::vec2(8.0, 6.0), 2.0, palette.accent);
    let columns_width = preview.width() - 24.0;
    for (offset, width) in [
        (7.0, columns_width * 0.27),
        (12.0 + columns_width * 0.27, columns_width * 0.46),
        (17.0 + columns_width * 0.73, columns_width * 0.27),
    ] {
        let panel = egui::Rect::from_min_size(
            preview.min + egui::vec2(offset, 18.0),
            egui::vec2(width, 43.0),
        );
        ui.painter().rect_filled(panel, 3.0, palette.bg_card);
        for row in 0..3 {
            let line = egui::Rect::from_min_size(
                panel.min + egui::vec2(5.0, 7.0 + row as f32 * 10.0),
                egui::vec2(width - 10.0, 4.0),
            );
            ui.painter().rect_filled(
                line,
                2.0,
                if row == 1 {
                    palette.accent_soft
                } else {
                    palette.bg_pill
                },
            );
        }
    }
    ui.painter().text(
        rect.left_bottom() + egui::vec2(12.0, -15.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.5),
        theme.text_primary,
    );
    if selected {
        ui.painter().text(
            rect.right_bottom() + egui::vec2(-15.0, -15.0),
            egui::Align2::CENTER_CENTER,
            egui_phosphor::regular::CHECK,
            egui::FontId::proportional(14.0),
            theme.accent,
        );
    }
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn profile_card(
    ui: &mut egui::Ui,
    title: &str,
    detail: &str,
    icon: &str,
    photo: Option<&[u8]>,
    selected: bool,
    theme: &AppTheme,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(192.0, 84.0), egui::Sense::click());
    let hover = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());
    let active = ui
        .ctx()
        .animate_bool(response.id.with("selected"), selected);
    ui.painter().rect_filled(
        rect,
        10.0,
        theme
            .bg_pill
            .lerp_to_gamma(theme.bg_card_hover, hover)
            .lerp_to_gamma(theme.accent_soft, active),
    );
    ui.painter().rect_stroke(
        rect,
        10.0,
        Stroke::new(
            1.0 + active,
            theme.border_default.lerp_to_gamma(theme.accent, active),
        ),
        egui::StrokeKind::Inside,
    );
    crate::ui::components::avatar::paint(
        ui,
        egui::Rect::from_center_size(rect.min + egui::vec2(26.0, 26.0), egui::vec2(32.0, 32.0)),
        photo,
        icon,
        theme,
        false,
    );
    if selected {
        ui.painter().text(
            rect.right_top() + egui::vec2(-14.0, 16.0),
            egui::Align2::CENTER_CENTER,
            egui_phosphor::regular::CHECK,
            egui::FontId::proportional(13.0),
            theme.accent,
        );
    }
    let name = ui.painter().layout_no_wrap(
        title.into(),
        egui::FontId::proportional(12.5),
        theme.text_primary,
    );
    let clipped = ui.painter().with_clip_rect(rect.shrink(10.0));
    clipped.galley(rect.min + egui::vec2(12.0, 46.0), name, theme.text_primary);
    clipped.text(
        rect.left_bottom() + egui::vec2(12.0, -9.0),
        egui::Align2::LEFT_BOTTOM,
        detail,
        egui::FontId::proportional(10.0),
        theme.text_secondary,
    );
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(title)
        .clicked()
}
