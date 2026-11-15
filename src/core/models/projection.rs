use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LyricsProjectionSettings {
    pub show_title_slide: bool,
    pub show_title: bool,
    pub show_artist: bool,
    pub title_slide_uppercase: bool,
    pub blank_slide_at_start: bool,
    pub blank_slide_at_end: bool,
    pub hide_syntax_markers: bool,
    pub show_strophe_indicators: bool,
    pub auto_advance: bool,
    pub auto_advance_seconds: u32,
    pub loop_song: bool,
}

impl Default for LyricsProjectionSettings {
    fn default() -> Self {
        Self {
            show_title_slide: true,
            show_title: true,
            show_artist: true,
            title_slide_uppercase: false,
            blank_slide_at_start: false,
            blank_slide_at_end: true,
            hide_syntax_markers: true,
            show_strophe_indicators: false,
            auto_advance: false,
            auto_advance_seconds: 8,
            loop_song: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaProjectionSettings {
    pub auto_loop: bool,
    pub auto_play: bool,
    pub pause_on_blank: bool,
    pub stop_on_item_change: bool,
    pub video_fit: String,
    pub image_fit: String,
    pub image_crossfade: bool,
}

impl Default for MediaProjectionSettings {
    fn default() -> Self {
        Self {
            auto_loop: true,
            auto_play: true,
            pause_on_blank: true,
            stop_on_item_change: true,
            video_fit: "contain".into(),
            image_fit: "contain".into(),
            image_crossfade: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TextsProjectionSettings {
    pub show_title_slide: bool,
    pub show_header: bool,
    pub blank_slide_at_end: bool,
    pub preserve_exact_breaks: bool,
    pub show_page_number: bool,
}

impl Default for TextsProjectionSettings {
    fn default() -> Self {
        Self {
            show_title_slide: false,
            show_header: false,
            blank_slide_at_end: false,
            preserve_exact_breaks: true,
            show_page_number: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VersesProjectionSettings {
    pub reference_position: String,
    pub show_version: bool,
    pub show_verse_numbers: bool,
    pub use_abbreviation: bool,
    pub uppercase_reference: bool,
    pub auto_split_long_verses: bool,
    pub one_verse_per_slide: bool,
}

impl Default for VersesProjectionSettings {
    fn default() -> Self {
        Self {
            reference_position: "bottom".into(),
            show_version: true,
            show_verse_numbers: true,
            use_abbreviation: false,
            uppercase_reference: false,
            auto_split_long_verses: true,
            one_verse_per_slide: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesignsProjectionSettings {
    pub overlay_mode: bool,
    pub enable_animations: bool,
    pub auto_advance: bool,
    pub loop_presentation: bool,
    pub smooth_transitions: bool,
}

impl Default for DesignsProjectionSettings {
    fn default() -> Self {
        Self {
            overlay_mode: false,
            enable_animations: true,
            auto_advance: false,
            loop_presentation: true,
            smooth_transitions: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectionSettings {
    pub lyrics: LyricsProjectionSettings,
    pub media: MediaProjectionSettings,
    pub texts: TextsProjectionSettings,
    pub verses: VersesProjectionSettings,
    pub designs: DesignsProjectionSettings,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projection_settings_defaults() {
        let settings = ProjectionSettings::default();
        assert!(settings.lyrics.show_title_slide);
        assert!(settings.lyrics.show_title);
        assert!(settings.lyrics.show_artist);
        assert!(settings.lyrics.blank_slide_at_end);
        assert!(settings.media.auto_loop);
        assert_eq!(settings.media.video_fit, "contain");
        assert!(!settings.texts.show_title_slide);
        assert!(settings.verses.show_version);
        assert!(!settings.designs.overlay_mode);
    }
}
