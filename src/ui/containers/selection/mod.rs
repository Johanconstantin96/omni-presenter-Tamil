mod actions;
mod conversion;
pub(crate) mod file_drop;
mod model;
#[cfg(test)]
mod tests;
mod toolbar;

use crate::{
    core::{bible::fold, slides::SlideDeck, state::ControlPanelState},
    services::db::AppDb,
    ui::components::*,
};
use actions::LibraryAction;
pub use actions::import_media;
pub use actions::poll_media_import;
pub use conversion::render_media_conversion;
use eframe::egui;
use egui_phosphor::regular::*;
use model::{LibraryEntry, LibraryKind};

#[derive(Clone, Default)]
struct LibrarySearchCache {
    tab: usize,
    count: usize,
    first_id: Option<i64>,
    last_id: Option<i64>,
    entries: std::sync::Arc<[(i64, String)]>,
}

pub fn render_selection_column(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
    db: Option<&AppDb>,
) {
    file_drop::handle(ui, state, ui.available_rect_before_wrap());
    toolbar::toolbar(ui, state, deck, db);
    if state.selected_tab == 4 {
        crate::ui::pages::design::library::render_selection(ui, state, deck, presenting, db);
        return;
    }
    let kind = LibraryKind::from_tab(state.selected_tab);
    let entries = model::entries(kind, state, deck);
    let folders = model::collect_folders(kind, state, &entries, db);
    let query_trimmed = state.search_query.trim();
    let query = fold(query_trimmed);

    let matching_ids: Option<std::collections::HashSet<i64>> = if query.is_empty() {
        None
    } else {
        let cache_id = ui.make_persistent_id("library_search_folded_cache");
        let mut cache = ui
            .ctx()
            .data_mut(|d| d.get_temp::<LibrarySearchCache>(cache_id))
            .unwrap_or_default();
        let current_fingerprint = (
            state.selected_tab,
            entries.len(),
            entries.first().map(|e| e.id),
            entries.last().map(|e| e.id),
        );
        if (cache.tab, cache.count, cache.first_id, cache.last_id) != current_fingerprint {
            cache.tab = current_fingerprint.0;
            cache.count = current_fingerprint.1;
            cache.first_id = current_fingerprint.2;
            cache.last_id = current_fingerprint.3;
            let raw_entries: Vec<(i64, String)> = match kind {
                LibraryKind::Song => deck
                    .library()
                    .iter()
                    .enumerate()
                    .map(|(index, song)| {
                        let id = song.id.unwrap_or(-(index as i64 + 1));
                        let text = format!("{} {} {} {} {}", song.title, song.artist, song.folder, song.tags, song.text);
                        (id, fold(&text))
                    })
                    .collect(),
                _ => entries
                    .iter()
                    .map(|e| {
                        let text = format!("{} {} {} {} {}", e.title, e.subtitle, e.folder, e.tags, e.content);
                        (e.id, fold(&text))
                    })
                    .collect(),
            };
            cache.entries = raw_entries.into();
            ui.ctx().data_mut(|d| d.insert_temp(cache_id, cache.clone()));
        }

        let tokens: Vec<&str> = query.split_whitespace().collect();
        Some(
            cache
                .entries
                .iter()
                .filter(|(_, text)| tokens.iter().all(|token| text.contains(token)))
                .map(|(id, _)| *id)
                .collect(),
        )
    };

    let filtered = model::filter_and_sort(
        &entries,
        kind,
        &query,
        &state.tag_filter,
        matching_ids.as_ref(),
    );

    let mut visible = Vec::new();
    if query.is_empty() {
        for folder in &folders {
            if kind.folders(state).contains(folder) {
                visible.extend(
                    filtered
                        .iter()
                        .filter(|item| item.folder == folder.as_str())
                        .map(|item| item.id),
                );
            }
        }
        visible.extend(
            filtered
                .iter()
                .filter(|item| item.folder.is_empty())
                .map(|item| item.id),
        );
    } else {
        visible.extend(filtered.iter().map(|item| item.id));
    }
    let mut action = None;
    if ui.is_enabled()
        && !ui.ctx().egui_wants_keyboard_input()
        && !egui::DragAndDrop::has_any_payload(ui.ctx())
        && ui.ctx().memory(|memory| memory.top_modal_layer().is_none())
    {
        if ui.input(|input| {
            (input.modifiers.ctrl || input.modifiers.command) && input.key_pressed(egui::Key::A)
        }) {
            *kind.selected_mut(state) = visible.iter().copied().collect();
            *kind.anchor(state) = visible.first().copied();
        }
        if !kind.selected(state).is_empty()
            && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            kind.clear(state);
        }
        if ui.input(|input| input.key_pressed(egui::Key::Delete))
            && !kind.selected(state).is_empty()
        {
            action = Some(LibraryAction::RequestDelete(
                kind.selected(state).iter().copied().collect(),
            ));
        }
    }
    selection_bar(ui, state, kind, &folders, &mut action);
    let theme = state.theme.clone();
    let language = state.settings.language;
    egui::ScrollArea::vertical()
        .id_salt("selection_scroll_area")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if query.is_empty() {
                for folder in &folders {
                    let items: Vec<_> = filtered
                        .iter()
                        .filter(|item| item.folder == folder.as_str())
                        .copied()
                        .collect();
                    let open = kind.folders(state).contains(folder);
                let folder_id =
                    ui.make_persistent_id(("library_folder_btn", kind.category(), folder));
                let (f_rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 28.0),
                    egui::Sense::click(),
                );

                let pointer_pos = ui.ctx().pointer_latest_pos().or_else(|| ui.ctx().pointer_interact_pos());
                let is_folder_hovered =
                    pointer_pos.is_some_and(|pt| f_rect.expand2(egui::vec2(0.0, 4.0)).contains(pt));
                let drag_payload =
                    egui::DragAndDrop::payload::<crate::ui::components::drag_drop::DragPayload>(
                        ui.ctx(),
                    );
                let is_drag_library = drag_payload.as_ref().is_some_and(|p| {
                    matches!(
                        p.source,
                        crate::ui::components::drag_drop::DragSource::Library
                    ) && (!p.library_ids.is_empty() || !p.items.is_empty())
                });
                let is_drop_target_active = is_folder_hovered && is_drag_library;

                if is_drop_target_active {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Copy);
                    let pointer_released = ui.input(|input| {
                        input.pointer.any_released()
                            || input.pointer.button_released(egui::PointerButton::Primary)
                            || (!input.pointer.is_decidedly_dragging() && input.pointer.primary_released())
                    });
                    if pointer_released
                        && let Some(payload) = drag_payload.as_ref()
                    {
                        let ids: Vec<i64> = if !payload.library_ids.is_empty() {
                            payload.library_ids.clone()
                        } else {
                            payload
                                .items
                                .iter()
                                .filter_map(|it| {
                                    it.id.rsplit('_').next().and_then(|s| s.parse().ok())
                                })
                                .collect()
                        };
                        if !ids.is_empty() {
                            action = Some(LibraryAction::Move(ids, folder.clone()));
                            kind.folders_mut(state).insert(folder.clone());
                            egui::DragAndDrop::clear_payload(ui.ctx());
                            ui.ctx().request_repaint();
                        }
                    }
                }

                let f_hover = ui
                    .ctx()
                    .animate_bool(folder_id.with("h"), response.hovered());
                let f_down = ui
                    .ctx()
                    .animate_bool(folder_id.with("d"), response.is_pointer_button_down_on());
                let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

                let f_fill = if is_drop_target_active {
                    theme.accent.linear_multiply(0.22)
                } else if f_down > 0.05 {
                    theme
                        .bg_card_hover
                        .lerp_to_gamma(theme.accent_soft, f_down * 0.45)
                } else if f_hover > 0.05 {
                    egui::Color32::TRANSPARENT.lerp_to_gamma(theme.bg_card_hover, f_hover)
                } else {
                    egui::Color32::TRANSPARENT
                };
                let f_stroke = if is_drop_target_active {
                    egui::Stroke::new(1.5, theme.accent)
                } else if f_down > 0.05 {
                    egui::Stroke::new(1.0, theme.accent.linear_multiply(0.7))
                } else if f_hover > 0.05 {
                    egui::Stroke::new(1.0, theme.border_subtle)
                } else {
                    egui::Stroke::NONE
                };
                let f_draw_rect = f_rect.shrink(f_down * 0.5);
                ui.painter().rect(
                    f_draw_rect,
                    theme.radius_md,
                    f_fill,
                    f_stroke,
                    egui::StrokeKind::Inside,
                );

                let f_text_y = f_draw_rect.center().y;
                let f_caret = if open { CARET_DOWN } else { CARET_RIGHT };
                let f_caret_g = ui.painter().layout_no_wrap(
                    f_caret.to_owned(),
                    egui::FontId::proportional(11.0),
                    theme.icon_muted,
                );
                ui.painter().galley(
                    egui::pos2(f_draw_rect.min.x + 8.0, f_text_y - f_caret_g.size().y / 2.0),
                    f_caret_g,
                    theme.icon_muted,
                );

                let folder_glyph = if is_drop_target_active || open {
                    FOLDER_OPEN
                } else {
                    FOLDER
                };
                let folder_color = if is_drop_target_active || open {
                    theme.accent
                } else {
                    theme.icon_muted
                };
                let f_icon_g = ui.painter().layout_no_wrap(
                    folder_glyph.to_owned(),
                    egui::FontId::proportional(12.5),
                    folder_color,
                );
                ui.painter().galley(
                    egui::pos2(f_draw_rect.min.x + 24.0, f_text_y - f_icon_g.size().y / 2.0),
                    f_icon_g,
                    folder_color,
                );

                let right_reserved_w = if is_drop_target_active {
                    let move_txt = language.text("common-move");
                    let move_g = ui.painter().layout_no_wrap(
                        move_txt.to_owned(),
                        egui::FontId::proportional(11.0),
                        theme.accent,
                    );
                    let w = move_g.size().x;
                    ui.painter().galley(
                        egui::pos2(
                            f_draw_rect.max.x - w - 8.0,
                            f_text_y - move_g.size().y / 2.0,
                        ),
                        move_g,
                        theme.accent,
                    );
                    w + 16.0
                } else if f_draw_rect.width() >= 90.0 {
                    let f_count_g = ui.painter().layout_no_wrap(
                        items.len().to_string(),
                        egui::FontId::proportional(11.0),
                        theme.text_muted,
                    );
                    let w = f_count_g.size().x;
                    ui.painter().galley(
                        egui::pos2(
                            f_draw_rect.max.x - w - 8.0,
                            f_text_y - f_count_g.size().y / 2.0,
                        ),
                        f_count_g,
                        theme.text_muted,
                    );
                    w + 16.0
                } else {
                    8.0
                };

                let max_name_w = (f_draw_rect.width() - 42.0 - right_reserved_w).max(10.0);
                let f_name_g = ui.painter().layout(
                    folder.clone(),
                    egui::FontId::proportional(12.5),
                    theme.text_primary,
                    max_name_w,
                );
                ui.painter().galley(
                    egui::pos2(f_draw_rect.min.x + 42.0, f_text_y - f_name_g.size().y / 2.0),
                    f_name_g,
                    theme.text_primary,
                );

                if response.clicked()
                    && !kind.folders_mut(state).remove(folder) {
                        kind.folders_mut(state).insert(folder.clone());
                    }
                response.context_menu(|ui| {
                    context_menu_header(ui);
                    if context_menu_item(ui, PLUS, language.text(kind.create_label()), &theme)
                        .clicked()
                    {
                        action = Some(LibraryAction::Create(Some(folder.clone())));
                        ui.close();
                    }
                    if kind == LibraryKind::Media
                        && context_menu_item(ui, GLOBE, language.text("library-add-web-video"), &theme)
                            .clicked()
                    {
                        state.web_media_modal.target_folder = folder.clone();
                        state.web_media_modal.open = true;
                        ui.close();
                    }
                    if context_menu_item(
                        ui,
                        LIST_PLUS,
                        language.text("common-add-playlist"),
                        &theme,
                    )
                    .clicked()
                    {
                        action = Some(LibraryAction::Playlist(
                            entries
                                .iter()
                                .filter(|item| &item.folder == folder)
                                .map(|item| item.id)
                                .collect(),
                        ));
                        ui.close();
                    }
                    if context_menu_item(
                        ui,
                        FOLDER_PLUS,
                        language.text("common-new-folder"),
                        &theme,
                    )
                    .clicked()
                    {
                        state.show_create_folder_modal = true;
                        ui.close();
                    }
                    context_menu_separator(ui);
                    if context_menu_danger_item(
                        ui,
                        TRASH,
                        language.text("common-delete-folder"),
                        &theme,
                    )
                    .clicked()
                    {
                        action = Some(LibraryAction::RequestDeleteFolder(folder.clone()));
                        ui.close();
                    }
                });
                if open {
                    ui.indent(("library_folder", folder), |ui| {
                        if items.is_empty() {
                            let (slot_rect, _) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), 26.0),
                                egui::Sense::hover(),
                            );
                            let pointer_pos = ui.ctx().pointer_latest_pos().or_else(|| ui.ctx().pointer_interact_pos());
                            let is_slot_hovered = pointer_pos.is_some_and(|pt| slot_rect.expand2(egui::vec2(0.0, 3.0)).contains(pt));
                            let is_slot_drop_active = is_slot_hovered && is_drag_library;
                            let slot_stroke = if is_slot_drop_active {
                                egui::Stroke::new(1.5, theme.accent)
                            } else {
                                egui::Stroke::new(1.0, theme.border_subtle)
                            };
                            let slot_bg = if is_slot_drop_active {
                                theme.accent.linear_multiply(0.18)
                            } else {
                                theme.bg_card.linear_multiply(0.4)
                            };
                            ui.painter().rect(slot_rect.shrink(1.0), theme.radius_sm, slot_bg, slot_stroke, egui::StrokeKind::Inside);
                            let slot_msg = if is_slot_drop_active {
                                language.text("common-move")
                            } else {
                                language.text("workspace-no-items-for-event")
                            };
                            let sg = ui.painter().layout_no_wrap(
                                format!("{FOLDER}  {slot_msg}"),
                                egui::FontId::proportional(11.0),
                                if is_slot_drop_active { theme.accent } else { theme.text_muted },
                            );
                            ui.painter().galley(slot_rect.center() - sg.size() * 0.5, sg, if is_slot_drop_active { theme.accent } else { theme.text_muted });

                            let pointer_released = ui.input(|input| {
                                input.pointer.any_released()
                                    || input.pointer.button_released(egui::PointerButton::Primary)
                                    || (!input.pointer.is_decidedly_dragging() && input.pointer.primary_released())
                            });
                            if is_slot_drop_active
                                && pointer_released
                                && let Some(payload) = drag_payload.as_ref()
                            {
                                let ids: Vec<i64> = if !payload.library_ids.is_empty() {
                                    payload.library_ids.clone()
                                } else {
                                    payload.items.iter().filter_map(|it| it.id.rsplit('_').next().and_then(|s| s.parse().ok())).collect()
                                };
                                if !ids.is_empty() {
                                    action = Some(LibraryAction::Move(ids, folder.clone()));
                                    kind.folders_mut(state).insert(folder.clone());
                                    egui::DragAndDrop::clear_payload(ui.ctx());
                                    ui.ctx().request_repaint();
                                }
                            }
                        }
                        for item in items {
                            row(ui, state, deck, kind, item, &visible, &folders, &mut action);
                        }
                    });
                }
                }
                for item in filtered.iter().filter(|item| item.folder.is_empty()) {
                    row(ui, state, deck, kind, item, &visible, &folders, &mut action);
                }
            } else {
                for item in &filtered {
                    row(ui, state, deck, kind, item, &visible, &folders, &mut action);
                }
            }
            if filtered.is_empty() {
                let searching = !query.is_empty();
                let (icon, hint_key) = if searching {
                    (MAGNIFYING_GLASS, "library-empty-search-hint")
                } else {
                    match kind {
                        LibraryKind::Song => (kind.icon(), "library-empty-songs-hint"),
                        LibraryKind::Text => (kind.icon(), "library-empty-texts-hint"),
                        LibraryKind::Media => (kind.icon(), "library-empty-media-hint"),
                        LibraryKind::Verse => (kind.icon(), "workspace-no-books-or-verses-found"),
                    }
                };
                let label = if searching {
                    language.text("library-clear-search").to_owned()
                } else {
                    format!("+  {}", language.text(kind.create_label()))
                };
                if crate::ui::components::empty_state::show(
                    ui, icon, "common-empty", hint_key, &label, &theme,
                ).clicked() {
                    if searching {
                        state.search_query.clear();
                    } else {
                        action = Some(LibraryAction::Create(None));
                    }
                }
            }
            let empty_h = (ui.available_height() - 8.0).max(40.0);
            let (root_rect, response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), empty_h),
                egui::Sense::click(),
            );
            let pointer_pos = ui.ctx().pointer_latest_pos().or_else(|| ui.ctx().pointer_interact_pos());
            let is_root_hover = pointer_pos.is_some_and(|pt| root_rect.intersect(ui.clip_rect()).contains(pt));
            let drag_payload =
                egui::DragAndDrop::payload::<crate::ui::components::drag_drop::DragPayload>(
                    ui.ctx(),
                );
            let is_drag_library = drag_payload.as_ref().is_some_and(|p| {
                matches!(
                    p.source,
                    crate::ui::components::drag_drop::DragSource::Library
                ) && (!p.library_ids.is_empty() || !p.items.is_empty())
            });
            let is_root_drop_active = is_root_hover && is_drag_library;

            if is_root_drop_active {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Copy);
                ui.painter().rect(
                    root_rect.shrink(4.0),
                    theme.radius_md,
                    theme.accent.linear_multiply(0.12),
                    egui::Stroke::new(1.5, theme.accent),
                    egui::StrokeKind::Inside,
                );
                let root_txt = language.text("common-root");
                let move_g = ui.painter().layout_no_wrap(
                    format!("{FOLDER}  {root_txt}"),
                    egui::FontId::proportional(11.5),
                    theme.accent,
                );
                ui.painter().galley(
                    root_rect.center() - move_g.size() * 0.5,
                    move_g,
                    theme.accent,
                );
                let pointer_released = ui.input(|input| {
                    input.pointer.any_released()
                        || input.pointer.button_released(egui::PointerButton::Primary)
                });
                if pointer_released
                    && let Some(payload) = drag_payload.as_ref()
                {
                    let ids: Vec<i64> = if !payload.library_ids.is_empty() {
                        payload.library_ids.clone()
                    } else {
                        payload
                            .items
                            .iter()
                            .filter_map(|it| {
                                it.id.rsplit('_').next().and_then(|s| s.parse().ok())
                            })
                            .collect()
                    };
                    if !ids.is_empty() {
                        action = Some(LibraryAction::Move(ids, String::new()));
                        egui::DragAndDrop::clear_payload(ui.ctx());
                        ui.ctx().request_repaint();
                    }
                }
            }

            if response.clicked() {
                kind.clear(state);
            }
            response.context_menu(|ui| {
                context_menu_header(ui);
                if context_menu_item(ui, PLUS, language.text(kind.create_label()), &theme).clicked()
                {
                    action = Some(LibraryAction::Create(None));
                    ui.close();
                }
                if kind == LibraryKind::Media
                    && context_menu_item(ui, GLOBE, language.text("library-add-web-video"), &theme)
                        .clicked()
                {
                    state.web_media_modal.target_folder.clear();
                    state.web_media_modal.open = true;
                    ui.close();
                }
                if context_menu_item(ui, FOLDER_PLUS, language.text("common-new-folder"), &theme)
                    .clicked()
                {
                    state.show_create_folder_modal = true;
                    ui.close();
                }
            });
        });

    if state.focused_container == Some(crate::core::layout::ContainerKind::Selection)
        && ui.memory(|m| m.focused().is_none())
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
    {
        if let Some(&first_id) = kind.selected(state).iter().next() {
            action = Some(LibraryAction::Present(first_id));
        } else if let Some(&first_id) = visible.first() {
            action = Some(LibraryAction::Present(first_id));
        }
    }

    if let Some(action) = action {
        actions::execute(action, kind, &entries, state, deck, presenting, db);
    }
}

fn selection_bar(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    kind: LibraryKind,
    folders: &[String],
    action: &mut Option<LibraryAction>,
) {
    let ids: Vec<_> = kind.selected(state).iter().copied().collect();
    if ids.len() < 2 {
        return;
    }
    let theme = state.theme.clone();
    let language = state.settings.language;
    ui.horizontal(|ui| {
        if crate::ui::components::interaction::selection_summary(ui, ids.len(), &theme).clicked() {
            kind.clear(state);
        }
        if ui
            .button(TRASH)
            .on_hover_text(language.text("common-delete"))
            .clicked()
        {
            *action = Some(LibraryAction::RequestDelete(ids.clone()));
        }
        let response = ui
            .button(FOLDER)
            .on_hover_text(language.text("common-move"));
        egui::Popup::menu(&response).show(|ui| {
            actions::move_menu(ui, state, folders, &ids, action);
        });
        if ui
            .button(LIST_PLUS)
            .on_hover_text(language.text("common-add-playlist"))
            .clicked()
        {
            *action = Some(LibraryAction::Playlist(ids.clone()));
        }
        if ui
            .button(PUSH_PIN)
            .on_hover_text(language.text("common-pin"))
            .clicked()
        {
            *action = Some(LibraryAction::Pin(ids));
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn row(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &SlideDeck,
    kind: LibraryKind,
    item: &LibraryEntry,
    visible: &[i64],
    folders: &[String],
    action: &mut Option<LibraryAction>,
) {
    let id = ui.make_persistent_id((kind.category(), item.id));
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }

    let theme = state.theme.clone();
    let language = state.settings.language;
    let is_selected = kind.selected(state).contains(&item.id);
    let is_current = kind == LibraryKind::Song
        && kind.selected(state).is_empty()
        && deck.selected_song_index() == item.index;
    let is_active = is_selected || is_current;

    let response = ui.interact(rect, id, egui::Sense::click_and_drag());
    let visual = crate::ui::components::interaction::row_emphasis(ui, &response, is_active, &theme);
    let _hover = visual.hover;
    let down = visual.press;
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

    let is_pressed = response.is_pointer_button_down_on();

    let draw_rect = rect.shrink(down * 0.5);

    if is_selected && kind.selected(state).len() > 1 {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                draw_rect.min + egui::vec2(2.5, 5.0),
                egui::vec2(2.5, draw_rect.height() - 10.0),
            ),
            1.5,
            theme.accent,
        );
    }

    let painter = ui
        .painter()
        .with_clip_rect(draw_rect.shrink2(egui::vec2(6.0, 0.0)));
    let text_y = draw_rect.center().y;
    let start_x = draw_rect.min.x
        + if is_selected && kind.selected(state).len() > 1 {
            12.0
        } else {
            8.0
        };

    let icon_galley = painter.layout_no_wrap(
        kind.icon().to_owned(),
        egui::FontId::proportional(12.0),
        if is_active || is_pressed {
            theme.accent
        } else {
            theme.icon_muted
        },
    );
    let icon_w = icon_galley.size().x;
    painter.galley(
        egui::pos2(start_x, text_y - icon_galley.size().y / 2.0),
        icon_galley,
        if is_active || is_pressed {
            theme.accent
        } else {
            theme.icon_muted
        },
    );

    let title_x = start_x + icon_w + 6.0;
    let title_color = if is_active || is_pressed {
        theme.accent
    } else {
        theme.text_primary
    };
    let author_color = if is_active || is_pressed {
        theme.text_secondary
    } else {
        theme.text_muted
    };
    let is_converting = state.is_item_converting(&item.content);
    let display_subtitle = if is_converting {
        format!("{} • {:.0}%", item.subtitle, state.media_conversion_percent() * 100.0)
    } else if !item.subtitle.is_empty() && !item.folder.is_empty() && !item.subtitle.eq_ignore_ascii_case(&item.folder) {
        format!("{} • {}", item.subtitle, item.folder)
    } else if !item.subtitle.is_empty() {
        item.subtitle.clone()
    } else {
        item.folder.clone()
    };
    let avail_w = (draw_rect.max.x - 8.0 - title_x).max(10.0);
    crate::ui::components::interaction::render_item_title_and_subtitle(
        ui,
        &painter,
        &item.title,
        &display_subtitle,
        egui::FontId::proportional(12.5),
        egui::FontId::proportional(11.0),
        title_color,
        author_color,
        egui::pos2(title_x, text_y),
        avail_w,
    );

    if is_converting {
        let pct = state.media_conversion_percent().clamp(0.0, 1.0);
        let bar_rect = egui::Rect::from_min_max(
            egui::pos2(draw_rect.min.x + 8.0, draw_rect.max.y - 2.5),
            egui::pos2(draw_rect.max.x - 8.0, draw_rect.max.y - 0.5),
        );
        ui.painter().rect_filled(bar_rect, 1.0, theme.bg_pill);
        let fill_w = (bar_rect.width() * pct).max(2.0);
        let fill_rect = egui::Rect::from_min_size(bar_rect.min, egui::vec2(fill_w, bar_rect.height()));
        ui.painter().rect_filled(fill_rect, 1.0, theme.accent);
    }

    let response = if is_converting {
        response.on_hover_text(state.media_conversion_info_tooltip())
    } else if item.subtitle.is_empty() {
        response.on_hover_text(&item.title)
    } else {
        response.on_hover_text(format!("{}\n{}", item.title, item.subtitle))
    };

    if response.drag_started() && !kind.selected(state).contains(&item.id) {
        model::select_range(kind, state, item.id, visible, false, false);
    }
    crate::ui::components::drag_drop::source(ui, &response, || {
        use crate::ui::components::drag_drop::{DragPayload, DragSource};
        let mut payload = DragPayload::new(
            DragSource::Library,
            &item.title,
            &item.subtitle,
            kind.icon(),
        );
        let selected = kind.selected(state);
        let matching: Vec<_> = model::entries(kind, state, deck)
            .into_iter()
            .filter(|entry| {
                if selected.contains(&item.id) {
                    selected.contains(&entry.id)
                } else {
                    entry.id == item.id
                }
            })
            .collect();
        payload.library_ids = matching.iter().map(|entry| entry.id).collect();
        payload.items = matching
            .into_iter()
            .map(|entry| crate::core::models::QuickItem {
                id: format!("{}_{}", kind.category(), entry.id),
                title: entry.title,
                subtitle: entry.subtitle,
                content: if kind == LibraryKind::Song {
                    deck.library().get(entry.index).map(|s| s.text.clone()).unwrap_or_default()
                } else {
                    entry.content
                },
                kind: kind.quick_kind(),
            })
            .collect();
        payload.count = payload.items.len();
        payload
    });

    if response.clicked() {
        let (shift, ctrl) = ui.input(|input| {
            (
                input.modifiers.shift,
                input.modifiers.ctrl || input.modifiers.command,
            )
        });
        model::select_range(kind, state, item.id, visible, shift, ctrl);
        if !shift && !ctrl {
            *action = Some(LibraryAction::Preview(item.id));
        }
    }
    if response.double_clicked() {
        let ids = if is_selected && kind.selected(state).len() > 1 {
            kind.selected(state).iter().copied().collect::<Vec<_>>()
        } else {
            vec![item.id]
        };
        *action = Some(LibraryAction::Playlist(ids));
    }
    response.context_menu(|ui| {
        context_menu_header(ui);
        let ids = if is_selected && kind.selected(state).len() > 1 {
            kind.selected(state).iter().copied().collect::<Vec<_>>()
        } else {
            vec![item.id]
        };
        if context_menu_item(ui, PLAY, language.text("common-project"), &theme).clicked() {
            *action = Some(LibraryAction::Present(item.id));
            ui.close();
        }
        if context_menu_item(ui, EYE, language.text("common-preview"), &theme).clicked() {
            *action = Some(LibraryAction::Preview(item.id));
            ui.close();
        }
        if matches!(kind, LibraryKind::Song | LibraryKind::Text)
            && context_menu_item(ui, PENCIL_SIMPLE, language.text("common-edit"), &theme).clicked()
        {
            *action = Some(LibraryAction::Edit(item.id));
            ui.close();
        }
        if kind == LibraryKind::Media
            && context_menu_item(ui, ARROWS_CLOCKWISE, language.text("library-convert-media"), &theme).clicked()
        {
            *action = Some(LibraryAction::Convert(item.id));
            ui.close();
        }
        if kind == LibraryKind::Media
            && context_menu_item(ui, INFO, language.text("library-media-info"), &theme).clicked()
        {
            *action = Some(LibraryAction::Info(item.id));
            ui.close();
        }
        if context_menu_item(ui, LIST_PLUS, language.text("common-add-playlist"), &theme).clicked()
        {
            *action = Some(LibraryAction::Playlist(ids.clone()));
            ui.close();
        }
        if context_menu_item(ui, PUSH_PIN, language.text("common-pin"), &theme).clicked() {
            *action = Some(LibraryAction::Pin(ids.clone()));
            ui.close();
        }
        context_menu_submenu(ui, FOLDER, language.text("common-move"), &theme, |ui| {
            actions::move_menu(ui, state, folders, &ids, action)
        });
        if matches!(kind, LibraryKind::Song | LibraryKind::Text)
            && context_menu_item(
                ui,
                EXPORT,
                language.text(if kind == LibraryKind::Song {
                    "workspace-export-song"
                } else {
                    "workspace-export-text"
                }),
                &theme,
            )
            .clicked()
        {
            *action = Some(LibraryAction::Export(ids.clone()));
            ui.close();
        }
        context_menu_separator(ui);
        if context_menu_danger_item(ui, TRASH, language.text("common-delete"), &theme).clicked() {
            *action = Some(LibraryAction::RequestDelete(ids));
            ui.close();
        }
    });
}
