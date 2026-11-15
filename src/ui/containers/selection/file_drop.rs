use crate::{core::state::ControlPanelState, ui::AppTheme};
use eframe::egui;
#[cfg(test)]
mod tests;

fn is_importable_package(ext: &str) -> bool {
    matches!(
        ext,
        "opsong"
            | "optext"
            | "opdesign"
            | "optheme"
            | "opplaylist"
            | "hinos"
            | "hino"
            | "letras"
            | "lrc"
            | "txt"
            | "omnipresenter-lyrics"
            | "omnipresenter-text"
            | "omnipresenter-design"
            | "omnipresenter-theme"
            | "omnipresenter-playlist"
            | "omnipresenter-backup"
    )
}

pub(crate) fn handle(ui: &egui::Ui, state: &mut ControlPanelState, rect: egui::Rect) {
    if !ui.is_enabled() || !ui.rect_contains_pointer(rect) {
        return;
    }
    let hovering = ui.input(|input| !input.raw.hovered_files.is_empty());
    let paths = ui.input_mut(|input| {
        if input.raw.dropped_files.is_empty() {
            return Vec::new();
        }
        std::mem::take(&mut input.raw.dropped_files)
            .into_iter()
            .map(|file| file.path().to_path_buf())
            .filter(|path| !path.as_os_str().is_empty())
            .collect::<Vec<_>>()
    });
    if !paths.is_empty() {
        let mut media_paths = Vec::new();
        for p in paths {
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            if is_importable_package(ext.as_str()) {
                state.pending_file_imports.push(p);
            } else {
                media_paths.push(p);
            }
        }
        if !media_paths.is_empty() {
            super::actions::import_media_paths(state, media_paths);
        }
    }
    if hovering {
        paint(ui, rect, &state.theme);
    }
}

/// Import drops not already handled by a more specific workspace target.
pub(crate) fn handle_remaining(ctx: &egui::Context, state: &mut ControlPanelState) {
    if state.media_conversion_prompt.is_some() { return; }
    let paths = ctx.input_mut(|input| std::mem::take(&mut input.raw.dropped_files)
        .into_iter().map(|file| file.path().to_path_buf())
        .filter(|path| !path.as_os_str().is_empty()).collect::<Vec<_>>());
    if !paths.is_empty() {
        let mut media_paths = Vec::new();
        for p in paths {
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            if is_importable_package(ext.as_str()) {
                state.pending_file_imports.push(p);
            } else {
                media_paths.push(p);
            }
        }
        if !media_paths.is_empty() {
            super::actions::import_media_paths(state, media_paths);
        }
    }
}

fn paint(ui: &egui::Ui, rect: egui::Rect, theme: &AppTheme) {
    let painter = ui
        .ctx()
        .layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            ui.scope_id().with("media_file_drop"),
        ))
        .with_clip_rect(rect.intersect(ui.clip_rect()));
    painter.rect_filled(rect.shrink(2.0), 8.0, theme.bg_panel.gamma_multiply(0.94));
    painter.rect_stroke(
        rect.shrink(3.0),
        8.0,
        egui::Stroke::new(2.0, theme.accent),
        egui::StrokeKind::Inside,
    );
    painter.text(
        rect.center() - egui::vec2(0.0, 15.0),
        egui::Align2::CENTER_CENTER,
        egui_phosphor::regular::UPLOAD_SIMPLE,
        egui::FontId::proportional(27.0),
        theme.accent,
    );
    painter.text(
        rect.center() + egui::vec2(0.0, 18.0),
        egui::Align2::CENTER_CENTER,
        crate::ui::i18n::text(ui, "drag-drop-media"),
        egui::FontId::proportional(12.0),
        theme.text_primary,
    );
}
