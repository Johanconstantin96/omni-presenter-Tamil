use crate::ui::AppTheme;
use eframe::egui::{self, Margin, RichText};

pub fn search_field(
    ui: &mut egui::Ui,
    id: impl egui::AsIdSalt,
    query: &mut String,
    hint: &str,
    width: f32,
    theme: &AppTheme,
) -> egui::Response {
    let field_w = width.max(100.0);
    let field_h = 32.0;

    let salt_id = ui.make_persistent_id(id);
    let input_id = salt_id.with("search_input");
    let is_focused = ui.ctx().memory(|m| m.has_focus(input_id));

    let stroke_c = if is_focused {
        theme.accent
    } else {
        theme.border_default
    };

    let frame = egui::Frame::new()
        .fill(theme.bg_pill)
        .stroke(egui::Stroke::new(1.0, stroke_c))
        .corner_radius(6.0)
        .inner_margin(Margin::symmetric(10, 6));

    let icon_c = if is_focused {
        theme.accent
    } else {
        theme.text_muted
    };

    let prefix_text = RichText::new(format!("{}  ", egui_phosphor::regular::MAGNIFYING_GLASS))
        .size(13.0)
        .color(icon_c);

    let edit = egui::TextEdit::singleline(query)
        .id(input_id)
        .frame(frame)
        .prefix(prefix_text)
        .hint_text(RichText::new(crate::ui::i18n::text(ui, hint)).color(theme.text_muted).size(12.0))
        .font(egui::FontId::proportional(12.0))
        .margin(Margin::symmetric(10, 6))
        .align(egui::Align2::LEFT_CENTER);

    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(field_w, field_h),
        egui::Sense::click(),
    );

    let edit_resp = ui.put(rect, edit);

    // Clear 'X' button overlay when query is not empty
    if !query.is_empty() {
        let btn_size = egui::vec2(18.0, 18.0);
        let btn_rect = egui::Rect::from_center_size(
            egui::pos2(rect.right() - 14.0, rect.center().y),
            btn_size,
        );
        let clear_resp = ui.put(
            btn_rect,
            egui::Button::new(
                RichText::new(egui_phosphor::regular::X)
                    .size(11.0)
                    .color(theme.text_muted),
            )
            .frame(false),
        );
        if clear_resp.clicked() {
            query.clear();
            edit_resp.request_focus();
        }
    }

    if resp.clicked() && !edit_resp.has_focus() {
        edit_resp.request_focus();
    }

    edit_resp
}

#[derive(Clone, Copy)]
pub enum ButtonTone {
    Neutral,
    Primary,
    Danger,
}

pub fn text_button(
    ui: &mut egui::Ui,
    text: &str,
    tone: ButtonTone,
    theme: &AppTheme,
) -> egui::Response {
    let text = crate::ui::i18n::text(ui, text);
    let font_id = egui::FontId::proportional(12.5);
    let dummy_galley = ui.painter().layout_no_wrap(text.to_string(), font_id.clone(), egui::Color32::WHITE);
    let pad_x = 14.0;
    let desired_size = egui::vec2(dummy_galley.size().x + pad_x * 2.0, 32.0);

    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    let hover = ui.ctx().animate_bool(response.id.with("hov"), response.hovered());
    let press = ui.ctx().animate_bool(response.id.with("prs"), response.is_pointer_button_down_on());

    let (bg, stroke, fg) = match tone {
        ButtonTone::Primary => {
            let base_bg = theme.accent;
            let bg = if press > 0.05 {
                base_bg.linear_multiply(0.88)
            } else {
                base_bg.lerp_to_gamma(egui::Color32::WHITE, hover * 0.15)
            };
            let stroke = if hover > 0.05 {
                egui::Stroke::new(1.0, theme.text_on_accent.linear_multiply(0.45 * hover))
            } else {
                egui::Stroke::NONE
            };
            (bg, stroke, theme.text_on_accent)
        }
        ButtonTone::Neutral => {
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
            (bg, egui::Stroke::new(1.0 + hover * 0.25, border_color), text_color)
        }
        ButtonTone::Danger => {
            let base_bg = theme.status_danger.linear_multiply(0.12);
            let bg = if press > 0.05 {
                theme.status_danger.linear_multiply(0.35)
            } else {
                base_bg.lerp_to_gamma(theme.status_danger.linear_multiply(0.25), hover)
            };
            let stroke = egui::Stroke::new(
                1.0 + hover * 0.25,
                theme.status_danger.linear_multiply(0.3 + hover * 0.7),
            );
            (bg, stroke, theme.status_danger)
        }
    };

    let draw_rect = rect.shrink(press * 0.5);
    ui.painter().rect(draw_rect, 7.0, bg, stroke, egui::StrokeKind::Inside);

    let real_galley = ui.painter().layout_no_wrap(text.to_string(), font_id, fg);
    let text_pos = egui::pos2(
        draw_rect.center().x - real_galley.size().x * 0.5,
        draw_rect.center().y - real_galley.size().y * 0.5,
    );
    ui.painter().galley(text_pos, real_galley, fg);

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn section_heading(ui: &mut egui::Ui, text: &str, theme: &AppTheme) {
    let text = crate::ui::i18n::text(ui, text);
    ui.label(
        RichText::new(text)
            .size(10.5)
            .strong()
            .color(theme.text_muted),
    );
    ui.add_space(4.0);
}
