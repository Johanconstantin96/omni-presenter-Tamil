use super::*;

fn frame(
    ctx: &egui::Context,
    settings: &AppSettings,
    db: &AppDb,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 1600.0),
            )),
            events,
            ..Default::default()
        },
        |ui| render(ui, settings, Some(db)),
    );
    output.textures_delta.clear();
    output
}

#[test]
fn transfer_panel_renders_real_database_controls_in_all_languages() {
    let db = AppDb::open_in_memory().unwrap();
    for language in crate::core::i18n::Language::ALL {
        let ctx = egui::Context::default();
        crate::ui::i18n::install(&ctx, language);
        let settings = AppSettings {
            language,
            ..Default::default()
        };
        let output = frame(&ctx, &settings, &db, vec![]);
        assert!(output.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.job.text == language.text("transfer-export"))));
        assert!(output.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.job.text == language.text("transfer-lyrics-choose"))));
        for detail in [
            "transfer-scope",
            "transfer-exclusions",
            "transfer-sensitive",
            "transfer-lyrics-formats",
        ] {
            assert!(!output.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.job.text == language.text(detail))), "Help should start collapsed: {detail}");
        }
        assert!(take_restore(&ctx).is_none());
    }
}

#[test]
fn action_cards_use_two_columns_on_wide_panels_and_stack_on_narrow_panels() {
    let db = AppDb::open_in_memory().unwrap();
    let settings = AppSettings::default();
    for width in [300.0, 760.0] {
        let ctx = egui::Context::default();
        crate::ui::i18n::install(&ctx, settings.language);
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 1200.0),
                )),
                ..Default::default()
            },
            |ui| render(ui, &settings, Some(&db)),
        );
        let position = |key| {
            output
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.job.text == settings.language.text(key) => {
                        Some(t.pos)
                    }
                    _ => None,
                })
                .unwrap()
        };
        let export = position("transfer-export-card");
        let import = position("transfer-import-card");
        let lyrics = position("transfer-lyrics");
        if width >= 480.0 {
            assert!((export.y - import.y).abs() < 1.0);
            assert!(import.x > export.x + 100.0);
        } else {
            assert!(import.y > export.y + 100.0);
        }
        assert!(lyrics.y > import.y + 100.0);
        for key in [
            "transfer-export",
            "transfer-import",
            "transfer-lyrics-choose",
        ] {
            let p = position(key);
            assert!(p.x >= 0.0 && p.x < width);
        }
        output.textures_delta.clear();
    }
}

#[test]
fn restoration_cannot_be_requested_until_replacement_is_explicitly_confirmed() {
    let db = AppDb::open_in_memory().unwrap();
    let settings = AppSettings::default();
    for confirmed in [false, true] {
        let ctx = egui::Context::default();
        let package = Package {
            format: "omni-presenter-workspace".into(),
            version: 1,
            application_version: "test".into(),
            tables: Vec::new(),
            assets: Default::default(),
            missing: Vec::new(),
        };
        put(
            &ctx,
            TransferUi {
                package: Some((PathBuf::from("backup.zip"), package)),
                confirmed,
                ..Default::default()
            },
        );
        let output = frame(&ctx, &settings, &db, vec![]);
        let position = output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t)
                    if t.galley.job.text == settings.language.text("transfer-restore") =>
                {
                    Some(t.pos + t.galley.rect.center().to_vec2())
                }
                _ => None,
            })
            .unwrap();
        let _ = frame(
            &ctx,
            &settings,
            &db,
            vec![
                egui::Event::PointerMoved(position),
                egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
                egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                },
            ],
        );
        assert_eq!(take_restore(&ctx).is_some(), confirmed);
    }
}

#[test]
fn lyrics_preview_starts_collapsed_with_clean_top_actions_and_toggles() {
    let db = AppDb::open_in_memory().unwrap();
    let settings = AppSettings::default();
    let ctx = egui::Context::default();
    crate::ui::i18n::install(&ctx, settings.language);

    let song1 = Song {
        id: Some(1),
        title: "HNC 001 - Doxologia".into(),
        artist: "Cantor".into(),
        folder: "".into(),
        tags: "".into(),
        text: "Glória a Deus".into(),
    };
    let song2 = Song {
        id: Some(2),
        title: "HNC 002 - Reverência".into(),
        artist: "Cantor".into(),
        folder: "".into(),
        tags: "".into(),
        text: "Santo, Santo".into(),
    };

    put(
        &ctx,
        TransferUi {
            songs: vec![(true, song1), (true, song2)],
            show_preview_list: false,
            ..Default::default()
        },
    );

    // Initial render: list is collapsed by default
    let output = frame(&ctx, &settings, &db, vec![]);
    // Primary add button should be visible right at the top
    assert!(output.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.job.text.contains(&settings.language.text("transfer-lyrics-add")))));
    // Review button should be visible with toggle label
    assert!(output.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.job.text.contains(&settings.language.text("transfer-lyrics-show-list")))));
    // Song items should not be visible when collapsed
    assert!(!output.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.job.text.contains("HNC 001 - Doxologia"))));

    // Now toggle list open
    let mut data = get(&ctx);
    data.show_preview_list = true;
    put(&ctx, data);

    let output_expanded = frame(&ctx, &settings, &db, vec![]);
    // Song title should now be visible
    assert!(output_expanded.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.job.text.contains("HNC 001 - Doxologia"))));
    // Toggle button should now show "Ocultar"
    assert!(output_expanded.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.job.text.contains(&settings.language.text("transfer-lyrics-hide-list")))));
    // Selection tools should be present
    assert!(output_expanded.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.job.text.contains(&settings.language.text("common-select-all")))));
}

