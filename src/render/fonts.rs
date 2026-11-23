//! The same base/fallback fonts for native windows and exported text layers.
use eframe::egui;
use std::sync::{Arc, OnceLock};

/// Bundled Hind Madurai SemiBold (OFL), the church's Tamil font, so Tamil
/// script renders with any theme font.
pub(crate) const TAMIL_FALLBACK: &str = "hind-madurai-semibold";
pub(crate) const TAMIL_FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/HindMadurai-SemiBold.ttf");

/// All bundled Hind Madurai weights, offered in the font picker as "Hind Madurai".
pub(crate) const HIND_MADURAI_FACES: [&[u8]; 4] = [
    include_bytes!("../../assets/fonts/HindMadurai-Regular.ttf"),
    include_bytes!("../../assets/fonts/HindMadurai-Medium.ttf"),
    include_bytes!("../../assets/fonts/HindMadurai-SemiBold.ttf"),
    include_bytes!("../../assets/fonts/HindMadurai-Bold.ttf"),
];

/// Adds the Tamil fallback at the end of a named (theme) font family.
pub(crate) fn add_tamil_fallback(ctx: &egui::Context, family: &str) {
    ctx.add_font(egui::FontInsert::new(
        TAMIL_FALLBACK,
        egui::FontData::from_static(TAMIL_FONT_BYTES),
        vec![egui::InsertFontFamily {
            family: egui::FontFamily::Name(family.into()),
            priority: egui::FontPriority::Lowest,
        }],
    ));
}

pub(crate) fn definitions() -> egui::FontDefinitions {
    static FONTS: OnceLock<egui::FontDefinitions> = OnceLock::new();
    FONTS
        .get_or_init(|| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "custom-icons".into(),
                Arc::new(egui::FontData::from_static(include_bytes!(
                    "../../assets/fonts/custom-icons.ttf"
                ))),
            );
            fonts.font_data.insert(
                "phosphor".into(),
                Arc::new(egui::FontData::from_static(
                    egui_phosphor::Variant::Regular.font_bytes(),
                )),
            );
            fonts.font_data.insert(
                TAMIL_FALLBACK.into(),
                Arc::new(egui::FontData::from_static(TAMIL_FONT_BYTES)),
            );
            if let Some(keys) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                keys.extend(["custom-icons".into(), "phosphor".into(), TAMIL_FALLBACK.into()]);
            }
            if let Some(keys) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                keys.push(TAMIL_FALLBACK.into());
            }
            if let Some(bytes) = crate::core::models::fonts::color_emoji_font_bytes() {
                fonts.font_data.insert(
                    "color-emoji".into(),
                    Arc::new(egui::FontData::from_owned(bytes)),
                );
                for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                    if let Some(keys) = fonts.families.get_mut(&family) {
                        keys.push("color-emoji".into());
                    }
                }
            }
            fonts
        })
        .clone()
}
