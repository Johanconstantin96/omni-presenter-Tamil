//! Text manipulation and selection actions for the song lyric editor.

use crate::core::state::ControlPanelState;

/// Safely maps a unicode char index (0-based count of chars) to a byte offset.
pub fn char_to_byte_idx(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(text.len())
}

/// Safely maps a byte offset to a unicode char index.
#[allow(dead_code)]
pub fn byte_to_char_idx(text: &str, byte_idx: usize) -> usize {
    let clamped = byte_idx.min(text.len());
    text[..clamped].chars().count()
}

/// Helper to wrap currently selected text in prefix and suffix (e.g. `**` for bold).
pub fn wrap_selection(state: &mut ControlPanelState, prefix: &str, suffix: &str) -> bool {
    let text = &state.song_editor.text;
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );

    if start_char == end_char {
        return false;
    }

    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);

    if start_byte >= end_byte || end_byte > text.len() {
        return false;
    }

    let selected = &text[start_byte..end_byte];

    // Toggle off if already wrapped
    if selected.starts_with(prefix)
        && selected.ends_with(suffix)
        && selected.len() >= prefix.len() + suffix.len()
    {
        let unwrapped = &selected[prefix.len()..selected.len() - suffix.len()];
        let new_text = format!("{}{}{}", &text[..start_byte], unwrapped, &text[end_byte..]);
        let unwrap_char_len = unwrapped.chars().count();
        state.song_editor.text = new_text;
        state.song_editor.selection = (start_char, start_char + unwrap_char_len);
        return true;
    }

    let wrapped = format!("{prefix}{selected}{suffix}");
    let wrapped_char_len = wrapped.chars().count();
    let new_text = format!("{}{}{}", &text[..start_byte], wrapped, &text[end_byte..]);
    state.song_editor.text = new_text;
    state.song_editor.selection = (start_char, start_char + wrapped_char_len);
    true
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CaseMode {
    Upper,
    Lower,
    Title,
}

/// Transforms case of the currently selected text.
pub fn transform_selection_case(state: &mut ControlPanelState, mode: CaseMode) -> bool {
    let text = &state.song_editor.text;
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );

    if start_char == end_char {
        return false;
    }

    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);

    if start_byte >= end_byte || end_byte > text.len() {
        return false;
    }

    let selected = &text[start_byte..end_byte];
    let transformed = match mode {
        CaseMode::Upper => selected.to_uppercase(),
        CaseMode::Lower => selected.to_lowercase(),
        CaseMode::Title => {
            let mut result = String::with_capacity(selected.len());
            let mut capitalize_next = true;
            for c in selected.chars() {
                if c.is_alphanumeric() {
                    if capitalize_next {
                        result.extend(c.to_uppercase());
                        capitalize_next = false;
                    } else {
                        result.extend(c.to_lowercase());
                    }
                } else {
                    result.push(c);
                    capitalize_next = true;
                }
            }
            result
        }
    };

    let trans_char_len = transformed.chars().count();
    let new_text = format!("{}{}{}", &text[..start_byte], transformed, &text[end_byte..]);
    state.song_editor.text = new_text;
    state.song_editor.selection = (start_char, start_char + trans_char_len);
    true
}

/// Prefixes lines within the current selection.
pub fn prefix_selected_lines(state: &mut ControlPanelState, prefix: &str) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    let text = &state.song_editor.text;
    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);

    let mut new_text = String::with_capacity(text.len() + 32);
    let mut current_byte = 0;
    for line in text.split('\n') {
        let line_len = line.len();
        let line_end_byte = current_byte + line_len;
        let intersects = if start_byte == end_byte {
            true
        } else {
            line_end_byte >= start_byte && current_byte <= end_byte
        };

        if intersects && !line.trim().is_empty() {
            new_text.push_str(prefix);
            new_text.push_str(line);
        } else {
            new_text.push_str(line);
        }
        new_text.push('\n');
        current_byte = line_end_byte + 1;
    }
    if !text.ends_with('\n') && new_text.ends_with('\n') {
        new_text.pop();
    }
    state.song_editor.text = new_text;
    true
}

/// Numbers lines within the current selection.
pub fn number_selected_lines(state: &mut ControlPanelState) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    let text = &state.song_editor.text;
    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);

    let mut counter = 0usize;
    let mut new_text = String::with_capacity(text.len() + 32);
    let mut current_byte = 0;
    for line in text.split('\n') {
        let line_len = line.len();
        let line_end_byte = current_byte + line_len;
        let intersects = if start_byte == end_byte {
            true
        } else {
            line_end_byte >= start_byte && current_byte <= end_byte
        };

        if intersects && !line.trim().is_empty() {
            counter += 1;
            new_text.push_str(&format!("{counter}. {line}"));
        } else {
            new_text.push_str(line);
        }
        new_text.push('\n');
        current_byte = line_end_byte + 1;
    }
    if !text.ends_with('\n') && new_text.ends_with('\n') {
        new_text.pop();
    }
    state.song_editor.text = new_text;
    true
}

/// Indents or decreases indent of lines within the current selection.
pub fn indent_selected_lines(state: &mut ControlPanelState, indent: bool) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    let text = &state.song_editor.text;
    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);

    let mut new_text = String::with_capacity(text.len() + 32);
    let mut current_byte = 0;
    for line in text.split('\n') {
        let line_len = line.len();
        let line_end_byte = current_byte + line_len;
        let intersects = if start_byte == end_byte {
            true
        } else {
            line_end_byte >= start_byte && current_byte <= end_byte
        };

        if intersects && !line.trim().is_empty() {
            if indent {
                new_text.push_str("  ");
                new_text.push_str(line);
            } else {
                let trimmed = line
                    .strip_prefix("  ")
                    .or_else(|| line.strip_prefix(' '))
                    .unwrap_or(line);
                new_text.push_str(trimmed);
            }
        } else {
            new_text.push_str(line);
        }
        new_text.push('\n');
        current_byte = line_end_byte + 1;
    }
    if !text.ends_with('\n') && new_text.ends_with('\n') {
        new_text.pop();
    }
    state.song_editor.text = new_text;
    true
}

/// Toggles comments (`//`) on lines within the current selection.
pub fn toggle_comment_selection(state: &mut ControlPanelState) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    let text = &state.song_editor.text;
    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);

    let mut new_text = String::with_capacity(text.len() + 16);
    let mut current_byte = 0;
    for line in text.split('\n') {
        let line_len = line.len();
        let line_end_byte = current_byte + line_len;
        let intersects = line_end_byte >= start_byte && current_byte <= end_byte;

        if intersects && !line.trim().is_empty() {
            if let Some(stripped) = line.strip_prefix("// ") {
                new_text.push_str(stripped);
            } else if let Some(stripped) = line.strip_prefix("//") {
                new_text.push_str(stripped);
            } else {
                new_text.push_str("// ");
                new_text.push_str(line);
            }
        } else {
            new_text.push_str(line);
        }
        new_text.push('\n');
        current_byte = line_end_byte + 1;
    }
    if !text.ends_with('\n') && new_text.ends_with('\n') {
        new_text.pop();
    }
    state.song_editor.text = new_text;
    true
}

/// Applies custom font tag `[fonte: font_name]` to current selection.
pub fn apply_font_selection(state: &mut ControlPanelState, font_name: &str) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    if start_char == end_char {
        return false;
    }
    let (updated, new_start, new_end) = crate::core::markup::apply_font_to_selection(
        &state.song_editor.text,
        start_char,
        end_char,
        font_name,
    );
    state.song_editor.text = updated;
    state.song_editor.selection = (new_start, new_end);
    true
}

/// Applies custom color tag `[cor: hex]` to current selection.
pub fn apply_color_selection(state: &mut ControlPanelState, hex: &str) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    if start_char == end_char {
        return false;
    }
    let (updated, new_start, new_end) = crate::core::markup::apply_color_to_selection(
        &state.song_editor.text,
        start_char,
        end_char,
        hex,
    );
    state.song_editor.text = updated;
    state.song_editor.selection = (new_start, new_end);
    true
}

/// Applies custom size tag `[tamanho: size]` to current selection.
pub fn apply_size_selection(state: &mut ControlPanelState, size_str: &str) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    if start_char == end_char {
        return false;
    }
    let (updated, new_start, new_end) = crate::core::markup::apply_size_to_selection(
        &state.song_editor.text,
        start_char,
        end_char,
        size_str,
    );
    state.song_editor.text = updated;
    state.song_editor.selection = (new_start, new_end);
    true
}

/// Applies custom shadow tag `[sombra: hex]` to current selection.
pub fn apply_shadow_selection(state: &mut ControlPanelState, hex: &str) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    if start_char == end_char {
        return false;
    }
    let (updated, new_start, new_end) = crate::core::markup::apply_shadow_to_selection(
        &state.song_editor.text,
        start_char,
        end_char,
        hex,
    );
    state.song_editor.text = updated;
    state.song_editor.selection = (new_start, new_end);
    true
}

/// Applies line alignment directive to current selection / stanza.
pub fn apply_align_directive(state: &mut ControlPanelState, align: &str) -> bool {
    let snippet = format!("\n//alinhar: {align}\n");
    insert_text_at_selection(state, &snippet)
}

/// Clears markdown formatting (`**`, `*`, `_`, `~`, `==`, `[fonte: ...]`, `[cor: ...]`, `[tamanho: ...]`, `[sombra: ...]`).
pub fn strip_formatting_from_selection(state: &mut ControlPanelState) -> bool {
    let text = &state.song_editor.text;
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );

    if start_char == end_char {
        let mut out = String::new();
        let mut blank_run = 0;
        for line in state.song_editor.text.lines() {
            if line.trim().is_empty() {
                blank_run += 1;
                if blank_run > 1 {
                    continue;
                }
                out.push('\n');
            } else {
                blank_run = 0;
                out.push_str(line.trim_end());
                out.push('\n');
            }
        }
        state.song_editor.text = out.trim_end().to_owned();
        return true;
    }

    let (updated, new_start, new_end) = crate::core::markup::clear_selection_formatting(
        text,
        start_char,
        end_char,
    );
    state.song_editor.text = updated;
    state.song_editor.selection = (new_start, new_end);
    true
}

/// Fügt `#*Name` als eigene Zeile am Anfang der Cursorzeile ein. Steht die Zeile mitten
/// in einer Strophe, wird davor eine Leerzeile gesetzt, damit eine neue Folie beginnt.
pub fn insert_stanza_label(state: &mut ControlPanelState, name: &str) -> bool {
    let text = state.song_editor.text.clone();
    let cursor_char = state.song_editor.selection.0.min(state.song_editor.selection.1);
    let cursor_byte = char_to_byte_idx(&text, cursor_char).min(text.len());
    let line_start = text[..cursor_byte].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = text[line_start..].find('\n').map(|i| line_start + i).unwrap_or(text.len());
    let current_line = text[line_start..line_end].trim();

    // Bereits eine Bezeichnung in dieser Zeile → ersetzen statt doppelt einfügen.
    if current_line.starts_with("#*") {
        let label = format!("#*{name}");
        let updated = format!("{}{}{}", &text[..line_start], label, &text[line_end..]);
        let pos = text[..line_start].chars().count() + label.chars().count();
        state.song_editor.text = updated;
        state.song_editor.selection = (pos, pos);
        return true;
    }

    let before = &text[..line_start];
    let needs_blank = !before.trim().is_empty() && !before.ends_with("\n\n");
    let mut snippet = String::new();
    if needs_blank {
        snippet.push('\n');
    }
    snippet.push_str("#*");
    snippet.push_str(name);
    snippet.push('\n');
    let updated = format!("{}{}{}", before, snippet, &text[line_start..]);
    let pos = before.chars().count() + snippet.chars().count();
    state.song_editor.text = updated;
    state.song_editor.selection = (pos, pos);
    true
}

/// Inserts text at current selection / cursor position.
pub fn insert_text_at_selection(state: &mut ControlPanelState, snippet: &str) -> bool {
    let text = &state.song_editor.text;
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );

    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);

    let new_text = format!("{}{}{}", &text[..start_byte], snippet, &text[end_byte..]);
    let snippet_chars = snippet.chars().count();
    let new_pos = start_char + snippet_chars;
    state.song_editor.text = new_text;
    state.song_editor.selection = (new_pos, new_pos);
    true
}

/// Inserts or updates a slide directive line (e.g. `//fonte: Montserrat`) in the stanza where the cursor or selection is located.
pub fn apply_or_update_slide_directive(
    state: &mut ControlPanelState,
    directive: &str,
    value: &str,
) -> bool {
    let single_alias = [directive];
    let aliases: &[&str] = match directive {
        "t" | "tamanho" | "size" | "s" => &["t", "tamanho", "size", "s"],
        "f" | "fonte" | "font" => &["f", "fonte", "font"],
        "c" | "cor" | "color" => &["c", "cor", "color"],
        "a" | "alinhar" | "align" => &["a", "alinhar", "align"],
        "l" | "linha" | "line" => &["l", "linha", "line"],
        "ls" | "letra" | "letter" => &["ls", "letra", "letter"],
        "sombra" | "shadow" | "fundo" | "box" | "b" => &["sombra", "shadow", "fundo", "box", "b"],
        _ => &single_alias,
    };

    let text = &state.song_editor.text;
    if text.is_empty() {
        state.song_editor.text = format!("//{directive}: {value}\n\n");
        state.song_editor.selection = (0, 0);
        return true;
    }

    let cursor_char = state.song_editor.selection.0.min(state.song_editor.selection.1);
    let cursor_byte = char_to_byte_idx(text, cursor_char);

    // Encontra os limites da estrofe atual (delimitada por linhas em branco \n\n ou início/fim do texto)
    let stanza_start_byte = text[..cursor_byte]
        .rfind("\n\n")
        .map(|idx| idx + 2)
        .unwrap_or(0);
    let stanza_end_byte = text[cursor_byte..]
        .find("\n\n")
        .map(|idx| cursor_byte + idx)
        .unwrap_or(text.len());

    let prefix_text = &text[..stanza_start_byte];
    let stanza_text = &text[stanza_start_byte..stanza_end_byte];
    let suffix_text = &text[stanza_end_byte..];

    let new_line = format!("//{directive}: {value}");

    let mut found = false;
    let mut updated_lines = Vec::new();
    for line in stanza_text.lines() {
        let trimmed = line.trim_start();
        let matches_alias = aliases.iter().any(|alias| {
            let p1 = format!("//{alias}:");
            let p2 = format!("// {alias}:");
            let p3 = format!("//{alias} ");
            let p4 = format!("// {alias} ");
            trimmed.starts_with(&p1) || trimmed.starts_with(&p2) || trimmed.starts_with(&p3) || trimmed.starts_with(&p4)
        });

        if matches_alias {
            updated_lines.push(new_line.clone());
            found = true;
        } else {
            updated_lines.push(line.to_string());
        }
    }

    let updated_stanza = if found {
        updated_lines.join("\n")
    } else {
        format!("{new_line}\n{stanza_text}")
    };

    let new_text = format!("{prefix_text}{updated_stanza}{suffix_text}");
    state.song_editor.text = new_text;
    true
}

/// Normalizes extra blank lines in the lyrics.
pub fn normalize_whitespace(state: &mut ControlPanelState) -> bool {
    let mut out = String::new();
    let mut blank_count = 0;
    for line in state.song_editor.text.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                out.push('\n');
            }
        } else {
            blank_count = 0;
            out.push_str(line.trim_end());
            out.push('\n');
        }
    }
    state.song_editor.text = out.trim_end().to_owned();
    true
}



/// Fügt den Sprachtrenner (unsichtbare Zeile mit U+200D) ein, ohne doppelte
/// Zeilenumbrüche: Steht der Cursor schon am Zeilenanfang, wird kein Umbruch
/// davor gesetzt; folgt direkt ein Umbruch, wird keiner danach gesetzt.
/// Der Cursor landet am Anfang der Zeile nach dem Trenner.
pub fn insert_language_spacer(state: &mut ControlPanelState) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    let (text, pos) = language_spacer_edit(&state.song_editor.text, start_char, end_char);
    state.song_editor.text = text;
    state.song_editor.selection = (pos, pos);
    true
}

fn language_spacer_edit(text: &str, start_char: usize, end_char: usize) -> (String, usize) {
    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);
    let before = &text[..start_byte];
    let after = &text[end_byte..];
    let need_leading = !before.is_empty() && !before.ends_with('\n');
    let next_is_newline = after.starts_with('\n');
    let mut snippet = String::new();
    if need_leading {
        snippet.push('\n');
    }
    snippet.push('\u{200D}');
    if !next_is_newline {
        snippet.push('\n');
    }
    let new_pos = start_char + snippet.chars().count() + usize::from(next_is_newline);
    (format!("{before}{snippet}{after}"), new_pos)
}

#[cfg(test)]
mod spacer_tests {
    use super::language_spacer_edit as run;

    #[test]
    fn after_text_at_end() {
        assert_eq!(run("தமிழ்", 5, 5).0, "தமிழ்\n\u{200D}\n");
    }

    #[test]
    fn at_start_of_empty_line_no_double_break() {
        assert_eq!(run("தமிழ்\n", 6, 6).0, "தமிழ்\n\u{200D}\n");
    }

    #[test]
    fn before_existing_line_break() {
        let (t, pos) = run("தமிழ்\nYesu", 5, 5);
        assert_eq!(t, "தமிழ்\n\u{200D}\nYesu");
        assert_eq!(pos, 8);
    }

    #[test]
    fn on_empty_line_between_blocks() {
        let (t, pos) = run("தமிழ்\n\nYesu", 6, 6);
        assert_eq!(t, "தமிழ்\n\u{200D}\nYesu");
        assert_eq!(pos, 8);
    }
}

/// Setzt eine Folien-Anweisung (Schrift, Größe, Farbe, Ausrichtung …) auf
/// ALLEN Folien des Liedes, nicht nur auf der Strophe mit dem Cursor.
pub fn apply_directive_all_stanzas(
    state: &mut ControlPanelState,
    directive: &str,
    value: &str,
) -> bool {
    let original_selection = state.song_editor.selection;
    let mut changed = false;
    let mut index = 0;
    loop {
        let text = state.song_editor.text.clone();
        // Anfangsbytes aller nicht-leeren Strophen (getrennt durch Leerzeilen).
        let mut starts = Vec::new();
        let mut pos = 0;
        for chunk in text.split("\n\n") {
            if !chunk.trim().is_empty() {
                starts.push(pos);
            }
            pos += chunk.len() + 2;
        }
        let Some(&start_byte) = starts.get(index) else {
            break;
        };
        let start_char = text[..start_byte].chars().count();
        state.song_editor.selection = (start_char, start_char);
        changed |= apply_or_update_slide_directive(state, directive, value);
        index += 1;
        if index > 500 {
            break;
        }
    }
    let len = state.song_editor.text.chars().count();
    state.song_editor.selection = (
        original_selection.0.min(len),
        original_selection.1.min(len),
    );
    changed
}

#[cfg(test)]
mod stanza_label_tests {
    use super::*;

    #[test]
    fn inserts_label_with_blank_line_and_replaces_existing() {
        let mut state = ControlPanelState::default();
        state.song_editor.text = "Zeile 1\nZeile 2".into();
        state.song_editor.selection = (9, 9); // in "Zeile 2"
        assert!(insert_stanza_label(&mut state, "Refrain"));
        assert_eq!(state.song_editor.text, "Zeile 1\n\n#*Refrain\nZeile 2");
        // Cursor steht jetzt am Anfang von "Zeile 2"; auf die Label-Zeile setzen und ersetzen.
        state.song_editor.selection = (10, 10);
        assert!(insert_stanza_label(&mut state, "Bridge"));
        assert_eq!(state.song_editor.text, "Zeile 1\n\n#*Bridge\nZeile 2");
    }

    #[test]
    fn inserts_at_start_without_blank_line() {
        let mut state = ControlPanelState::default();
        state.song_editor.text = "Text".into();
        state.song_editor.selection = (0, 0);
        insert_stanza_label(&mut state, "Strophe");
        assert_eq!(state.song_editor.text, "#*Strophe\nText");
    }
}
