use crate::{
    core::i18n::Language,
    services::search::{SearchResult, SearchSnapshot, SearchTarget},
    ui::AppTheme,
};
use eframe::egui::{self, RichText};
use egui_phosphor::regular::*;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

fn category_icon(category: &str) -> &'static str {
    match category {
        "search-bible" | "library-verses" => BOOK_OPEN,
        "library-songs" => MUSIC_NOTES,
        "library-media" => VIDEO,
        "library-texts" => FILE_TEXT,
        "library-designs" => crate::ui::components::ICON_DESIGN,
        "search-themes" => PALETTE,
        "search-actions" => KEYBOARD,
        "search-settings" => GEAR,
        "plugins-title" => PUZZLE_PIECE,
        "search-chat" => CHAT_CIRCLE_TEXT,
        _ => MAGNIFYING_GLASS,
    }
}

#[derive(Default)]
pub struct GlobalSearch {
    pub open: bool,
    query: String,
    results: Vec<SearchResult>,
    selected: usize,
    focus: bool,
    requested_at: Option<Instant>,
    revision: u64,
    receiver: Option<mpsc::Receiver<(u64, Vec<SearchResult>)>>,
    pending: bool,
    cancellation: Option<Arc<AtomicBool>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_search_only_draws_the_input_and_waits_for_typing() {
        let ctx = egui::Context::default();
        let mut search = GlobalSearch::default();
        search.toggle();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1260.0, 760.0),
            )),
            ..Default::default()
        };
        for _ in 0..2 {
            let mut output = ctx.run_ui(input.clone(), |_| {
                search.show(&ctx, Language::English, &AppTheme::dark());
            });
            output.textures_delta.clear();
        }
        let mut output = ctx.run_ui(input.clone(), |_| {
            search.show(&ctx, Language::English, &AppTheme::dark());
        });
        output.textures_delta.clear();
        let texts: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if !text.galley.job.text.is_empty() => {
                    Some(text.galley.job.text.as_str())
                }
                _ => None,
            })
            .collect();
        assert!(texts.contains(&"Search"));
        assert!(!texts.contains(&"No items found"));
        assert!(!search.needs_search());
        let mut typed = input;
        typed.events.push(egui::Event::Text("settings".into()));
        let mut output = ctx.run_ui(typed, |_| {
            search.show(&ctx, Language::English, &AppTheme::dark());
        });
        output.textures_delta.clear();
        assert_eq!(search.query, "settings");
        assert!(search.pending);
    }

    #[test]
    fn rendering_results_shows_title_detail_and_category_icons() {
        let ctx = egui::Context::default();
        let mut search = GlobalSearch {
            open: true,
            query: "Salmo".into(),
            results: vec![SearchResult {
                title: "Salmos 119:11 · ARA".into(),
                detail: "Guardo no coração as tuas palavras".into(),
                category: "search-bible",
                target: SearchTarget::Bible {
                    version: "ARA".into(),
                    book: 18,
                    chapter: 119,
                    verse: 11,
                    end: 11,
                },
                score: 950,
            }],
            ..Default::default()
        };
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1260.0, 760.0),
            )),
            ..Default::default()
        };
        for _ in 0..2 {
            let mut output = ctx.run_ui(input.clone(), |_| {
                search.show(&ctx, Language::Portuguese, &AppTheme::dark());
            });
            output.textures_delta.clear();
        }
        let mut output = ctx.run_ui(input, |_| {
            search.show(&ctx, Language::Portuguese, &AppTheme::dark());
        });
        output.textures_delta.clear();
        let texts: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if !text.galley.job.text.is_empty() => {
                    Some(text.galley.job.text.as_str())
                }
                _ => None,
            })
            .collect();
        assert!(texts.contains(&"Salmos 119:11 · ARA"));
        assert!(texts.contains(&"Guardo no coração as tuas palavras"));
        assert!(texts.contains(&"Passagens bíblicas"));
    }
}

impl GlobalSearch {
    pub fn toggle(&mut self) {
        if let Some(flag) = &self.cancellation {
            flag.store(true, Ordering::Relaxed);
        }
        self.open = !self.open;
        if self.open {
            self.query.clear();
            self.results.clear();
            self.selected = 0;
            self.pending = false;
            self.receiver = None;
            self.revision += 1;
            self.focus = true;
            self.requested_at = None;
        }
    }

    pub fn needs_search(&self) -> bool {
        self.open
            && !self.query.trim().is_empty()
            && self
                .requested_at
                .is_some_and(|time| time.elapsed() >= Duration::from_millis(60))
    }

    pub fn search(&mut self, snapshot: SearchSnapshot, ctx: &egui::Context) {
        let query = self.query.clone();
        let revision = self.revision;
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        if let Some(flag) = &self.cancellation {
            flag.store(true, Ordering::Relaxed);
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        self.cancellation = Some(cancelled.clone());
        std::thread::spawn(move || {
            let local = snapshot.local_results(&query);
            if !local.is_empty() && !cancelled.load(Ordering::Relaxed) {
                let _ = sender.send((revision, local));
                ctx.request_repaint();
            }
            let results = snapshot.search_with_cancellation(&query, &cancelled);
            if !cancelled.load(Ordering::Relaxed) {
                let _ = sender.send((revision, results));
                ctx.request_repaint();
            }
        });
        self.receiver = Some(receiver);
        self.pending = true;
        self.requested_at = None;
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        language: Language,
        theme: &AppTheme,
    ) -> Option<SearchTarget> {
        if !self.open {
            return None;
        }
        if let Some(receiver) = &self.receiver {
            while let Ok((revision, results)) = receiver.try_recv() {
                if revision == self.revision {
                    let selected = self
                        .results
                        .get(self.selected)
                        .map(|result| result.target.clone());
                    self.selected = selected
                        .and_then(|target| {
                            results.iter().position(|result| result.target == target)
                        })
                        .unwrap_or(0);
                    self.results = results;
                    self.pending = false;
                }
            }
        }
        let mut target = None;
        let close =
            ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        let down =
            ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
        let up =
            ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
        let enter =
            ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
        if down {
            self.selected = (self.selected + 1).min(self.results.len().min(6).saturating_sub(1));
        }
        if up {
            self.selected = self.selected.saturating_sub(1);
        }
        if enter && !self.query.trim().is_empty() && !self.pending && self.requested_at.is_none() {
            target = self
                .results
                .get(self.selected)
                .map(|result| result.target.clone());
        }
        let id = egui::Id::unique("global_search");
        let screen = ctx.input(|input| input.viewport_rect());
        let modal = egui::Modal::new(id)
            .area(egui::Modal::default_area(id).anchor(
                egui::Align2::CENTER_TOP,
                egui::vec2(0.0, (screen.height() * 0.16).max(32.0)),
            ))
            .backdrop_color(egui::Color32::from_black_alpha(100))
            .frame(
                egui::Frame::new()
                    .fill(theme.bg_panel)
                    .stroke(egui::Stroke::new(1.0, theme.border_subtle))
                    .corner_radius(12.0)
                    .inner_margin(0)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 10],
                        blur: 32,
                        spread: 0,
                        color: egui::Color32::from_black_alpha(80),
                    }),
            )
            .show(ctx, |ui| {
                ui.set_width(580.0_f32.min(screen.width() - 48.0));
                let has_query = !self.query.trim().is_empty();

                // Search input row
                egui::Frame::new()
                    .inner_margin(egui::Margin { left: 14, right: 14, top: 11, bottom: 11 })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(10.0, 0.0);
                            let icon_color = if has_query { theme.accent } else { theme.icon_muted };
                            ui.label(RichText::new(MAGNIFYING_GLASS).size(16.0).color(icon_color));

                            let edit_w = (ui.available_width() - if has_query || self.pending { 28.0 } else { 38.0 }).max(40.0);
                            let response = ui.add_sized(
                                [edit_w, 24.0],
                                egui::TextEdit::singleline(&mut self.query)
                                    .font(egui::FontId::proportional(15.0))
                                    .hint_text(language.text("common-search"))
                                    .text_color(theme.text_primary)
                                    .frame(egui::Frame::NONE),
                            );
                            if self.focus {
                                response.request_focus();
                                self.focus = false;
                            }
                            if response.changed() {
                                if let Some(flag) = &self.cancellation {
                                    flag.store(true, Ordering::Relaxed);
                                }
                                self.revision += 1;
                                self.results.clear();
                                self.selected = 0;
                                self.pending = !self.query.trim().is_empty();
                                self.requested_at = if self.pending {
                                    Some(Instant::now())
                                } else {
                                    None
                                };
                                if self.pending {
                                    ctx.request_repaint_after(Duration::from_millis(65));
                                }
                            }

                            if self.pending {
                                ui.spinner();
                            } else if has_query {
                                if ui.add(
                                    egui::Button::new(
                                        RichText::new(X)
                                            .size(13.0)
                                            .color(theme.icon_muted),
                                    )
                                    .frame(false),
                                )
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                                {
                                    self.query.clear();
                                    self.results.clear();
                                    self.selected = 0;
                                    self.pending = false;
                                }
                            } else {
                                let (esc_rect, _) = ui.allocate_exact_size(egui::vec2(32.0, 18.0), egui::Sense::hover());
                                ui.painter().rect_filled(esc_rect, 4.0, theme.bg_card);
                                ui.painter().rect_stroke(
                                    esc_rect,
                                    4.0,
                                    egui::Stroke::new(1.0, theme.border_subtle),
                                    egui::StrokeKind::Inside,
                                );
                                ui.painter().text(
                                    esc_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "ESC",
                                    egui::FontId::proportional(9.5),
                                    theme.text_muted,
                                );
                            }
                        });
                    });

                if has_query || !self.results.is_empty() {
                    let (sep_rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 1.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().line_segment(
                        [sep_rect.left_top(), sep_rect.right_top()],
                        egui::Stroke::new(1.0, theme.border_subtle),
                    );

                    if self.pending {
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            ui.add_space(14.0);
                            ui.spinner();
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(language.text("search-loading"))
                                    .size(12.0)
                                    .color(theme.text_muted),
                            );
                        });
                        ui.add_space(12.0);
                    } else if self.results.is_empty() {
                        ui.add_space(16.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(language.text("search-empty"))
                                    .size(12.0)
                                    .color(theme.text_muted),
                            );
                        });
                        ui.add_space(16.0);
                    } else {
                        ui.add_space(6.0);
                        for (index, result) in self.results.iter().take(6).enumerate() {
                            let has_detail = !result.detail.is_empty();
                            let row_h = if has_detail { 48.0 } else { 38.0 };
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), row_h),
                                egui::Sense::click(),
                            );
                            let selected = index == self.selected;
                            let hovered = response.hovered();

                            let item_rect = rect.shrink2(egui::vec2(8.0, 2.0));
                            if selected || hovered {
                                ui.painter().rect_filled(
                                    item_rect,
                                    6.0,
                                    if selected {
                                        theme.accent_soft
                                    } else {
                                        theme.bg_card_hover
                                    },
                                );
                                if selected {
                                    let bar = egui::Rect::from_min_size(
                                        item_rect.left_top() + egui::vec2(0.0, 3.0),
                                        egui::vec2(3.0, item_rect.height() - 6.0),
                                    );
                                    ui.painter().rect_filled(bar, 1.5, theme.accent);
                                }
                            }

                            // Left category icon box
                            let icon_size = egui::vec2(26.0, 26.0);
                            let icon_rect = egui::Rect::from_center_size(
                                item_rect.left_center() + egui::vec2(18.0, 0.0),
                                icon_size,
                            );
                            ui.painter().rect_filled(
                                icon_rect,
                                6.0,
                                if selected {
                                    theme.accent.linear_multiply(0.12)
                                } else {
                                    theme.bg_card
                                },
                            );
                            ui.painter().text(
                                icon_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                category_icon(result.category),
                                egui::FontId::proportional(13.0),
                                if selected { theme.accent } else { theme.icon_muted },
                            );

                            // Category badge on the right
                            let cat_text = language.text(result.category);
                            let cat_galley = ui.painter().layout_no_wrap(
                                cat_text.into(),
                                egui::FontId::proportional(10.0),
                                if selected { theme.accent } else { theme.text_muted },
                            );
                            let cat_w = cat_galley.size().x + 12.0;
                            let cat_rect = egui::Rect::from_center_size(
                                egui::pos2(item_rect.right() - cat_w / 2.0 - 6.0, item_rect.center().y),
                                egui::vec2(cat_w, 20.0),
                            );
                            ui.painter().rect_filled(
                                cat_rect,
                                4.0,
                                if selected { theme.accent.linear_multiply(0.12) } else { theme.bg_card },
                            );
                            ui.painter().galley(
                                egui::pos2(
                                    cat_rect.left() + 6.0,
                                    cat_rect.center().y - cat_galley.size().y / 2.0,
                                ),
                                cat_galley,
                                if selected { theme.accent } else { theme.text_muted },
                            );

                            // Content area
                            let text_left = icon_rect.right() + 10.0;
                            let text_right = cat_rect.left() - 8.0;
                            let clip_rect = egui::Rect::from_min_max(
                                egui::pos2(text_left, item_rect.top()),
                                egui::pos2(text_right, item_rect.bottom()),
                            );

                            let title_galley = ui.painter().layout_no_wrap(
                                result.title.clone(),
                                egui::FontId::proportional(12.5),
                                theme.text_primary,
                            );

                            if has_detail {
                                let line1_y = item_rect.top() + 6.0;
                                let line2_y = line1_y + title_galley.size().y + 2.0;
                                ui.painter().with_clip_rect(clip_rect).galley(
                                    egui::pos2(text_left, line1_y),
                                    title_galley,
                                    theme.text_primary,
                                );
                                let detail_galley = ui.painter().layout_no_wrap(
                                    result.detail.clone(),
                                    egui::FontId::proportional(10.5),
                                    theme.text_muted,
                                );
                                ui.painter().with_clip_rect(clip_rect).galley(
                                    egui::pos2(text_left, line2_y),
                                    detail_galley,
                                    theme.text_muted,
                                );
                            } else {
                                ui.painter().with_clip_rect(clip_rect).galley(
                                    egui::pos2(text_left, item_rect.center().y - title_galley.size().y / 2.0),
                                    title_galley,
                                    theme.text_primary,
                                );
                            }

                            if response.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                target = Some(result.target.clone());
                            }
                        }
                        ui.add_space(4.0);
                    }

                    // Footer bar
                    if !self.results.is_empty() {
                        let (fsep_rect, _) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 1.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().line_segment(
                            [fsep_rect.left_top(), fsep_rect.right_top()],
                            egui::Stroke::new(1.0, theme.border_subtle),
                        );
                        egui::Frame::new()
                            .inner_margin(egui::Margin { left: 14, right: 14, top: 6, bottom: 6 })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(language.text("search-help"))
                                            .size(10.0)
                                            .color(theme.text_muted),
                                    );
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(format!("{}/{}", self.selected + 1, self.results.len().min(6)))
                                                .size(10.0)
                                                .color(theme.text_muted),
                                        );
                                    });
                                });
                            });
                    }
                }
            });
        if close || modal.should_close() || target.is_some() {
            self.open = false;
            if let Some(flag) = &self.cancellation {
                flag.store(true, Ordering::Relaxed);
            }
        }
        target
    }
}
