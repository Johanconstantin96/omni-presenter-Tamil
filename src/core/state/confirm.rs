#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfirmAction {
    DeleteTheme {
        id: String,
        name: String,
    },
    DeleteSongs {
        ids: Vec<i64>,
    },
    DeleteMedia {
        ids: Vec<i64>,
    },
    DeleteTexts {
        ids: Vec<i64>,
    },
    DeleteSavedVerses {
        ids: Vec<i64>,
    },
    DeleteFolder {
        category: String,
        folder: String,
        delete_contents: bool,
    },
    ClearPlaylist,
    DeletePlaylistItems {
        indices: Vec<usize>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfirmDialogState {
    pub title: String,
    pub message: String,
    pub confirm_label: Option<String>,
    pub action: ConfirmAction,
    pub checkbox_label: Option<String>,
    pub checkbox_checked: bool,
}

impl ConfirmDialogState {
    pub fn new(title: impl Into<String>, message: impl Into<String>, action: ConfirmAction) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_label: None,
            action,
            checkbox_label: None,
            checkbox_checked: false,
        }
    }

    pub fn with_confirm_label(mut self, label: impl Into<String>) -> Self {
        self.confirm_label = Some(label.into());
        self
    }

    pub fn with_checkbox(mut self, label: impl Into<String>, checked: bool) -> Self {
        self.checkbox_label = Some(label.into());
        self.checkbox_checked = checked;
        self
    }
}

