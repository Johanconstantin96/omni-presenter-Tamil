use crate::{
    core::{
        bible::BibleReferencePosition,
        slides::SlideDeck,
        state::ControlPanelState,
    },
    services::db::AppDb,
    ui::{
        components::dialog,
        modals::settings::components::{
            apply_settings_style, card, segmented_picker, setting_toggle, styled_button, ButtonTone,
        },
    },
};
use eframe::egui::{self, Frame, Margin, RichText, Stroke};
use egui_phosphor::regular::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProjectionTab {
    Songs,
    Media,
    Texts,
    Verses,
    Designs,
}

impl ProjectionTab {
    fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::Songs,
            1 => Self::Media,
            2 => Self::Texts,
            3 => Self::Verses,
            _ => Self::Designs,
        }
    }

    fn to_index(self) -> usize {
        match self {
            Self::Songs => 0,
            Self::Media => 1,
            Self::Texts => 2,
            Self::Verses => 3,
            Self::Designs => 4,
        }
    }
}

pub fn render_library_projection_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    db: Option<&AppDb>,
) {
    if !state.show_library_projection_modal {
        return;
    }

    let tokens = state.theme.clone();
    let mut active_tab = ProjectionTab::from_index(state.library_projection_modal_tab);
    let mut reset_defaults = false;

    let initial_proj = state.settings.projection.clone();
    let initial_bible = state.settings.bible_settings.clone();

    let close = dialog::show(
        ctx,
        "library_projection_modal",
        "library-projection-modal-title",
        "library-projection-modal-subtitle",
        580.0,
        &tokens,
        |ui| {
            apply_settings_style(ui, &tokens);
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 10.0);

            // ========================================================
            // 1. TOP: Segmented Tab Picker
            // ========================================================
            ui.vertical_centered(|ui| {
                ui.set_width(ui.available_width().min(540.0));
                ui.push_id("library_projection_tabs", |ui| {
                    if segmented_picker(
                        ui,
                        &mut active_tab,
                        &[
                            (ProjectionTab::Songs, "library-songs"),
                            (ProjectionTab::Media, "library-media"),
                            (ProjectionTab::Texts, "library-texts"),
                            (ProjectionTab::Verses, "library-verses"),
                            (ProjectionTab::Designs, "library-designs"),
                        ],
                        &tokens,
                    ) {
                        state.library_projection_modal_tab = active_tab.to_index();
                    }
                });
            });

            ui.add_space(2.0);

            // ========================================================
            // 2. SCROLLABLE BODY
            // ========================================================
            let narrow_footer = ui.available_width() < 500.0;
            let footer_height = if narrow_footer { 88.0 } else { 52.0 };
            let header_height = ui.min_size().y;
            let body_height = (ctx.content_rect().height() - 96.0 - header_height - footer_height).max(120.0);

            egui::ScrollArea::vertical()
                .id_salt(("library_projection_body", active_tab as u8))
                .max_height(body_height)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 10.0);
                    match active_tab {
                        ProjectionTab::Songs => {
                            render_songs_tab(ui, state, db, &tokens);
                        }
                        ProjectionTab::Media => {
                            render_media_tab(ui, state, &tokens);
                        }
                        ProjectionTab::Texts => {
                            render_texts_tab(ui, state, &tokens);
                        }
                        ProjectionTab::Verses => {
                            render_verses_tab(ui, state, &tokens);
                        }
                        ProjectionTab::Designs => {
                            render_designs_tab(ui, state, &tokens);
                        }
                    }
                });

            // ========================================================
            // 3. FOOTER
            // ========================================================
            ui.add_space(6.0);
            ui.separator();
            if narrow_footer {
                ui.add_space(2.0);
                ui.vertical_centered(|ui| {
                    if styled_button(
                        ui,
                        Some(CHECK),
                        crate::ui::i18n::text(ui, "common-done"),
                        ButtonTone::Primary,
                        &tokens,
                    )
                    .clicked()
                    {
                        state.show_library_projection_modal = false;
                    }

                    ui.add_space(4.0);

                    if styled_button(
                        ui,
                        Some(ARROW_COUNTER_CLOCKWISE),
                        crate::ui::i18n::text(ui, "library-projection-reset"),
                        ButtonTone::Ghost,
                        &tokens,
                    )
                    .clicked()
                    {
                        reset_defaults = true;
                    }
                });
            } else {
                ui.horizontal(|ui| {
                    if styled_button(
                        ui,
                        Some(ARROW_COUNTER_CLOCKWISE),
                        crate::ui::i18n::text(ui, "library-projection-reset"),
                        ButtonTone::Ghost,
                        &tokens,
                    )
                    .clicked()
                    {
                        reset_defaults = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if styled_button(
                            ui,
                            Some(CHECK),
                            crate::ui::i18n::text(ui, "common-done"),
                            ButtonTone::Primary,
                            &tokens,
                        )
                        .clicked()
                        {
                            state.show_library_projection_modal = false;
                        }
                    });
                });
            }
        },
    );

    if close {
        state.show_library_projection_modal = false;
    }

    if reset_defaults {
        match active_tab {
            ProjectionTab::Songs => {
                state.settings.projection.lyrics = Default::default();
                for theme in &mut state.themes {
                    theme.options.wrap_text = true;
                    theme.options.auto_fit = false;
                }
                state.setup_wizard.wrap_text = true;
                state.setup_wizard.auto_fit = false;
            }
            ProjectionTab::Media => state.settings.projection.media = Default::default(),
            ProjectionTab::Texts => state.settings.projection.texts = Default::default(),
            ProjectionTab::Verses => state.settings.bible_settings = Default::default(),
            ProjectionTab::Designs => state.settings.projection.designs = Default::default(),
        }
    }

    let changed = reset_defaults
        || state.settings.projection != initial_proj
        || state.settings.bible_settings != initial_bible;

    if changed {
        deck.lyrics_settings = state.settings.projection.lyrics.clone();
        deck.reload_song_slides();
        deck.reload_custom_content_slides(
            state.settings.projection.texts.show_title_slide,
            state.settings.projection.texts.blank_slide_at_end,
        );
        if let Some(db) = db {
            let _ = db.save_settings(&state.settings);
        }
    }
}

fn render_songs_tab(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
    tokens: &crate::ui::theme::AppTheme,
) {
    // Card 1: Slide de Abertura (Título e Artista)
    card(ui, tokens, |ui| {
        ui.label(
            RichText::new(format!("{TELEVISION_SIMPLE}  {}", crate::ui::i18n::text(ui, "library-projection-lyrics-title-slide")))
                .size(12.5)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(4.0);

        setting_toggle(
            ui,
            &mut state.settings.projection.lyrics.show_title_slide,
            "library-projection-lyrics-enable-title-slide",
            "library-projection-lyrics-enable-title-slide-desc",
            tokens,
        );

        if state.settings.projection.lyrics.show_title_slide {
            ui.add_space(4.0);
            ui.indent("lyrics_title_options", |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;

                setting_toggle(
                    ui,
                    &mut state.settings.projection.lyrics.show_title,
                    "library-projection-lyrics-show-title",
                    "library-projection-lyrics-show-title-desc",
                    tokens,
                );

                setting_toggle(
                    ui,
                    &mut state.settings.projection.lyrics.show_artist,
                    "library-projection-lyrics-show-artist",
                    "library-projection-lyrics-show-artist-desc",
                    tokens,
                );

                setting_toggle(
                    ui,
                    &mut state.settings.projection.lyrics.title_slide_uppercase,
                    "library-projection-lyrics-uppercase",
                    "library-projection-lyrics-uppercase-desc",
                    tokens,
                );

                ui.add_space(2.0);
                let hint = match (
                    state.settings.projection.lyrics.show_title,
                    state.settings.projection.lyrics.show_artist,
                ) {
                    (true, true) => crate::ui::i18n::text(ui, "library-projection-lyrics-hint-both"),
                    (true, false) => crate::ui::i18n::text(ui, "library-projection-lyrics-hint-title-only"),
                    (false, true) => crate::ui::i18n::text(ui, "library-projection-lyrics-hint-artist-only"),
                    (false, false) => crate::ui::i18n::text(ui, "library-projection-lyrics-hint-none"),
                };

                Frame::new()
                    .fill(tokens.bg_pill)
                    .stroke(Stroke::new(1.0, tokens.border_default))
                    .corner_radius(6.0)
                    .inner_margin(Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        ui.label(RichText::new(hint).size(11.0).color(tokens.text_secondary));
                    });
            });
        }
    });

    // Card 2: Quebra e Ajuste de Linhas
    card(ui, tokens, |ui| {
        ui.label(
            RichText::new(format!("{TEXT_T}  {}", crate::ui::i18n::text(ui, "library-projection-lyrics-typography-title")))
                .size(12.5)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(2.0);
        ui.label(
            RichText::new(crate::ui::i18n::text(ui, "library-projection-lyrics-typography-desc"))
                .size(11.5)
                .color(tokens.text_secondary),
        );
        ui.add_space(6.0);

        // Opções visuais interativas mostrando diretamente como o texto se comporta
        render_typography_visual_options(ui, state, db, tokens);
    });

    // Card 3: Layout dos Slides e Formatação
    card(ui, tokens, |ui| {
        ui.label(
            RichText::new(format!("{ROWS}  {}", crate::ui::i18n::text(ui, "library-projection-lyrics-layout-title")))
                .size(12.5)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(4.0);

        setting_toggle(
            ui,
            &mut state.settings.projection.lyrics.blank_slide_at_end,
            "library-projection-lyrics-blank-end",
            "library-projection-lyrics-blank-end-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.projection.lyrics.blank_slide_at_start,
            "library-projection-lyrics-blank-start",
            "library-projection-lyrics-blank-start-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.projection.lyrics.hide_syntax_markers,
            "library-projection-lyrics-hide-syntax",
            "library-projection-lyrics-hide-syntax-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.projection.lyrics.show_strophe_indicators,
            "library-projection-lyrics-strophe-indicator",
            "library-projection-lyrics-strophe-indicator-desc",
            tokens,
        );
    });

    // Card 3: Automação e Transição
    card(ui, tokens, |ui| {
        ui.label(
            RichText::new(format!("{ARROWS_CLOCKWISE}  {}", crate::ui::i18n::text(ui, "library-projection-lyrics-auto-title")))
                .size(12.5)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(4.0);

        setting_toggle(
            ui,
            &mut state.settings.projection.lyrics.auto_advance,
            "library-projection-lyrics-auto-advance",
            "library-projection-lyrics-auto-advance-desc",
            tokens,
        );

        if state.settings.projection.lyrics.auto_advance {
            ui.indent("lyrics_auto_advance_sec", |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "library-projection-lyrics-seconds-per-slide"))
                            .size(11.0)
                            .color(tokens.text_secondary),
                    );
                    ui.add(
                        egui::Slider::new(&mut state.settings.projection.lyrics.auto_advance_seconds, 3..=30)
                            .suffix("s"),
                    );
                });
            });
        }

        setting_toggle(
            ui,
            &mut state.settings.projection.lyrics.loop_song,
            "library-projection-lyrics-loop-song",
            "library-projection-lyrics-loop-song-desc",
            tokens,
        );
    });
}

fn render_media_tab(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    tokens: &crate::ui::theme::AppTheme,
) {
    // Card 1: Vídeos e Áudios
    card(ui, tokens, |ui| {
        ui.label(
            RichText::new(format!("{FILM_STRIP}  {}", crate::ui::i18n::text(ui, "library-projection-media-title")))
                .size(12.5)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(4.0);

        setting_toggle(
            ui,
            &mut state.settings.projection.media.auto_loop,
            "library-projection-media-loop",
            "library-projection-media-loop-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.projection.media.auto_play,
            "library-projection-media-autoplay",
            "library-projection-media-autoplay-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.projection.media.pause_on_blank,
            "library-projection-media-pause-blank",
            "library-projection-media-pause-blank-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.projection.media.stop_on_item_change,
            "library-projection-media-stop-change",
            "library-projection-media-stop-change-desc",
            tokens,
        );
    });

    // Card 2: Ajuste e Enquadramento
    card(ui, tokens, |ui| {
        ui.label(
            RichText::new(format!("{DESKTOP}  {}", crate::ui::i18n::text(ui, "library-projection-media-scale-title")))
                .size(12.5)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(4.0);

        ui.label(
            RichText::new(crate::ui::i18n::text(ui, "library-projection-media-video-fit-label"))
                .size(11.0)
                .color(tokens.text_secondary),
        );
        ui.add_space(2.0);

        #[derive(Clone, Copy, PartialEq, Eq)]
        enum FitOption {
            Contain,
            Cover,
            Stretch,
        }

        let current_video_fit = match state.settings.projection.media.video_fit.as_str() {
            "cover" => FitOption::Cover,
            "stretch" => FitOption::Stretch,
            _ => FitOption::Contain,
        };

        let mut sel_fit = current_video_fit;
        ui.push_id("media_video_fit_picker", |ui| {
            if segmented_picker(
                ui,
                &mut sel_fit,
                &[
                    (FitOption::Contain, "library-projection-media-fit-contain"),
                    (FitOption::Cover, "library-projection-media-fit-cover"),
                    (FitOption::Stretch, "library-projection-media-fit-stretch"),
                ],
                tokens,
            ) {
                state.settings.projection.media.video_fit = match sel_fit {
                    FitOption::Cover => "cover".into(),
                    FitOption::Stretch => "stretch".into(),
                    FitOption::Contain => "contain".into(),
                };
                if let Some(media) = &mut state.active_player_media {
                    media.video_fit = state.settings.projection.media.video_fit.clone();
                }
            }
        });

        ui.add_space(8.0);
        setting_toggle(
            ui,
            &mut state.settings.projection.media.image_crossfade,
            "library-projection-media-crossfade",
            "library-projection-media-crossfade-desc",
            tokens,
        );
    });
}

fn render_texts_tab(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    tokens: &crate::ui::theme::AppTheme,
) {
    // Card 1: Estrutura dos Slides
    card(ui, tokens, |ui| {
        ui.label(
            RichText::new(format!("{FILE_TEXT}  {}", crate::ui::i18n::text(ui, "library-projection-texts-title")))
                .size(12.5)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(4.0);

        setting_toggle(
            ui,
            &mut state.settings.projection.texts.show_title_slide,
            "library-projection-texts-title-slide",
            "library-projection-texts-title-slide-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.projection.texts.show_header,
            "library-projection-texts-header",
            "library-projection-texts-header-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.projection.texts.blank_slide_at_end,
            "library-projection-texts-blank-end",
            "library-projection-texts-blank-end-desc",
            tokens,
        );
    });

    // Card 2: Quebras e Paginação
    card(ui, tokens, |ui| {
        ui.label(
            RichText::new(format!("{TEXT_T}  {}", crate::ui::i18n::text(ui, "library-projection-texts-format-title")))
                .size(12.5)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(4.0);

        setting_toggle(
            ui,
            &mut state.settings.projection.texts.preserve_exact_breaks,
            "library-projection-texts-preserve-breaks",
            "library-projection-texts-preserve-breaks-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.projection.texts.show_page_number,
            "library-projection-texts-page-num",
            "library-projection-texts-page-num-desc",
            tokens,
        );
    });
}

fn render_verses_tab(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    tokens: &crate::ui::theme::AppTheme,
) {
    // Card 1: Referência Bíblica
    card(ui, tokens, |ui| {
        ui.label(
            RichText::new(format!("{BOOK_OPEN}  {}", crate::ui::i18n::text(ui, "library-projection-verses-ref-section")))
                .size(12.5)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(4.0);

        ui.label(
            RichText::new(crate::ui::i18n::text(ui, "library-projection-verses-ref-pos"))
                .size(11.0)
                .color(tokens.text_secondary),
        );
        ui.add_space(2.0);

        ui.push_id("verses_ref_pos_seg", |ui| {
            segmented_picker(
                ui,
                &mut state.settings.bible_settings.reference_position,
                &[
                    (BibleReferencePosition::BelowVerse, "workspace-bible-ref-below-verse"),
                    (BibleReferencePosition::Top, "workspace-bible-ref-top"),
                    (BibleReferencePosition::EndOfVerse, "workspace-bible-ref-end-of-verse"),
                    (BibleReferencePosition::Hidden, "workspace-bible-ref-hidden"),
                ],
                tokens,
            );
        });

        ui.add_space(6.0);

        setting_toggle(
            ui,
            &mut state.settings.bible_settings.show_version,
            "library-projection-verses-version",
            "library-projection-verses-version-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.bible_settings.use_abbreviation,
            "library-projection-verses-abbrev",
            "library-projection-verses-abbrev-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.bible_settings.uppercase_reference,
            "library-projection-verses-upper",
            "library-projection-verses-upper-desc",
            tokens,
        );
    });

    // Card 2: Formatação dos Versículos
    card(ui, tokens, |ui| {
        ui.label(
            RichText::new(format!("{ROWS}  {}", crate::ui::i18n::text(ui, "library-projection-verses-text-section")))
                .size(12.5)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(4.0);

        setting_toggle(
            ui,
            &mut state.settings.bible_settings.show_verse_numbers,
            "library-projection-verses-numbers",
            "library-projection-verses-numbers-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.bible_settings.quotes_around_verse,
            "library-projection-verses-quotes",
            "library-projection-verses-quotes-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.bible_settings.auto_split_long_verses,
            "library-projection-verses-split",
            "library-projection-verses-split-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.bible_settings.one_verse_per_slide,
            "library-projection-verses-single",
            "library-projection-verses-single-desc",
            tokens,
        );
    });
}

fn render_designs_tab(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    tokens: &crate::ui::theme::AppTheme,
) {
    // Card 1: Exibição e Camadas
    card(ui, tokens, |ui| {
        ui.label(
            RichText::new(format!("{PALETTE}  {}", crate::ui::i18n::text(ui, "library-projection-designs-title")))
                .size(12.5)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(4.0);

        setting_toggle(
            ui,
            &mut state.settings.projection.designs.overlay_mode,
            "library-projection-designs-overlay",
            "library-projection-designs-overlay-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.projection.designs.enable_animations,
            "library-projection-designs-anim",
            "library-projection-designs-anim-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.projection.designs.smooth_transitions,
            "library-projection-designs-smooth",
            "library-projection-designs-smooth-desc",
            tokens,
        );
    });

    // Card 2: Navegação e Temporização
    card(ui, tokens, |ui| {
        ui.label(
            RichText::new(format!("{ARROWS_CLOCKWISE}  {}", crate::ui::i18n::text(ui, "library-projection-designs-nav-section")))
                .size(12.5)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(4.0);

        setting_toggle(
            ui,
            &mut state.settings.projection.designs.auto_advance,
            "library-projection-designs-auto",
            "library-projection-designs-auto-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.settings.projection.designs.loop_presentation,
            "library-projection-designs-loop",
            "library-projection-designs-loop-desc",
            tokens,
        );
    });
}

enum LineModePreviewKind {
    Wrap,
    Fit,
}

fn render_line_choice_card(
    ui: &mut egui::Ui,
    id_str: &str,
    is_selected: bool,
    title: &str,
    description: &str,
    preview_kind: LineModePreviewKind,
    tokens: &crate::ui::theme::AppTheme,
) -> bool {
    let mut clicked = false;
    let card_id = ui.make_persistent_id(id_str);

    let (bg, stroke) = if is_selected {
        (
            tokens.accent_soft,
            Stroke::new(1.5, tokens.accent),
        )
    } else {
        (tokens.bg_pill, Stroke::new(1.0, tokens.border_default))
    };

    let frame_response = Frame::new()
        .fill(bg)
        .stroke(stroke)
        .corner_radius(8.0)
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 10.0;

            // 1. Mini visual projector screen preview in exact 16:9 proportion
            let box_width = ui.available_width();
            let box_height = (box_width * 9.0 / 16.0).round().max(50.0);
            ui.set_min_height(box_height + 84.0);

            let (box_rect, _) = ui.allocate_exact_size(
                egui::vec2(box_width, box_height),
                egui::Sense::hover(),
            );
            let painter = ui.painter();
            painter.rect_filled(box_rect, 6.0, egui::Color32::from_rgb(16, 20, 26));
            let box_border = if is_selected {
                tokens.accent.linear_multiply(0.5)
            } else {
                egui::Color32::from_rgb(42, 50, 62)
            };
            painter.rect_stroke(box_rect, 6.0, Stroke::new(1.0, box_border), egui::StrokeKind::Inside);

            let center = box_rect.center();
            match preview_kind {
                LineModePreviewKind::Wrap => {
                    painter.text(
                        egui::pos2(center.x, center.y - 12.0),
                        egui::Align2::CENTER_CENTER,
                        "Porque Ele vive, posso crer",
                        egui::FontId::proportional(12.5),
                        egui::Color32::WHITE,
                    );
                    painter.text(
                        egui::pos2(center.x, center.y + 12.0),
                        egui::Align2::CENTER_CENTER,
                        "no amanhã!",
                        egui::FontId::proportional(12.5),
                        egui::Color32::WHITE,
                    );
                }
                LineModePreviewKind::Fit => {
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        "Porque Ele vive, posso crer no amanhã!",
                        egui::FontId::proportional(8.5),
                        egui::Color32::WHITE,
                    );
                }
            }

            // 2. Title with radio icon
            ui.horizontal(|ui| {
                let icon = if is_selected { CHECK_CIRCLE } else { CIRCLE };
                let icon_color = if is_selected {
                    tokens.accent
                } else {
                    tokens.text_secondary
                };
                ui.label(RichText::new(icon).size(15.0).color(icon_color));

                ui.label(
                    RichText::new(title)
                        .size(12.5)
                        .strong()
                        .color(if is_selected {
                            tokens.accent
                        } else {
                            tokens.text_primary
                        }),
                );
            });

            // 3. Description
            ui.add(
                egui::Label::new(
                    RichText::new(description)
                        .size(10.5)
                        .color(tokens.text_secondary),
                )
                .wrap(),
            );
        });

    let rect = frame_response.response.rect;
    let interact = ui.interact(rect, card_id, egui::Sense::click());
    if interact.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        if !is_selected {
            ui.painter().rect_stroke(
                rect,
                8.0,
                Stroke::new(1.0, tokens.border_active),
                egui::StrokeKind::Inside,
            );
        }
    }
    if interact.clicked() {
        clicked = true;
    }

    clicked
}

fn render_typography_visual_options(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    _db: Option<&AppDb>,
    tokens: &crate::ui::theme::AppTheme,
) {
    let current_wrap = state.settings.projection.lyrics.wrap_text;
    let current_fit = state.settings.projection.lyrics.auto_fit;

    let is_adaptar_selected = !current_wrap && current_fit;
    let is_quebrar_selected = !is_adaptar_selected;

    let mut new_selection = None;

    let prev_spacing_x = ui.spacing().item_spacing.x;
    ui.spacing_mut().item_spacing.x = 14.0;

    ui.columns(2, |columns| {
        // Coluna 1: Quebrar
        let wrap_title = crate::ui::i18n::text(&columns[0], "library-projection-lyrics-mode-wrap");
        let wrap_desc = crate::ui::i18n::text(&columns[0], "library-projection-lyrics-mode-wrap-desc");
        if render_line_choice_card(
            &mut columns[0],
            "line_mode_wrap",
            is_quebrar_selected,
            wrap_title,
            wrap_desc,
            LineModePreviewKind::Wrap,
            tokens,
        ) {
            new_selection = Some((true, false));
        }

        // Coluna 2: Adaptar
        let fit_title = crate::ui::i18n::text(&columns[1], "library-projection-lyrics-mode-fit");
        let fit_desc = crate::ui::i18n::text(&columns[1], "library-projection-lyrics-mode-fit-desc");
        if render_line_choice_card(
            &mut columns[1],
            "line_mode_fit",
            is_adaptar_selected,
            fit_title,
            fit_desc,
            LineModePreviewKind::Fit,
            tokens,
        ) {
            new_selection = Some((false, true));
        }
    });

    ui.spacing_mut().item_spacing.x = prev_spacing_x;

    if let Some((wrap, fit)) = new_selection {
        state.settings.projection.lyrics.wrap_text = wrap;
        state.settings.projection.lyrics.auto_fit = fit;
        state.setup_wizard.wrap_text = wrap;
        state.setup_wizard.auto_fit = fit;
        for theme in &mut state.themes {
            theme.options.wrap_text = wrap;
            theme.options.auto_fit = fit;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_projection_modal_lifecycle_and_tab_navigation() {
        let mut state = ControlPanelState::default();
        let mut deck = SlideDeck::default();
        let ctx = egui::Context::default();

        state.show_library_projection_modal = true;
        state.library_projection_modal_tab = 0; // Songs

        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        };

        // Render tab 0 (Songs)
        let mut out = ctx.run_ui(raw.clone(), |ui| {
            render_library_projection_modal(ui.ctx(), &mut state, &mut deck, None);
        });
        out.textures_delta.clear();
        assert!(state.show_library_projection_modal);

        // Switch to tab 1 (Media)
        state.library_projection_modal_tab = 1;
        let mut out = ctx.run_ui(raw.clone(), |ui| {
            render_library_projection_modal(ui.ctx(), &mut state, &mut deck, None);
        });
        out.textures_delta.clear();

        // Switch to tab 2 (Texts)
        state.library_projection_modal_tab = 2;
        let mut out = ctx.run_ui(raw.clone(), |ui| {
            render_library_projection_modal(ui.ctx(), &mut state, &mut deck, None);
        });
        out.textures_delta.clear();

        // Switch to tab 3 (Verses)
        state.library_projection_modal_tab = 3;
        let mut out = ctx.run_ui(raw.clone(), |ui| {
            render_library_projection_modal(ui.ctx(), &mut state, &mut deck, None);
        });
        out.textures_delta.clear();

        // Switch to tab 4 (Designs)
        state.library_projection_modal_tab = 4;
        let mut out = ctx.run_ui(raw, |ui| {
            render_library_projection_modal(ui.ctx(), &mut state, &mut deck, None);
        });
        out.textures_delta.clear();
        assert!(state.show_library_projection_modal);
    }

    #[test]
    fn test_typography_line_preview_and_toggle_options() {
        let mut state = ControlPanelState::default();
        let mut deck = SlideDeck::default();
        let ctx = egui::Context::default();

        state.show_library_projection_modal = true;
        state.library_projection_modal_tab = 0; // Songs

        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        };

        // Render tab with default wrap_text = true, auto_fit = true
        let mut out = ctx.run_ui(raw.clone(), |ui| {
            render_library_projection_modal(ui.ctx(), &mut state, &mut deck, None);
        });
        out.textures_delta.clear();

        // Mutate theme options and verify rendering with other combinations
        for theme in &mut state.themes {
            theme.options.wrap_text = false;
            theme.options.auto_fit = true;
        }

        let mut out = ctx.run_ui(raw.clone(), |ui| {
            render_library_projection_modal(ui.ctx(), &mut state, &mut deck, None);
        });
        out.textures_delta.clear();

        for theme in &mut state.themes {
            theme.options.wrap_text = true;
            theme.options.auto_fit = false;
        }

        let mut out = ctx.run_ui(raw.clone(), |ui| {
            render_library_projection_modal(ui.ctx(), &mut state, &mut deck, None);
        });
        out.textures_delta.clear();

        for theme in &mut state.themes {
            theme.options.wrap_text = false;
            theme.options.auto_fit = false;
        }

        let mut out = ctx.run_ui(raw, |ui| {
            render_library_projection_modal(ui.ctx(), &mut state, &mut deck, None);
        });
        out.textures_delta.clear();
        assert!(state.show_library_projection_modal);
    }
}
