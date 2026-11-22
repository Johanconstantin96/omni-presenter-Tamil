//! Per-output filtering of bilingual (Tamil + transliteration) song lyrics.
//!
//! Songs are written with Tamil-script lines and their Latin transliteration
//! ("Tanglish") lines on the same slide. Every output can choose which of the
//! two it shows. Bible passages and other content are never filtered.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LyricFilter {
    /// Show every line unchanged.
    #[default]
    All,
    /// Show only lines without Tamil script (the transliteration).
    LatinOnly,
    /// Like `LatinOnly`, but join every two lines into one (4 lines become 2).
    LatinCompact,
}

impl LyricFilter {
    pub const ALL: [LyricFilter; 3] = [LyricFilter::All, LyricFilter::LatinOnly, LyricFilter::LatinCompact];

    /// Fluent message id of the option label.
    pub fn label_key(self) -> &'static str {
        match self {
            LyricFilter::All => "settings-lyric-filter-all",
            LyricFilter::LatinOnly => "settings-lyric-filter-latin-only",
            LyricFilter::LatinCompact => "settings-lyric-filter-latin-compact",
        }
    }
}

/// True when the line contains at least one character of the Tamil Unicode block.
pub fn is_tamil_line(line: &str) -> bool {
    line.chars().any(|c| ('\u{0B80}'..='\u{0BFF}').contains(&c))
}

/// Lines that carry formatting or commands rather than lyrics.
fn is_command_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//") || t.starts_with("#*")
}

/// Applies `filter` to the text of one slide.
///
/// If the filter would remove every lyric line (for example a Tamil-only song
/// without transliteration), the original text is returned so the output never
/// goes blank by accident.
pub fn apply(text: &str, filter: LyricFilter) -> String {
    if filter == LyricFilter::All || text.trim().is_empty() {
        return text.to_owned();
    }
    let mut kept: Vec<&str> = Vec::new();
    let mut lyric_lines = 0usize;
    for line in text.lines() {
        if is_command_line(line) {
            kept.push(line);
        } else if line.trim().is_empty() {
            continue;
        } else if !is_tamil_line(line) {
            kept.push(line);
            lyric_lines += 1;
        }
    }
    if lyric_lines == 0 {
        return text.to_owned();
    }
    if filter == LyricFilter::LatinOnly {
        return kept.join("\n");
    }
    // LatinCompact: join consecutive lyric lines in pairs, keep commands in place.
    let mut out: Vec<String> = Vec::new();
    let mut pending: Option<String> = None;
    for line in kept {
        if is_command_line(line) {
            if let Some(p) = pending.take() {
                out.push(p);
            }
            out.push(line.to_owned());
            continue;
        }
        match pending.take() {
            None => pending = Some(line.trim_end().to_owned()),
            Some(first) => out.push(format!("{} {}", first, line.trim())),
        }
    }
    if let Some(p) = pending {
        out.push(p);
    }
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SLIDE: &str = "இயேசுவே என் தெய்வமே\nYesuve en dheivame\nஉம்மை நான் போற்றுவேன்\nUmmai naan potruvaen";
    const BLOCK: &str = "இயேசுவே என் தெய்வமே\nஉம்மை நான் போற்றுவேன்\nஎன்னைக் காக்கும் தேவனே\nஎன்றும் உம்மைப் பாடுவேன்\nYesuve en dheivame\nUmmai naan potruvaen\nEnnai kaakum devane\nEndrum ummai paaduvaen";

    #[test]
    fn all_is_unchanged() {
        assert_eq!(apply(SLIDE, LyricFilter::All), SLIDE);
    }

    #[test]
    fn latin_only_drops_tamil_lines() {
        assert_eq!(apply(SLIDE, LyricFilter::LatinOnly), "Yesuve en dheivame\nUmmai naan potruvaen");
    }

    #[test]
    fn compact_joins_pairs_with_a_space() {
        assert_eq!(
            apply(BLOCK, LyricFilter::LatinCompact),
            "Yesuve en dheivame Ummai naan potruvaen\nEnnai kaakum devane Endrum ummai paaduvaen"
        );
    }

    #[test]
    fn compact_keeps_odd_last_line() {
        assert_eq!(apply("A\nB\nC", LyricFilter::LatinCompact), "A B\nC");
    }

    #[test]
    fn tamil_only_song_falls_back_to_original() {
        let tamil = "இயேசுவே என் தெய்வமே\nஉம்மை நான் போற்றுவேன்";
        assert_eq!(apply(tamil, LyricFilter::LatinOnly), tamil);
    }

    #[test]
    fn command_lines_are_kept_and_not_paired() {
        let text = "//size: 40\nஇயேசுவே\nYesuve\nUmmai";
        assert_eq!(apply(text, LyricFilter::LatinCompact), "//size: 40\nYesuve Ummai");
    }

    #[test]
    fn markup_with_tamil_is_detected() {
        assert!(is_tamil_line("[color: #FFD700]இயேசுவே[/color]"));
        assert!(!is_tamil_line("**Yesuve** en dheivame"));
    }
}
