//! Per-installation display settings: typeface, weight and size.
//!
//! `settings.json` lives next to the executable, like `quotes.json`, and is
//! exported with the defaults on first run so the knobs are discoverable
//! without reading the source. Every field is optional and falls back to a
//! built-in default, so a partially edited file still works. A file that
//! cannot be parsed, or values the renderer cannot express, are reported to
//! the user once at startup and replaced by the defaults — the strip always
//! renders.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Where the settings live relative to the executable.
pub const FILE_NAME: &str = "settings.json";

/// Default face: Microsoft YaHei UI, the standard Simplified-Chinese UI
/// face on Windows (GDI substitutes another face when it is absent).
pub const DEFAULT_FACE: &str = "Microsoft YaHei UI";
/// Default weight: 700 (bold) — Light 300 read as washed out at strip size,
/// and the family ships only Light/Regular/Bold.
pub const DEFAULT_WEIGHT: i32 = 700;
/// Default size in points for the 45-logical-px strip.
pub const DEFAULT_PT: i32 = 14;

/// `LOGFONTW::lfFaceName` is `[u16; 32]`: 31 units plus the terminator.
const MAX_FACE_UNITS: usize = 31;
const MIN_WEIGHT: i32 = 100;
const MAX_WEIGHT: i32 = 900;
const MIN_PT: i32 = 8;
const MAX_PT: i32 = 48;

/// Display settings as written in `settings.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Font face name as GDI knows it: `Microsoft YaHei UI`, `SimHei`,
    /// `SimSun`, `KaiTi`, `DengXian`, ...
    pub font: String,
    /// Font weight: 400 = regular, 700 = bold.
    pub weight: i32,
    /// Font size in points.
    pub size: i32,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            font: DEFAULT_FACE.to_string(),
            weight: DEFAULT_WEIGHT,
            size: DEFAULT_PT,
        }
    }
}

impl Settings {
    /// Replace values the renderer cannot express, reporting what changed.
    pub fn sanitized(mut self) -> (Settings, Option<String>) {
        let mut problems: Vec<String> = Vec::new();
        let face = self.font.trim().to_string();
        if face.is_empty() {
            problems.push("font 不能为空".to_string());
            self.font = DEFAULT_FACE.to_string();
        } else if face.encode_utf16().count() > MAX_FACE_UNITS {
            problems.push(format!("font 不能超过 {MAX_FACE_UNITS} 个字符"));
            self.font = DEFAULT_FACE.to_string();
        } else {
            self.font = face;
        }
        if !(MIN_WEIGHT..=MAX_WEIGHT).contains(&self.weight) {
            problems.push(format!(
                "weight 需在 {MIN_WEIGHT}–{MAX_WEIGHT} 之间，收到 {}",
                self.weight
            ));
            self.weight = DEFAULT_WEIGHT;
        }
        if !(MIN_PT..=MAX_PT).contains(&self.size) {
            problems.push(format!(
                "size 需在 {MIN_PT}–{MAX_PT} pt 之间，收到 {}",
                self.size
            ));
            self.size = DEFAULT_PT;
        }
        let note = (!problems.is_empty()).then(|| {
            format!(
                "settings.json 中有无法使用的值：{}。\n本次运行使用默认值，修正该文件后重启应用即可。",
                problems.join("；")
            )
        });
        (self, note)
    }
}

/// Parse `settings.json` (UTF-8, optional BOM). Pure.
///
/// Missing fields take their defaults; unknown fields are ignored so a
/// newer file never breaks an older build.
pub fn parse(raw: &str) -> Result<Settings, String> {
    let s = raw.strip_prefix('\u{feff}').unwrap_or(raw);
    if s.trim().is_empty() {
        return Err("文件为空".to_string());
    }
    serde_json::from_str(s).map_err(|e| format!("JSON 解析失败: {e}"))
}

/// Serialize the shape the user is meant to edit, with a trailing newline.
pub fn to_json(settings: &Settings) -> Result<String, String> {
    serde_json::to_string_pretty(settings)
        .map(|text| text + "\n")
        .map_err(|e| e.to_string())
}

/// Load the settings from `dir` (the executable's directory).
///
/// Missing file → write the defaults out, then use them. Present but
/// unreadable/unparseable → defaults plus a human-readable reason.
pub fn load(dir: &Path) -> (Settings, Option<String>) {
    let path = dir.join(FILE_NAME);
    let defaults = Settings::default();
    match fs::read_to_string(&path) {
        Ok(raw) => match parse(&raw) {
            Ok(settings) => settings.sanitized(),
            Err(reason) => (
                defaults,
                Some(format!(
                    "{}: {reason}\n本次运行使用默认字体。修正该文件后重启应用即可。",
                    path.display()
                )),
            ),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if let Ok(text) = to_json(&defaults) {
                let _ = fs::write(&path, text);
            }
            (defaults, None)
        }
        Err(e) => (
            defaults,
            Some(format!(
                "无法读取 {}: {e}\n本次运行使用默认字体。",
                path.display()
            )),
        ),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d =
            std::env::temp_dir().join(format!("jinju-set-{tag}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn defaults_are_the_shipped_look() {
        let d = Settings::default();
        assert_eq!(d.font, DEFAULT_FACE);
        assert_eq!(d.weight, 700);
        assert_eq!(d.size, 14);
    }

    #[test]
    fn parse_fills_missing_fields_with_defaults() {
        let s = parse(r#"{"font":"SimHei"}"#).unwrap();
        assert_eq!(s.font, "SimHei");
        assert_eq!(s.weight, DEFAULT_WEIGHT);
        assert_eq!(s.size, DEFAULT_PT);
    }

    #[test]
    fn parse_ignores_unknown_fields() {
        let s = parse(r#"{"font":"SimHei","colour":"red","size":12}"#).unwrap();
        assert_eq!(s.font, "SimHei");
        assert_eq!(s.size, 12);
    }

    #[test]
    fn parse_accepts_bom_and_windows_newlines() {
        let raw = "\u{feff}{\r\n  \"font\": \"KaiTi\"\r\n}";
        assert_eq!(parse(raw).unwrap().font, "KaiTi");
    }

    #[test]
    fn parse_rejects_empty_and_broken_files() {
        assert!(parse("  \n ").unwrap_err().contains("文件为空"));
        assert!(parse("{{{ nope").unwrap_err().contains("JSON"));
    }

    #[test]
    fn sanitize_rejects_out_of_range_values() {
        let (s, note) = parse(r#"{"weight":12,"size":400}"#).unwrap().sanitized();
        assert_eq!(s.weight, DEFAULT_WEIGHT);
        assert_eq!(s.size, DEFAULT_PT);
        let note = note.expect("a note for out-of-range values");
        assert!(note.contains("weight"), "{note}");
        assert!(note.contains("size"), "{note}");
    }

    #[test]
    fn sanitize_rejects_blank_and_overlong_faces() {
        let (s, note) = parse(r#"{"font":"   "}"#).unwrap().sanitized();
        assert_eq!(s.font, DEFAULT_FACE);
        assert!(note.unwrap().contains("font"));

        let long = format!(r#"{{"font":"{}"}}"#, "X".repeat(40));
        let (s, note) = parse(&long).unwrap().sanitized();
        assert_eq!(s.font, DEFAULT_FACE);
        assert!(note.unwrap().contains("31"));
    }

    #[test]
    fn sanitize_trims_a_user_supplied_face() {
        let (s, note) = parse(r#"{"font":"  SimHei  "}"#).unwrap().sanitized();
        assert_eq!(s.font, "SimHei");
        assert!(note.is_none());
    }

    #[test]
    fn to_json_round_trips() {
        let s = Settings {
            font: "SimHei".to_string(),
            weight: 400,
            size: 13,
        };
        assert_eq!(parse(&to_json(&s).unwrap()).unwrap(), s);
    }

    #[test]
    fn load_exports_defaults_when_missing() {
        let dir = temp_dir("export");
        assert!(!dir.join(FILE_NAME).exists());
        let (s, note) = load(&dir);
        assert_eq!(s, Settings::default());
        assert!(note.is_none());
        let on_disk = fs::read_to_string(dir.join(FILE_NAME)).unwrap();
        assert_eq!(parse(&on_disk).unwrap(), s);
        assert!(
            on_disk.contains("font"),
            "the exported file documents the knobs"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_reads_user_settings() {
        let dir = temp_dir("user");
        fs::write(
            dir.join(FILE_NAME),
            r#"{"font":"SimHei","weight":400,"size":13}"#,
        )
        .unwrap();
        let (s, note) = load(&dir);
        assert_eq!(s.font, "SimHei");
        assert_eq!((s.weight, s.size), (400, 13));
        assert!(note.is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_falls_back_on_broken_file() {
        let dir = temp_dir("broken");
        fs::write(dir.join(FILE_NAME), "not json").unwrap();
        let (s, note) = load(&dir);
        assert_eq!(s, Settings::default());
        assert!(note.unwrap().contains("JSON"));
        fs::remove_dir_all(&dir).ok();
    }
}
