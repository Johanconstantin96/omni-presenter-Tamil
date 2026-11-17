//! Operating System desktop wallpaper detection.
//! Detects current user desktop background on Linux, Windows, and macOS.

use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

/// Decodes percent-encoded file URI or string into a clean filesystem path.
pub fn decode_uri_path(raw: &str) -> String {
    let raw = raw.trim().trim_matches('\'').trim_matches('"');
    let path_str = raw.strip_prefix("file://").unwrap_or(raw);
    let mut bytes = Vec::new();
    let mut iter = path_str.bytes();
    while let Some(b) = iter.next() {
        if b == b'%' {
            let h1 = iter.next();
            let h2 = iter.next();
            if let (Some(c1), Some(c2)) = (h1, h2) {
                if let Ok(val) = u8::from_str_radix(
                    std::str::from_utf8(&[c1, c2]).unwrap_or(""),
                    16,
                ) {
                    bytes.push(val);
                    continue;
                }
                bytes.push(b'%');
                bytes.push(c1);
                bytes.push(c2);
                continue;
            }
        }
        bytes.push(b);
    }
    String::from_utf8_lossy(&bytes).to_string()
}

/// Detects the OS desktop wallpaper file path if available.
pub fn detect_os_wallpaper() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        // 1. GNOME / Cinnamon / MATE via gsettings
        const GSETTINGS_PROBES: &[(&str, &str)] = &[
            ("org.gnome.desktop.background", "picture-uri-dark"),
            ("org.gnome.desktop.background", "picture-uri"),
            ("org.cinnamon.desktop.background", "picture-uri"),
            ("org.mate.background", "picture-filename"),
        ];

        for (schema, key) in GSETTINGS_PROBES {
            if let Ok(output) = std::process::Command::new("gsettings")
                .args(["get", schema, key])
                .output()
                && output.status.success()
            {
                let raw = String::from_utf8_lossy(&output.stdout);
                let decoded = decode_uri_path(&raw);
                if !decoded.is_empty() && Path::new(&decoded).is_file() {
                    return Some(decoded);
                }
            }
        }

        // 2. Fallback: ~/.config/background (standard GNOME symlink/target)
        if let Some(home) = std::env::var_os("HOME") {
            let gnome_bg = PathBuf::from(&home).join(".config/background");
            if gnome_bg.is_file() {
                return Some(gnome_bg.to_string_lossy().to_string());
            }

            // 3. KDE Plasma config file parsing
            let kde_cfg = PathBuf::from(&home).join(".config/plasma-org.kde.plasma.desktop-appletsrc");
            if kde_cfg.is_file()
                && let Ok(content) = std::fs::read_to_string(&kde_cfg)
            {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if let Some(rest) = trimmed.strip_prefix("Image=") {
                        let decoded = decode_uri_path(rest);
                        if !decoded.is_empty() && Path::new(&decoded).is_file() {
                            return Some(decoded);
                        }
                    }
                }
            }
        }

        // 4. XFCE via xfconf-query
        if let Ok(output) = std::process::Command::new("xfconf-query")
            .args(["-c", "xfce4-desktop", "-p", "/backdrop/screen0/monitor0/workspace0/last-image"])
            .output()
            && output.status.success()
        {
            let raw = String::from_utf8_lossy(&output.stdout);
            let decoded = decode_uri_path(&raw);
            if !decoded.is_empty() && Path::new(&decoded).is_file() {
                return Some(decoded);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Read registry key for current wallpaper
        if let Ok(output) = std::process::Command::new("reg")
            .args(["query", "HKCU\\Control Panel\\Desktop", "/v", "Wallpaper"])
            .output()
            && output.status.success()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("Wallpaper") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let candidate = parts[2..].join(" ");
                        if Path::new(&candidate).is_file() {
                            return Some(candidate);
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let script = "tell application \"Finder\" to get POSIX path of (get desktop picture as alias)";
        if let Ok(output) = std::process::Command::new("osascript")
            .args(["-e", script])
            .output()
            && output.status.success()
        {
            let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !raw.is_empty() && Path::new(&raw).is_file() {
                return Some(raw);
            }
        }
    }

    None
}

struct WallpaperCache {
    path: Option<String>,
    checked_at: Instant,
}

static CACHE: OnceLock<Mutex<WallpaperCache>> = OnceLock::new();

/// Returns the cached OS desktop wallpaper, refreshing at most once every 30 seconds.
pub fn os_wallpaper() -> Option<String> {
    let cache_mutex = CACHE.get_or_init(|| {
        Mutex::new(WallpaperCache {
            path: detect_os_wallpaper(),
            checked_at: Instant::now(),
        })
    });

    let mut lock = cache_mutex.lock().ok()?;
    if lock.checked_at.elapsed() > Duration::from_secs(30) {
        lock.path = detect_os_wallpaper();
        lock.checked_at = Instant::now();
    }
    lock.path.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_uri_path_spaces_and_quotes() {
        assert_eq!(
            decode_uri_path("'file:///home/user/Pictures/My%20Wallpaper.png'"),
            "/home/user/Pictures/My Wallpaper.png"
        );
        assert_eq!(
            decode_uri_path("\"/home/user/bg.jpg\""),
            "/home/user/bg.jpg"
        );
        assert_eq!(
            decode_uri_path("file:///C:/Wallpapers/test%20file.jpg"),
            "/C:/Wallpapers/test file.jpg"
        );
    }

    #[test]
    fn test_os_wallpaper_smoke() {
        let _ = os_wallpaper();
    }
}
