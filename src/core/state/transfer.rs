use crate::core::models::PresentationTheme;

#[derive(Clone, Debug)]
pub struct PendingFontTheme {
    pub theme: PresentationTheme,
    pub missing_font: String,
    pub selected_replacement: String,
    pub search_query: String,
    pub font_file_status: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PlaylistExportModalState {
    pub playlist_name: String,
}
