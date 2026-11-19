use super::bible::{BibleTranslation, fold};

/// Keep leading book numbers (`1 Samuel`) separate from chapter/verse numbers.
pub fn split_reference(query: &str) -> (String, String) {
    let folded = fold(query).replace(' ', "");
    let mut seen_letter = false;
    for (index, c) in folded.char_indices() {
        if c.is_alphabetic() {
            seen_letter = true;
        } else if c == ':' || c == '.' || (c.is_ascii_digit() && seen_letter) {
            return (folded[..index].to_owned(), folded[index..].to_owned());
        }
    }
    (folded, String::new())
}

/// Complete numbers once another digit would exceed the chapter/verse count.
pub fn number_is_complete(text: &str, max: usize) -> bool {
    let Ok(value) = text.parse::<usize>() else {
        return false;
    };
    let max = max.max(1);
    let digits = max.ilog10() as usize + 1;
    value >= 1 && (text.len() >= digits || value > max / 10)
}

fn book_keys(bible: &BibleTranslation, index: usize) -> Vec<String> {
    let mut keys = vec![
        fold(&bible.books[index].name).replace(' ', ""),
        fold(&bible.books[index].abbrev).replace(' ', ""),
    ];
    if index < 66 {
        for name in [
            crate::core::bible_books::BOOK_NAMES_PT[index],
            crate::core::bible_books::BOOK_NAMES_EN[index],
            crate::core::bible_books::BOOK_NAMES_ES[index],
        ] {
            keys.push(fold(name).replace(' ', ""));
        }
        for abbrev in [
            crate::core::bible_books::BOOK_ABBREVS_PT[index],
            crate::core::bible_books::BOOK_ABBREVS_EN[index],
            crate::core::bible_books::BOOK_ABBREVS_ES[index],
        ] {
            keys.push(fold(abbrev).replace(' ', ""));
        }
        match index {
            8 => keys.push("1sam".into()),
            9 => keys.push("2sam".into()),
            10 => keys.push("1reis".into()),
            11 => keys.push("2reis".into()),
            12 => keys.push("1cron".into()),
            13 => keys.push("2cron".into()),
            18 => {
                keys.push("salmo".into());
                keys.push("salmos".into());
                keys.push("psalm".into());
                keys.push("psalms".into());
            }
            21 => {
                keys.push("cantares".into());
                keys.push("cantico".into());
                keys.push("canticos".into());
                keys.push("canticosdesalomao".into());
            }
            45 => keys.push("1cor".into()),
            46 => keys.push("2cor".into()),
            51 => keys.push("1tess".into()),
            52 => keys.push("2tess".into()),
            53 => keys.push("1tim".into()),
            54 => keys.push("2tim".into()),
            59 => keys.push("1ped".into()),
            60 => keys.push("2ped".into()),
            _ => {}
        }
    }
    keys.retain(|k| !k.is_empty());
    keys
}

pub fn matching_books(bible: &BibleTranslation, query: &str) -> Vec<usize> {
    let query = fold(query).replace(' ', "");
    if query.is_empty() {
        return Vec::new();
    }
    let reference_prefix = |key: &str| {
        !key.is_empty()
            && query.strip_prefix(key).is_some_and(|rest| {
                rest.starts_with(|c: char| c.is_ascii_digit() || c == ':' || c == '.')
            })
    };
    let mut matches: Vec<usize> = bible
        .books
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            let keys = book_keys(bible, *index);
            keys.iter().any(|k| k.starts_with(&query) || reference_prefix(k))
        })
        .map(|(index, _)| index)
        .collect();
    matches.sort_by_key(|&index| {
        let abbrev = fold(&bible.books[index].abbrev).replace(' ', "");
        let name = fold(&bible.books[index].name).replace(' ', "");
        let exact = name == query || abbrev == query;
        let abbrev_prefix = abbrev.starts_with(&query);
        (!exact, !abbrev_prefix, index)
    });
    matches
}

pub struct QueryParse {
    pub book: Option<usize>,
    pub matches: Vec<usize>,
    pub chapter: Option<usize>,
    pub verse: Option<usize>,
    pub has_separator: bool,
    pub book_complete: bool,
}

pub fn parse_query(bible: &BibleTranslation, query: &str) -> QueryParse {
    let (book_part, numbers) = split_reference(query);
    let matches = matching_books(bible, &book_part);
    let mut exact_name = None;
    let mut exact_other = None;
    for &index in &matches {
        let name_key = fold(&bible.books[index].name).replace(' ', "");
        let abbrev_key = fold(&bible.books[index].abbrev).replace(' ', "");
        if !name_key.is_empty() && name_key == book_part {
            exact_name = Some(index);
            break;
        }
        if exact_other.is_none() && !abbrev_key.is_empty() && abbrev_key == book_part {
            exact_other = Some(index);
        }
        if index < 66 {
            let canon_name = fold(crate::core::bible_books::BOOK_NAMES_PT[index]).replace(' ', "");
            if !canon_name.is_empty() && canon_name == book_part {
                exact_name = Some(index);
                break;
            }
            if exact_other.is_none() {
                for key in book_keys(bible, index) {
                    if key == book_part {
                        exact_other = Some(index);
                        break;
                    }
                }
            }
        }
    }
    let exact = exact_name.or(exact_other);
    let has_delimiter = query.ends_with(' ') || !numbers.is_empty();
    let book_complete = (exact.is_some() && (matches.len() == 1 || has_delimiter))
        || (exact.is_none() && matches.len() == 1 && !book_part.is_empty());
    let book = exact.or_else(|| matches.first().copied());
    let has_separator = numbers.contains([':', '.']);
    let (chapter, verse) = if numbers.is_empty() {
        (None, None)
    } else {
        match numbers.split_once([':', '.']) {
            Some((chapter, verse)) => (chapter.parse().ok(), verse.parse().ok()),
            None => (numbers.parse().ok(), None),
        }
    };
    QueryParse {
        book,
        matches,
        chapter,
        verse,
        has_separator,
        book_complete,
    }
}
