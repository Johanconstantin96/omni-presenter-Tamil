use super::*;

#[test]
fn test_setup_wizard_lifecycle_and_steps_rendering() {
    let mut state = ControlPanelState {
        show_setup_wizard: true,
        ..Default::default()
    };
    assert!(!state.settings.first_run_completed);

    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(1024.0, 768.0),
        )),
        ..Default::default()
    };

    let monitors = vec![
        MonitorInfo {
            winit_index: 0,
            name: "HDMI-1".to_string(),
            display_name: "Samsung 4K".to_string(),
            resolution: (3840, 2160),
            position: (0, 0),
            scale_factor: 1.0,
        },
        MonitorInfo {
            winit_index: 1,
            name: "DP-1".to_string(),
            display_name: "LG Projector".to_string(),
            resolution: (1920, 1080),
            position: (3840, 0),
            scale_factor: 1.0,
        },
    ];

    // Verify each step renders without panic
    for step in 0..=4 {
        state.setup_wizard.step = step;
        let mut output = ctx.run_ui(raw_input.clone(), |ui| {
            render_setup_wizard_modal(ui.ctx(), &mut state, &monitors, None);
        });
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty(), "Step {step} should produce shapes");
    }

    // Test finish_wizard
    state.setup_wizard.wrap_text = true;
    state.setup_wizard.auto_fit = true;
    finish_wizard(&mut state, None);

    assert!(state.settings.first_run_completed);
    assert!(!state.show_setup_wizard);
    assert_eq!(state.settings.service().aspect_ratio, 0.0);
    for theme in &state.themes {
        assert!(theme.options.wrap_text);
        assert!(theme.options.auto_fit);
    }
}

#[test]
fn test_setup_wizard_compact_viewport_rendering() {
    let mut state = ControlPanelState {
        show_setup_wizard: true,
        ..Default::default()
    };
    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(580.0, 480.0),
        )),
        ..Default::default()
    };

    for step in 0..=4 {
        state.setup_wizard.step = step;
        let mut output = ctx.run_ui(raw_input.clone(), |ui| {
            render_setup_wizard_modal(ui.ctx(), &mut state, &[], None);
        });
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty(), "Step {step} should render properly in compact viewport");
    }
}
