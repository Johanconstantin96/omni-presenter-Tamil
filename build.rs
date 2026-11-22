use serde::Deserialize;
use std::{collections::HashMap, env, fs, path::Path};

#[path = "build_support/web_assets.rs"]
mod web_assets;

#[derive(Deserialize)]
struct IndexLanguage {
    language: String,
    versions: Vec<IndexVersion>,
}

#[derive(Deserialize)]
struct IndexVersion {
    name: String,
    abbreviation: String,
}

#[derive(Clone, Debug)]
struct BibleFileEntry {
    file_name: String,
    stem: String,
    id: String,
    name: String,
    language: String,
    is_pt: bool,
}

fn localize_language(lang: &str) -> &'static str {
    match lang.to_lowercase().as_str() {
        "portuguese" => "Portugiesisch",
        "english" => "Englisch",
        "spanish" => "Spanisch",
        "afrikaans" => "Afrikaans",
        "akan" => "Akan",
        "albanian" => "Albanisch",
        "ancient greek" => "Altgriechisch",
        "arabic" => "Arabisch",
        "armenian" => "Armenisch",
        "assamese" => "Assamesisch",
        "basque" => "Baskisch",
        "belarusian" => "Belarussisch",
        "bengali" => "Bengalisch",
        "bislama" => "Bislama",
        "bulgarian" => "Bulgarisch",
        "burmese" => "Birmanisch",
        "catalan" => "Katalanisch",
        "cebuano" => "Cebuano",
        "central kurdish" => "Zentralkurdisch",
        "chinese" => "Chinesisch",
        "croatian" => "Kroatisch",
        "czech" => "Tschechisch",
        "dutch" => "Niederländisch",
        "esperanto" => "Esperanto",
        "estonian" => "Estnisch",
        "ewe" => "Ewe",
        "filipino" => "Filipino",
        "finnish" => "Finnisch",
        "french" => "Französisch",
        "german" => "Deutsch",
        "greek" => "Neugriechisch",
        "gujarati" => "Gujarati",
        "haitian creole" => "Haitianisch",
        "hausa" => "Hausa",
        "hebrew" => "Hebräisch",
        "hindi" => "Hindi",
        "hungarian" => "Ungarisch",
        "icelandic" => "Isländisch",
        "igbo" => "Igbo",
        "indonesian" => "Indonesisch",
        "italian" => "Italienisch",
        "japanese" => "Japanisch",
        "kannada" => "Kannada",
        "kazakh" => "Kasachisch",
        "kikuyu" => "Kikuyu",
        "kinyarwanda" => "Kinyarwanda",
        "kirundi" => "Kirundi",
        "korean" => "Koreanisch",
        "latin" => "Latein",
        "latvian" => "Lettisch",
        "lingala" => "Lingala",
        "luganda" => "Luganda",
        "malay" => "Malaiisch",
        "malayalam" => "Malayalam",
        "maori" => "Maori",
        "marathi" => "Marathi",
        "nepali" => "Nepalesisch",
        "northern ndebele" => "Nord-Ndebele",
        "norwegian (bokmål)" | "norwegian" => "Norueguês",
        "nyanja" => "Nyanja",
        "odia" => "Oriya",
        "oromo" => "Oromo",
        "persian" => "Persisch",
        "polish" => "Polnisch",
        "punjabi" => "Panjabi",
        "romanian" => "Rumänisch",
        "russian" => "Russisch",
        "sanskrit" => "Sanskrit",
        "scottish gaelic" => "Schottisch-Gälisch",
        "serbian" => "Serbisch",
        "shona" => "Shona",
        "slovak" => "Slowakisch",
        "swahili" => "Suaheli",
        "swedish" => "Schwedisch",
        "tamil" => "Tamil",
        "telugu" => "Telugu",
        "thai" => "Thailändisch",
        "tongan" => "Tongaisch",
        "tswana" => "Tswana",
        "turkish" => "Türkisch",
        "ukrainian" => "Ukrainisch",
        "urdu" => "Urdu",
        "vietnamese" => "Vietnamesisch",
        "yiddish" => "Jiddisch",
        "yoruba" => "Yoruba",
        _ => "Andere Sprache",
    }
}

fn main() {
    let manifest = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).to_path_buf();
    build_web_assets(&manifest);
    embed_locales(&manifest);
    let folder = manifest.join("assets/bible-source/json");
    println!("cargo:rerun-if-changed={}", folder.display());
    assert!(folder.join("index.json").is_file(), "Bible submodule is missing; run `git submodule update --init --recursive`");

    // Load metadata map from index.json if present
    let mut meta_map: HashMap<String, (String, String)> = HashMap::new();
    let index_file = folder.join("index.json");
    if index_file.is_file() {
        println!("cargo:rerun-if-changed={}", index_file.display());
        if let Ok(content) = fs::read(&index_file)
            && let Ok(languages) = serde_json::from_slice::<Vec<IndexLanguage>>(&content)
        {
            for lang in languages {
                for v in lang.versions {
                    meta_map.insert(
                        v.abbreviation.to_lowercase(),
                        (v.name, localize_language(&lang.language).to_string()),
                    );
                }
            }
        }
    }

    let mut entries: Vec<BibleFileEntry> = fs::read_dir(&folder)
        .map(|dir_entries| {
            dir_entries
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    let name = path.file_name()?.to_str()?.to_owned();
                    if name == "index.json" || name == "book_names.json"
                        || path.extension().and_then(|e| e.to_str()) != Some("json")
                    {
                        return None;
                    }
                    let stem = name.trim_end_matches(".json").to_owned();
                    let is_pt = stem.starts_with("pt_");
                    let id = if let Some((lang, rest)) = stem.split_once('_') {
                        if (2..=4).contains(&lang.len())
                            && lang.chars().all(|c| c.is_ascii_alphabetic())
                            && !rest.is_empty()
                        {
                            rest.to_uppercase()
                        } else {
                            stem.to_uppercase()
                        }
                    } else {
                        stem.to_uppercase()
                    };
                    let (version_name, language) = meta_map
                        .get(&stem.to_lowercase())
                        .cloned()
                        .unwrap_or_else(|| {
                            if is_pt {
                                (format!("Bíblia ({id})"), "Português".to_string())
                            } else {
                                (format!("Bíblia ({id})"), "Outro".to_string())
                            }
                        });
                    Some(BibleFileEntry {
                        file_name: name,
                        stem,
                        id,
                        name: version_name,
                        language,
                        is_pt,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Sort entries: Portuguese first (prioritizing common ones), then others by language and name
    let pt_priority = |id: &str| -> usize {
        match id {
            "ARA" => 0,
            "NVI" => 1,
            "NVT" => 2,
            "ACF" => 3,
            "ARC" => 4,
            "NAA" => 5,
            "NTLH" => 6,
            "KJA" => 7,
            "BKJ" => 8,
            "A21" => 9,
            "AA" => 10,
            "TB" => 11,
            "NBV" => 12,
            "VFL" => 13,
            _ => 50,
        }
    };

    let lang_priority = |lang: &str| -> usize {
        match lang {
            "Português" => 0,
            "Inglês" => 1,
            "Espanhol" => 2,
            _ => 10,
        }
    };

    entries.sort_by(|a, b| {
        let la = lang_priority(&a.language);
        let lb = lang_priority(&b.language);
        if la != lb {
            la.cmp(&lb)
        } else if a.is_pt && b.is_pt {
            let pa = pt_priority(&a.id);
            let pb = pt_priority(&b.id);
            pa.cmp(&pb).then_with(|| a.id.cmp(&b.id))
        } else {
            a.language.cmp(&b.language).then_with(|| a.name.cmp(&b.name))
        }
    });

    let mut output = String::new();
    output.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n");
    output.push_str("pub struct BundledBibleMeta {\n");
    output.push_str("    pub stem: &'static str,\n");
    output.push_str("    pub id: &'static str,\n");
    output.push_str("    pub name: &'static str,\n");
    output.push_str("    pub language: &'static str,\n");
    output.push_str("    pub is_pt: bool,\n");
    output.push_str("}\n\n");

    output.push_str("pub static BUNDLED_BIBLES: &[(&str, &[u8])] = &[\n");
    for entry in &entries {
        output.push_str(&format!(
            "    (\"{}\", include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/assets/bible-source/json/{}\"))),\n",
            entry.stem, entry.file_name
        ));
    }
    output.push_str("];\n\n");

    output.push_str("pub static BUNDLED_BIBLES_META: &[BundledBibleMeta] = &[\n");
    for entry in &entries {
        // Escape quotes if any in name
        let escaped_name = entry.name.replace('\\', "\\\\").replace('"', "\\\"");
        output.push_str(&format!(
            "    BundledBibleMeta {{ stem: \"{}\", id: \"{}\", name: \"{}\", language: \"{}\", is_pt: {} }},\n",
            entry.stem, entry.id, escaped_name, entry.language, entry.is_pt
        ));
    }
    output.push_str("];\n");

    let target = Path::new(&env::var("OUT_DIR").unwrap()).join("bible_assets.rs");
    fs::write(target, output).unwrap();
}


fn embed_locales(manifest: &Path) {
    let root = manifest.join("locales");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut locales: Vec<_> = fs::read_dir(&root).expect("missing locales directory")
        .flatten().map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.join("locale.json").is_file()).collect();
    locales.sort();
    let mut output = String::from("pub static BUNDLED_LOCALES: &[(&str, &str, &[(&str, &str)])] = &[\n");
    for locale in locales {
        let code = locale.file_name().unwrap().to_str().unwrap();
        let mut files: Vec<_> = fs::read_dir(&locale).unwrap().flatten().map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("ftl")).collect();
        files.sort();
        output.push_str(&format!("({code:?}, include_str!({:?}), &[\n", locale.join("locale.json")));
        for file in files {
            output.push_str(&format!("({:?}, include_str!({file:?})),\n", file.file_name().unwrap().to_str().unwrap()));
        }
        output.push_str("]),\n");
    }
    output.push_str("];\n");
    fs::write(Path::new(&env::var("OUT_DIR").unwrap()).join("locale_assets.rs"), output).unwrap();
}

fn build_web_assets(manifest: &Path) {
    let web_dir = manifest.join("web");
    println!("cargo:rerun-if-changed={}", web_dir.join("src").display());
    println!("cargo:rerun-if-changed={}", web_dir.join("package.json").display());
    println!("cargo:rerun-if-changed={}", web_dir.join("tsconfig.json").display());
    println!("cargo:rerun-if-changed={}", web_dir.join("build.ts").display());
    println!("cargo:rerun-if-changed={}", web_dir.join("bun.lock").display());
    println!("cargo:rerun-if-changed=build_support/web_assets.rs");
    println!("cargo:rerun-if-env-changed=BUN");
    let bun = env::var_os("BUN").unwrap_or_else(|| "bun".into());
    web_assets::compile_web_assets(manifest, &bun)
        .unwrap_or_else(|error| panic!("{error}"));
}
