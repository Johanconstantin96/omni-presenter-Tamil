use super::DesignEditor;
use crate::core::design::{new_id, DesignLayer, LayerContent};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Key, Rect};
use std::path::Path;

pub fn import_image_file(path: &Path) -> Result<String, String> {
    let image = image::open(path)
        .map_err(|e| format!("Falha ao abrir imagem: {e}"))?;
    let image = if image.width() > 2560 || image.height() > 2560 {
        image.thumbnail(2560, 2560)
    } else {
        image
    };
    let folder = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("design-assets");
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let target = folder.join(format!("{}.png", new_id()));
    image.save(&target).map_err(|e| e.to_string())?;
    Ok(target.to_string_lossy().into_owned())
}

pub fn import_image_bytes(bytes: &[u8]) -> Result<String, String> {
    let image = image::load_from_memory(bytes)
        .map_err(|e| format!("Falha ao processar dados da imagem: {e}"))?;
    let image = if image.width() > 2560 || image.height() > 2560 {
        image.thumbnail(2560, 2560)
    } else {
        image
    };
    let folder = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("design-assets");
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let target = folder.join(format!("{}.png", new_id()));
    image.save(&target).map_err(|e| e.to_string())?;
    Ok(target.to_string_lossy().into_owned())
}

pub fn get_clipboard_image() -> Option<image::DynamicImage> {
    let mut clip = arboard::Clipboard::new().ok()?;
    let img_data = clip.get_image().ok()?;
    let width = img_data.width as u32;
    let height = img_data.height as u32;
    let rgba = image::RgbaImage::from_raw(width, height, img_data.bytes.into_owned())?;
    let dyn_img = image::DynamicImage::ImageRgba8(rgba);
    if width > 2560 || height > 2560 {
        Some(dyn_img.thumbnail(2560, 2560))
    } else {
        Some(dyn_img)
    }
}

pub fn save_clipboard_image(image: &image::DynamicImage) -> Result<String, String> {
    let folder = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("design-assets");
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let target = folder.join(format!("{}.png", new_id()));
    image.save(&target).map_err(|e| e.to_string())?;
    Ok(target.to_string_lossy().into_owned())
}

pub fn save_color_image(color_image: &egui::ColorImage) -> Result<String, String> {
    let width = color_image.width() as u32;
    let height = color_image.height() as u32;
    if width == 0 || height == 0 {
        return Err("Imagem vazia".to_string());
    }
    let mut rgba_bytes = Vec::with_capacity((width * height * 4) as usize);
    for pixel in &color_image.pixels {
        rgba_bytes.extend_from_slice(&pixel.to_array());
    }
    let rgba = image::RgbaImage::from_raw(width, height, rgba_bytes)
        .ok_or_else(|| "Falha ao processar ColorImage".to_string())?;
    let dyn_img = image::DynamicImage::ImageRgba8(rgba);
    let dyn_img = if width > 2560 || height > 2560 {
        dyn_img.thumbnail(2560, 2560)
    } else {
        dyn_img
    };
    let folder = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("design-assets");
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let target = folder.join(format!("{}.png", new_id()));
    dyn_img.save(&target).map_err(|e| e.to_string())?;
    Ok(target.to_string_lossy().into_owned())
}

pub fn get_clipboard_text() -> Option<String> {
    let mut clip = arboard::Clipboard::new().ok()?;
    clip.get_text().ok()
}

pub fn set_clipboard_text(text: &str) {
    if let Ok(mut clip) = arboard::Clipboard::new() {
        let _ = clip.set_text(text);
    }
}

impl DesignEditor {
    pub fn copy_selected(&mut self, ctx: Option<&egui::Context>) {
        let Some(scene) = &self.scene else { return; };
        let mut indices: Vec<usize> = if !self.selected_layers.is_empty() {
            self.selected_layers.iter().copied().collect()
        } else if let Some(sel) = self.selected {
            vec![sel]
        } else {
            Vec::new()
        };
        indices.sort_unstable();
        if indices.is_empty() {
            return;
        }

        self.clipboard_layers.clear();
        for &idx in &indices {
            if let Some(layer) = scene.layers.get(idx) {
                self.clipboard_layers.push(layer.clone());
            }
        }

        if self.clipboard_layers.len() == 1 {
            if let LayerContent::Text(text) = &self.clipboard_layers[0].content {
                if let Some(ctx) = ctx {
                    ctx.copy_text(text.clone());
                }
                set_clipboard_text(text);
            } else if let Ok(json) = serde_json::to_string(&self.clipboard_layers) {
                let payload = format!("omnipresenter:layers:{json}");
                if let Some(ctx) = ctx {
                    ctx.copy_text(payload.clone());
                }
                set_clipboard_text(&payload);
            }
        } else if !self.clipboard_layers.is_empty()
            && let Ok(json) = serde_json::to_string(&self.clipboard_layers)
        {
            let payload = format!("omnipresenter:layers:{json}");
            if let Some(ctx) = ctx {
                ctx.copy_text(payload.clone());
            }
            set_clipboard_text(&payload);
        }
    }

    pub fn cut_selected(&mut self, ctx: Option<&egui::Context>) {
        self.copy_selected(ctx);
        let before = self.scene.clone();
        self.delete_layer();
        if let Some(before) = before {
            self.changed(before, true);
        }
    }

    pub fn paste(&mut self) {
        self.paste_payload(None, None);
    }

    pub fn paste_payload(
        &mut self,
        pasted_text: Option<String>,
        pasted_image: Option<std::sync::Arc<egui::ColorImage>>,
    ) {
        let before = self.scene.clone();
        let Some(scene) = self.scene.as_mut() else { return; };

        // 1. Image payload from egui PasteImage event
        if let Some(color_img) = pasted_image
            && let Ok(path) = save_color_image(&color_img)
        {
            let mut layer = DesignLayer::new(LayerContent::Image { path, cover: true });
            layer.rect = [0.15, 0.15, 0.7, 0.7];
            let idx = scene.layers.len();
            scene.layers.push(layer);
            self.selected = Some(idx);
            self.selected_layers.clear();
            self.selected_layers.insert(idx);
            self.inspector_tab = 1;
            if let Some(before) = before {
                self.changed(before, true);
            }
            return;
        }

        // 2. Direct image from system clipboard via arboard
        if let Some(img) = get_clipboard_image() {
            match save_clipboard_image(&img) {
                Ok(path) => {
                    let mut layer = DesignLayer::new(LayerContent::Image { path, cover: true });
                    layer.rect = [0.15, 0.15, 0.7, 0.7];
                    let idx = scene.layers.len();
                    scene.layers.push(layer);
                    self.selected = Some(idx);
                    self.selected_layers.clear();
                    self.selected_layers.insert(idx);
                    self.inspector_tab = 1;
                    if let Some(before) = before {
                        self.changed(before, true);
                    }
                    return;
                }
                Err(err) => {
                    self.error = err;
                }
            }
        }

        // 3. Text payload from event or system clipboard
        let text_candidate = pasted_text.or_else(get_clipboard_text);
        if let Some(text) = text_candidate {
            let trimmed = text.trim();
            // A. Serialized omni-presenter layers
            if let Some(json) = trimmed.strip_prefix("omnipresenter:layers:")
                && let Ok(external_layers) = serde_json::from_str::<Vec<DesignLayer>>(json)
                && !external_layers.is_empty()
            {
                self.clipboard_layers = external_layers;
            } else {
                // B. Image file path from file explorer
                let path_str = trimmed.strip_prefix("file://").unwrap_or(trimmed);
                let path = Path::new(path_str);
                if path.is_file()
                    && let Some(ext) = path.extension().and_then(|s| s.to_str())
                    && matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp")
                    && let Ok(saved_path) = import_image_file(path)
                {
                    let mut layer = DesignLayer::new(LayerContent::Image { path: saved_path, cover: true });
                    layer.rect = [0.15, 0.15, 0.7, 0.7];
                    let idx = scene.layers.len();
                    scene.layers.push(layer);
                    self.selected = Some(idx);
                    self.selected_layers.clear();
                    self.selected_layers.insert(idx);
                    self.inspector_tab = 1;
                    if let Some(before) = before {
                        self.changed(before, true);
                    }
                    return;
                }
            }
        }

        // 4. In-memory clipboard layers
        if !self.clipboard_layers.is_empty() {
            let mut gid_map = std::collections::HashMap::new();
            let mut new_selection = std::collections::BTreeSet::new();
            let base_offset = 0.03;

            for original in &self.clipboard_layers {
                let mut layer = original.clone();
                layer.id = new_id();
                layer.rect[0] = (layer.rect[0] + base_offset).min(1.0 - layer.rect[2].min(0.99)).max(0.0);
                layer.rect[1] = (layer.rect[1] + base_offset).min(1.0 - layer.rect[3].min(0.99)).max(0.0);
                if let Some(gid) = &layer.group_id {
                    let new_gid = gid_map.entry(gid.clone()).or_insert_with(new_id).clone();
                    layer.group_id = Some(new_gid);
                }
                let idx = scene.layers.len();
                scene.layers.push(layer);
                new_selection.insert(idx);
            }
            self.selected = new_selection.iter().next_back().copied();
            self.selected_layers = new_selection;
            self.inspector_tab = 1;
            if let Some(before) = before {
                self.changed(before, true);
            }
            return;
        }

        // 5. Plain text layer fallback
        let text_candidate = get_clipboard_text();
        if let Some(text) = text_candidate {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                let mut layer = DesignLayer::new(LayerContent::Text(trimmed.to_string()));
                layer.rect = [0.15, 0.35, 0.7, 0.3];
                let idx = scene.layers.len();
                scene.layers.push(layer);
                self.selected = Some(idx);
                self.selected_layers.clear();
                self.selected_layers.insert(idx);
                self.inspector_tab = 1;
                if let Some(before) = before {
                    self.changed(before, true);
                }
            }
        }
    }

    pub fn select_all_layers(&mut self) {
        let Some(scene) = &self.scene else { return; };
        self.selected_layers = (0..scene.layers.len())
            .filter(|&i| scene.layers[i].visible)
            .collect();
        self.selected = self.selected_layers.iter().next_back().copied();
    }

    pub fn nudge_selected(&mut self, dx: f32, dy: f32) {
        let before = self.scene.clone();
        let Some(scene) = self.scene.as_mut() else { return; };
        let mut moved = false;
        for &idx in &self.selected_layers {
            if let Some(layer) = scene.layers.get_mut(idx)
                && !layer.locked
            {
                layer.rect[0] = (layer.rect[0] + dx).clamp(0.0, 1.0 - layer.rect[2].min(1.0));
                layer.rect[1] = (layer.rect[1] + dy).clamp(0.0, 1.0 - layer.rect[3].min(1.0));
                moved = true;
            }
        }
        if moved && let Some(before) = before {
            self.changed(before, true);
        }
    }
}

pub fn handle_canvas_keyboard_shortcuts(
    ui: &mut egui::Ui,
    editor: &mut DesignEditor,
    canvas_rect: Rect,
) {
    if editor.scene.is_none() {
        return;
    }
    // Never intercept shortcuts when user is typing inside an active TextEdit
    if ui.ctx().text_edit_focused() {
        return;
    }

    let mut copy_requested = false;
    let mut cut_requested = false;
    let mut paste_text = None;
    let mut paste_image = None;
    let mut paste_key_requested = false;
    let mut duplicate_requested = false;
    let mut select_all_requested = false;
    let mut escape_requested = false;
    let mut delete_requested = false;
    let mut undo_requested = false;
    let mut redo_requested = false;
    let mut group_requested = false;
    let mut ungroup_requested = false;
    let mut nudge_x: f32 = 0.0;
    let mut nudge_y: f32 = 0.0;

    let step = if ui.input(|i| i.modifiers.shift) { 10.0 } else { 1.0 };
    let dx = step / canvas_rect.width().max(1.0);
    let dy = step / canvas_rect.height().max(1.0);

    ui.input_mut(|i| {
        i.events.retain(|event| {
            match event {
                egui::Event::Copy => {
                    copy_requested = true;
                    false
                }
                egui::Event::Cut => {
                    cut_requested = true;
                    false
                }
                egui::Event::Paste(text) => {
                    paste_text = Some(text.clone());
                    false
                }
                egui::Event::PasteImage(img) => {
                    paste_image = Some(img.clone());
                    false
                }
                egui::Event::Key {
                    key,
                    modifiers,
                    pressed: true,
                    ..
                } => {
                    let ctrl_or_cmd = modifiers.ctrl || modifiers.command;
                    if ctrl_or_cmd && !modifiers.shift && !modifiers.alt {
                        match key {
                            Key::C => {
                                copy_requested = true;
                                false
                            }
                            Key::X => {
                                cut_requested = true;
                                false
                            }
                            Key::V => {
                                paste_key_requested = true;
                                false
                            }
                            Key::D => {
                                duplicate_requested = true;
                                false
                            }
                            Key::A => {
                                select_all_requested = true;
                                false
                            }
                            Key::Z => {
                                undo_requested = true;
                                false
                            }
                            Key::Y => {
                                redo_requested = true;
                                false
                            }
                            Key::G => {
                                group_requested = true;
                                false
                            }
                            _ => true,
                        }
                    } else if ctrl_or_cmd && modifiers.shift && !modifiers.alt {
                        match key {
                            Key::Z => {
                                redo_requested = true;
                                false
                            }
                            Key::G => {
                                ungroup_requested = true;
                                false
                            }
                            _ => true,
                        }
                    } else if !ctrl_or_cmd && !modifiers.alt {
                        match key {
                            Key::Escape => {
                                escape_requested = true;
                                false
                            }
                            Key::Delete | Key::Backspace => {
                                delete_requested = true;
                                false
                            }
                            Key::ArrowLeft => {
                                nudge_x -= dx;
                                false
                            }
                            Key::ArrowRight => {
                                nudge_x += dx;
                                false
                            }
                            Key::ArrowUp => {
                                nudge_y -= dy;
                                false
                            }
                            Key::ArrowDown => {
                                nudge_y += dy;
                                false
                            }
                            _ => true,
                        }
                    } else {
                        true
                    }
                }
                _ => true,
            }
        });
    });

    if copy_requested {
        editor.copy_selected(Some(ui.ctx()));
    } else if cut_requested {
        editor.cut_selected(Some(ui.ctx()));
    } else if paste_image.is_some() || paste_text.is_some() || paste_key_requested {
        editor.paste_payload(paste_text, paste_image);
    } else if duplicate_requested {
        let before = editor.scene.clone();
        editor.duplicate_layer();
        if let Some(before) = before {
            editor.changed(before, true);
        }
    } else if select_all_requested {
        editor.select_all_layers();
    } else if escape_requested {
        editor.clear_layer_selection();
    } else if delete_requested {
        let before = editor.scene.clone();
        editor.delete_layer();
        if let Some(before) = before {
            editor.changed(before, true);
        }
    } else if undo_requested {
        editor.history(false);
    } else if redo_requested {
        editor.history(true);
    } else if group_requested {
        editor.group_selected();
    } else if ungroup_requested {
        editor.ungroup_selected();
    } else if nudge_x != 0.0 || nudge_y != 0.0 {
        editor.nudge_selected(nudge_x, nudge_y);
    }
}

pub fn handle_canvas_file_drop(
    ui: &egui::Ui,
    editor: &mut DesignEditor,
    canvas_rect: Rect,
    theme: &AppTheme,
) {
    if editor.scene.is_none() {
        return;
    }

    let is_hovering_canvas = ui.rect_contains_pointer(canvas_rect);
    let has_hovered_files = ui.input(|i| !i.raw.hovered_files.is_empty());

    if is_hovering_canvas && has_hovered_files {
        let painter = ui
            .ctx()
            .layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                ui.scope_id().with("design_canvas_dropzone"),
            ))
            .with_clip_rect(canvas_rect);

        painter.rect_filled(
            canvas_rect.shrink(4.0),
            12.0,
            theme.bg_panel.gamma_multiply(0.88),
        );
        painter.rect_stroke(
            canvas_rect.shrink(6.0),
            12.0,
            egui::Stroke::new(2.5, theme.accent),
            egui::StrokeKind::Inside,
        );

        let center = canvas_rect.center();
        painter.text(
            center - egui::vec2(0.0, 14.0),
            egui::Align2::CENTER_CENTER,
            egui_phosphor::regular::IMAGE,
            egui::FontId::proportional(36.0),
            theme.accent,
        );
        let drop_label = editor.language.text("design-drop-image-cue");
        painter.text(
            center + egui::vec2(0.0, 20.0),
            egui::Align2::CENTER_CENTER,
            drop_label,
            egui::FontId::proportional(15.0),
            theme.text_primary,
        );
    }

    if is_hovering_canvas {
        let dropped = ui.input_mut(|i| {
            if i.raw.dropped_files.is_empty() {
                Vec::new()
            } else {
                std::mem::take(&mut i.raw.dropped_files)
            }
        });

        if !dropped.is_empty() {
            let pointer_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or(canvas_rect.center());
            let norm_x = ((pointer_pos.x - canvas_rect.min.x) / canvas_rect.width()).clamp(0.0, 1.0);
            let norm_y = ((pointer_pos.y - canvas_rect.min.y) / canvas_rect.height()).clamp(0.0, 1.0);

            for file in dropped {
                let path = file.path();
                if !path.as_os_str().is_empty() {
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                    if ext == "svg"
                        && let Ok(svg) = std::fs::read_to_string(path)
                    {
                        let before = editor.scene.clone();
                        if let Some(scene) = editor.scene.as_mut() {
                            let mut layer = DesignLayer::new(LayerContent::Vector { svg });
                            layer.rect = [
                                (norm_x - 0.25).clamp(0.0, 0.5),
                                (norm_y - 0.25).clamp(0.0, 0.5),
                                0.5,
                                0.5,
                            ];
                            let idx = scene.layers.len();
                            scene.layers.push(layer);
                            editor.selected = Some(idx);
                            editor.selected_layers.clear();
                            editor.selected_layers.insert(idx);
                            editor.inspector_tab = 1;
                            if let Some(before) = before {
                                editor.changed(before, true);
                            }
                        }
                        continue;
                    }
                    if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp")
                        && let Ok(saved_path) = import_image_file(path)
                    {
                        let before = editor.scene.clone();
                        if let Some(scene) = editor.scene.as_mut() {
                            let mut layer = DesignLayer::new(LayerContent::Image {
                                path: saved_path,
                                cover: true,
                            });
                            layer.rect = [
                                (norm_x - 0.25).clamp(0.0, 0.5),
                                (norm_y - 0.25).clamp(0.0, 0.5),
                                0.5,
                                0.5,
                            ];
                            let idx = scene.layers.len();
                            scene.layers.push(layer);
                            editor.selected = Some(idx);
                            editor.selected_layers.clear();
                            editor.selected_layers.insert(idx);
                            editor.inspector_tab = 1;
                            if let Some(before) = before {
                                editor.changed(before, true);
                            }
                        }
                        continue;
                    }
                }
                if let Ok(bytes) = file.bytes()
                    && let Ok(saved_path) = import_image_bytes(&bytes)
                {
                    let before = editor.scene.clone();
                    if let Some(scene) = editor.scene.as_mut() {
                        let mut layer = DesignLayer::new(LayerContent::Image {
                            path: saved_path,
                            cover: true,
                        });
                        layer.rect = [
                            (norm_x - 0.25).clamp(0.0, 0.5),
                            (norm_y - 0.25).clamp(0.0, 0.5),
                            0.5,
                            0.5,
                        ];
                        let idx = scene.layers.len();
                        scene.layers.push(layer);
                        editor.selected = Some(idx);
                        editor.selected_layers.clear();
                        editor.selected_layers.insert(idx);
                        editor.inspector_tab = 1;
                        if let Some(before) = before {
                            editor.changed(before, true);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::design::{DesignDocument, DesignLayer, LayerContent};

    #[test]
    fn copy_and_paste_layers_preserves_content_with_new_ids_and_offset() {
        let mut editor = DesignEditor::default();
        let mut scene = DesignDocument::template(0);
        let mut layer = DesignLayer::new(LayerContent::Text("Slide Title".into()));
        layer.rect = [0.1, 0.1, 0.4, 0.2];
        scene.layers.push(layer);
        editor.select(scene);
        editor.select_layer(0, false);

        editor.copy_selected(None);
        assert_eq!(editor.clipboard_layers.len(), 1);
        let orig_id = editor.clipboard_layers[0].id.clone();

        editor.paste();
        let scene = editor.scene.as_ref().unwrap();
        assert_eq!(scene.layers.len(), 2);
        let pasted = &scene.layers[1];
        assert_ne!(pasted.id, orig_id);
        assert!((pasted.rect[0] - 0.13).abs() < 1e-4);
        assert!((pasted.rect[1] - 0.13).abs() < 1e-4);
    }

    #[test]
    fn cut_selected_removes_layer_and_populates_clipboard() {
        let mut editor = DesignEditor::default();
        let mut scene = DesignDocument::template(0);
        scene.layers.push(DesignLayer::new(LayerContent::Text("To Cut".into())));
        editor.select(scene);
        editor.select_layer(0, false);

        editor.cut_selected(None);
        assert_eq!(editor.clipboard_layers.len(), 1);
        assert_eq!(editor.scene.as_ref().unwrap().layers.len(), 0);
    }

    #[test]
    fn select_all_and_nudge_moves_layers() {
        let mut editor = DesignEditor::default();
        let mut scene = DesignDocument::template(0);
        let mut layer = DesignLayer::new(LayerContent::Text("Move Me".into()));
        layer.rect = [0.2, 0.2, 0.3, 0.3];
        scene.layers.push(layer);
        editor.select(scene);

        editor.select_all_layers();
        assert!(editor.selected_layers.contains(&0));

        editor.nudge_selected(0.05, -0.02);
        let layer = &editor.scene.as_ref().unwrap().layers[0];
        assert!((layer.rect[0] - 0.25).abs() < 1e-4);
        assert!((layer.rect[1] - 0.18).abs() < 1e-4);
    }

    #[test]
    fn import_image_bytes_saves_valid_png() {
        let img = image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255]));
        let mut bytes = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut bytes);
        img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();

        let path_str = import_image_bytes(&bytes).expect("Should import valid png bytes");
        let path = std::path::Path::new(&path_str);
        assert!(path.exists());
        let loaded = image::open(path).expect("Saved image must be readable");
        assert_eq!(loaded.width(), 2);
        assert_eq!(loaded.height(), 2);
        let _ = std::fs::remove_file(path);
    }
}

