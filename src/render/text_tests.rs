use super::*;

#[test]
fn bible_reference_is_visible_and_verse_is_centered() {
    use crate::core::bible::{BibleSettings, format_bible_slide};
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(1280.0, 720.0));
    let mut theme = PresentationTheme::default_themes().remove(0);
    theme.font_family = "Proportional".into();
    theme.shadow_enabled = false;
    // Bible alignment must override a theme's lower/right placement.
    theme.align_h = "right".into();
    theme.align_v = "bottom".into();
    let (_, content) = format_bible_slide(
        &BibleSettings {
            reference_position: crate::core::bible::BibleReferencePosition::Bottom,
            ..Default::default()
        },
        "Salmos",
        "Sl",
        "NTLH",
        39,
        4,
        4,
        &["Ó Senhor Deus, quanto tempo ainda vou viver? Mostra-me como é passageira a minha vida. Quando é que vou morrer?".into()],
    );
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        },
        |ui| render(ui.painter(), screen, &theme, &content),
    );
    let texts: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|s| {
            if let egui::Shape::Text(t) = &s.shape {
                Some((
                    s.clip_rect,
                    t.visual_bounding_rect(),
                    t.galley.job.text.clone(),
                ))
            } else {
                None
            }
        })
        .collect();
    output.textures_delta.clear();
    assert_eq!(texts.len(), 2, "{texts:?}");
    assert!(
        texts[1].0.contains_rect(texts[1].1),
        "Reference is clipped: {texts:?}"
    );
    assert!(
        (texts[0].1.center().x - screen.center().x).abs() < 16.0,
        "Verse is off center: {texts:?}"
    );
    assert!(
        (texts[0].1.center().y - screen.center().y).abs() < 16.0,
        "Verse is off center: {texts:?}"
    );
}

#[test]
fn light_solid_theme_uses_readable_text_color() {
    let mut theme = PresentationTheme::default();
    theme.bg_type = "gradient".into();
    theme.bg_top = Color32::from_rgb(245, 247, 250);
    theme.bg_bottom = Color32::from_rgb(235, 238, 243);
    theme.text_color = Color32::WHITE;
    let color = readable_text_color(&theme);
    assert!(contrast(color, theme.bg_top) >= 4.0);
    assert!(luminance(color) < 0.2);

    theme.text_color = Color32::from_rgb(24, 28, 36);
    assert_eq!(readable_text_color(&theme), theme.text_color);
}

#[test]
fn preview_and_secondary_screen_have_identical_normalized_typography() {
    let ctx = egui::Context::default();
    let mut normalized = Vec::new();
    for size in [egui::vec2(320.0, 180.0), egui::vec2(1920.0, 1080.0)] {
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
                ..Default::default()
            },
            |ui| {
                let mut theme = PresentationTheme::default();
                theme.font_family = "Proportional".into();
                theme.font_size = 260.0;
                theme.options.auto_fit = false;
                theme.shadow_enabled = false;
                render(
                    ui.painter(),
                    Rect::from_min_size(Pos2::ZERO, size),
                    &theme,
                    "Aa",
                );
            },
        );
        let bounds = output
            .shapes
            .iter()
            .find_map(|clipped| {
                if let egui::Shape::Text(text) = &clipped.shape {
                    Some(text.visual_bounding_rect())
                } else {
                    None
                }
            })
            .unwrap();
        normalized.push([
            bounds.left() / size.x,
            bounds.top() / size.y,
            bounds.width() / size.x,
            bounds.height() / size.y,
        ]);
        output.textures_delta.clear();
    }
    for (preview, live) in normalized[0].iter().zip(normalized[1]) {
        assert!((preview - live).abs() < 0.001, "{normalized:?}");
    }
}

#[test]
fn typography_wraps_and_has_no_120_point_ceiling() {
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let mut theme = PresentationTheme::default();
        theme.font_family = "Proportional".into();
        theme.font_size = 360.0;
        theme.options.auto_fit = false;
        let result = layout(ui.painter(), &theme, "Aa", egui::vec2(1200.0, 700.0));
        assert!((result.font_size - 360.0).abs() < 0.01);
        theme.font_size = 44.0;
        let text = "Uma passagem longa precisa manter a mesma quebra de linha na prévia e na tela secundária.";
        let result = layout(ui.painter(), &theme, text, egui::vec2(450.0, 400.0));
        assert!(result.galley.rows.len() > 1);
        assert!(result.size.x <= 450.1 && result.size.y <= 400.1);
        for size in [egui::vec2(320.0, 180.0), egui::vec2(1920.0, 1080.0)] {
            render(ui.painter(), Rect::from_min_size(Pos2::ZERO, size), &theme, text);
        }
    });
    output.textures_delta.clear();
}

#[test]
fn text_shadow_and_outline_render_smoothly_without_panicking() {
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let mut theme = PresentationTheme::default();
        theme.shadow_enabled = true;
        theme.shadow_blur = 24.0;
        theme.shadow_offset_x = 0.0;
        theme.shadow_offset_y = 3.0;
        theme.shadow_color = Color32::from_black_alpha(230);
        theme.outline_enabled = true;
        theme.outline_width = 4.0;
        theme.outline_color = Color32::BLACK;
        render(
            ui.painter(),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(1920.0, 1080.0)),
            &theme,
            "Se não for para te Adorar",
        );
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}

#[test]
fn test_show_song_effects_toggle() {
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let mut theme = PresentationTheme::default();
        theme.text_color = Color32::WHITE;
        theme.font_size = 40.0;
        theme.options.auto_fit = false;
        let text = "//t: 80\n//c: #FF0000\nHello [c: #00FF00]World[/c] and **holy**";

        // With song effects enabled:
        theme.options.show_song_effects = true;
        let layout_with = layout(ui.painter(), &theme, text, egui::vec2(1200.0, 700.0));

        // With song effects disabled:
        theme.options.show_song_effects = false;
        let layout_without = layout(ui.painter(), &theme, text, egui::vec2(1200.0, 700.0));

        // When song effects are disabled, requested size is theme font_size (40.0), not directive size (80.0)
        assert_eq!(layout_without.font_size, 40.0);
        assert_eq!(layout_with.font_size, 80.0);
        assert!(!layout_without.galley.rows.is_empty());
    });
    output.textures_delta.clear();
}

#[test]
fn test_render_blurred_produces_smooth_shapes() {
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let theme = PresentationTheme::default();
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 720.0));
        render_blurred(ui.painter(), rect, &theme, "Holy Holy Holy", 16.0);
    });
    output.textures_delta.clear();
    assert!(
        output.shapes.len() >= 30,
        "Gaussian spiral should generate rich multi-sample shapes"
    );
}

#[test]
fn test_bible_slides_always_wrap_and_never_affected_by_lyrics_wrap_toggle() {
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let mut theme = PresentationTheme::default();
        theme.options.wrap_text = false; // "Adaptar" selected for lyrics
        theme.options.auto_fit = false;

        let long_bible_text = "//bible\nPorque Deus amou o mundo de tal maneira que deu o seu Filho unigênito, para que todo aquele que nele crê não pereça, mas tenha a vida eterna.\n//ref: João 3:16\n//ref-pos: bottom";
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(600.0, 400.0));
        render(ui.painter(), rect, &theme, long_bible_text);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}
