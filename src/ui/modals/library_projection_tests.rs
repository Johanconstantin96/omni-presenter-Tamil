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
