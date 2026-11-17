use super::DesignEditor;
use crate::{
    core::design::DesignLayer,
    ui::{AppTheme, components::*},
};
use eframe::egui;
use egui_phosphor::regular::*;

#[derive(Clone, Copy)]
pub(super) enum LayerAction {
    Edit,
    Visibility,
    Lock,
    Forward,
    Backward,
    Duplicate,
    Delete,
    Group,
    Ungroup,
}

pub(super) fn layer_menu(
    ui: &mut egui::Ui,
    layer: &DesignLayer,
    index: usize,
    count: usize,
    selected_count: usize,
    theme: &AppTheme,
) -> Option<LayerAction> {
    context_menu_header(ui);
    let mut action = None;
    for (value, glyph, label, enabled) in [
        (LayerAction::Edit, PENCIL_SIMPLE, crate::ui::i18n::text(ui, "design-edit-layer"), true),
        (
            LayerAction::Visibility,
            if layer.visible { EYE_SLASH } else { EYE },
            if layer.visible {
                crate::ui::i18n::text(ui, "design-hide-layer")
            } else {
                crate::ui::i18n::text(ui, "design-show-layer")
            },
            true,
        ),
        (
            LayerAction::Lock,
            if layer.locked {
                LOCK_SIMPLE_OPEN
            } else {
                LOCK_SIMPLE
            },
            if layer.locked {
                crate::ui::i18n::text(ui, "design-unlock-layer")
            } else {
                crate::ui::i18n::text(ui, "design-lock-layer")
            },
            true,
        ),
    ] {
        if menu_item(ui, glyph, label, enabled, theme) {
            action = Some(value);
        }
    }
    context_menu_separator(ui);
    for (value, glyph, label, enabled) in [
        (
            LayerAction::Forward,
            ARROW_UP,
            crate::ui::i18n::text(ui, "design-bring-forward"),
            !layer.locked && index + 1 < count,
        ),
        (
            LayerAction::Backward,
            ARROW_DOWN,
            crate::ui::i18n::text(ui, "design-send-backward"),
            !layer.locked && index > 0,
        ),
        (LayerAction::Duplicate, COPY, crate::ui::i18n::text(ui, "design-duplicate-layer"), true),
    ] {
        if menu_item(ui, glyph, label, enabled, theme) {
            action = Some(value);
        }
    }
    if selected_count > 1 || layer.group_id.is_some() {
        context_menu_separator(ui);
        if selected_count > 1 && menu_item(ui, FOLDER_PLUS, crate::ui::i18n::text(ui, "design-group"), true, theme) {
            action = Some(LayerAction::Group);
        }
        if layer.group_id.is_some() && menu_item(ui, FOLDER_MINUS, crate::ui::i18n::text(ui, "design-ungroup"), true, theme) {
            action = Some(LayerAction::Ungroup);
        }
    }
    context_menu_separator(ui);
    let delete_label = crate::ui::i18n::text(ui, "design-delete-layer");
    if !layer.locked {
        if context_menu_danger_item(ui, TRASH, delete_label, theme).clicked() {
            action = Some(LayerAction::Delete);
            ui.close();
        }
    } else {
        context_menu_disabled_item(ui, TRASH, delete_label, theme);
    }
    action
}

fn menu_item(ui: &mut egui::Ui, glyph: &str, label: &str, enabled: bool, theme: &AppTheme) -> bool {
    if !enabled {
        context_menu_disabled_item(ui, glyph, label, theme);
        return false;
    }
    let clicked = context_menu_item(ui, glyph, label, theme).clicked();
    if clicked {
        ui.close();
    }
    clicked
}

pub(super) fn apply_layer(editor: &mut DesignEditor, index: usize, action: LayerAction) {
    let Some(layer) = editor
        .scene
        .as_ref()
        .and_then(|scene| scene.layers.get(index))
    else {
        return;
    };
    if layer.locked
        && matches!(
            action,
            LayerAction::Forward | LayerAction::Backward | LayerAction::Delete
        )
    {
        return;
    }
    if matches!(action, LayerAction::Group) {
        editor.group_selected();
        return;
    }
    if matches!(action, LayerAction::Ungroup) {
        editor.ungroup_selected();
        return;
    }
    editor.selected = Some(index);
    editor.inspector_tab = 1;
    if matches!(action, LayerAction::Edit) {
        return;
    }
    if matches!(action, LayerAction::Forward | LayerAction::Backward) {
        editor.reorder(matches!(action, LayerAction::Forward));
        return;
    }
    let before = editor.scene.clone().unwrap();
    match action {
        LayerAction::Visibility => {
            editor.scene.as_mut().unwrap().layers[index].visible = !before.layers[index].visible
        }
        LayerAction::Lock => {
            editor.scene.as_mut().unwrap().layers[index].locked = !before.layers[index].locked
        }
        LayerAction::Duplicate => editor.duplicate_layer(),
        LayerAction::Delete => editor.delete_layer(),
        _ => {}
    }
    editor.changed(before, true);
}

pub(super) fn new_design_menu(ui: &mut egui::Ui, theme: &AppTheme) -> Option<usize> {
    context_menu_header(ui);
    let mut kind = None;
    let create = context_menu_item(ui, PLUS, "design-new-blank-design", theme);
    if create.clicked() {
        kind = Some(0);
        ui.close();
    }
    context_menu_submenu(ui, STACK, "design-create-from-template", theme, |ui| {
        for (value, glyph, label) in [
            (1, HAND_WAVING, "design-welcome-label"),
            (2, TIMER, "design-countdown"),
            (3, QUESTION, "design-questions-and-answers"),
        ] {
            if context_menu_item(ui, glyph, label, theme).clicked() {
                kind = Some(value);
                ui.close();
            }
        }
        context_menu_separator(ui);
        for (value, glyph, label) in [
            (4, CLOCK, "design-clock-overlay-label"),
            (5, RECTANGLE, "design-lower-third-overlay"),
        ] {
            if context_menu_item(ui, glyph, label, theme).clicked() {
                kind = Some(value);
                ui.close();
            }
        }
    });
    kind
}

pub(super) fn new_design_template_menu(ui: &mut egui::Ui, theme: &AppTheme) -> Option<usize> {
    ui.set_width(300.0);
    ui.spacing_mut().item_spacing = egui::vec2(4.0, 3.0);
    ui.label(
        egui::RichText::new(crate::ui::i18n::text(ui, "design-templates"))
            .size(10.5)
            .strong()
            .color(theme.text_muted),
    );
    ui.add_space(2.0);
    let mut selected = None;
    for (kind, label, description, icon, (r, g, b), tag) in [
        (0, "design-blank", "design-start-from-scratch", FILE_TEXT, (148, 163, 184), "SLIDE"),
        (1, "design-welcome-label", "design-title-and-message", HAND_WAVING, (251, 191, 36), "SLIDE"),
        (2, "design-countdown", "design-service-countdown", TIMER, (56, 189, 248), "TIMER"),
        (3, "design-questions-and-answers", "design-interactive-activity", QUESTION, (52, 211, 153), "QUIZ"),
        (4, "design-clock-overlay-label", "design-clock-overlay", CLOCK, (167, 139, 250), "EXTRA"),
        (5, "design-lower-third-overlay", "design-lower-third-banner", RECTANGLE, (251, 113, 133), "EXTRA"),
    ] {
        if kind == 4 {
            ui.add_space(3.0);
            ui.separator();
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(crate::ui::i18n::text(ui, "control-overlay-design"))
                    .size(10.0)
                    .strong()
                    .color(theme.text_muted),
            );
            ui.add_space(1.0);
        }
        let (rect, response) = ui
            .allocate_exact_size(egui::vec2(ui.available_width(), 46.0), egui::Sense::click());
        if response.hovered() {
            ui.painter().rect_filled(rect, 8.0, theme.bg_card_hover);
            ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(1.0, theme.border_subtle), egui::StrokeKind::Inside);
        }
        let badge =
            egui::Rect::from_min_size(rect.min + egui::vec2(6.0, 8.0), egui::vec2(30.0, 30.0));
        let badge_bg = egui::Color32::from_rgba_unmultiplied(r, g, b, 35);
        let badge_stroke = egui::Color32::from_rgba_unmultiplied(r, g, b, 70);
        let badge_fg = egui::Color32::from_rgb(r, g, b);
        ui.painter().rect_filled(badge, 8.0, badge_bg);
        ui.painter().rect_stroke(badge, 8.0, egui::Stroke::new(1.0, badge_stroke), egui::StrokeKind::Inside);
        ui.painter().text(
            badge.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(15.0),
            badge_fg,
        );
        let title = ui.painter().layout_no_wrap(
            crate::ui::i18n::text(ui, label).to_owned(),
            egui::FontId::proportional(12.5),
            theme.text_primary,
        );
        ui.painter().galley(
            egui::pos2(badge.right() + 10.0, rect.center().y - title.size().y + 1.0),
            title,
            theme.text_primary,
        );
        let subtitle = ui.painter().layout_no_wrap(
            crate::ui::i18n::text(ui, description).to_owned(),
            egui::FontId::proportional(10.5),
            theme.text_muted,
        );
        ui.painter().galley(
            egui::pos2(badge.right() + 10.0, rect.center().y + 2.0),
            subtitle,
            theme.text_muted,
        );

        let chip_w = 40.0;
        let chip_rect = egui::Rect::from_min_size(
            egui::pos2(rect.right() - chip_w - 6.0, rect.center().y - 8.0),
            egui::vec2(chip_w, 16.0),
        );
        let (chip_bg, chip_fg) = if tag == "EXTRA" {
            (egui::Color32::from_rgba_unmultiplied(139, 92, 246, 35), egui::Color32::from_rgb(196, 181, 253))
        } else {
            (egui::Color32::from_rgba_unmultiplied(100, 116, 139, 25), egui::Color32::from_rgb(148, 163, 184))
        };
        ui.painter().rect_filled(chip_rect, 4.0, chip_bg);
        ui.painter().text(
            chip_rect.center(),
            egui::Align2::CENTER_CENTER,
            tag,
            egui::FontId::proportional(9.0),
            chip_fg,
        );

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if response.clicked() {
            selected = Some(kind);
            ui.close();
        }
    }
    selected
}
