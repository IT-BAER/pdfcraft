//! Settings saved by several windows (Preferences ▸ Open each PDF in a new window): each window
//! writes the whole settings blob when it saves, so without a merge the last window to close
//! undid what the others changed (their preferences, recent files and remembered views). A save
//! merges this window's settings with the ones on disk instead: what this window changed since it
//! loaded or last saved them is kept, and everything else is taken from disk.

use serde_json::Value;

/// `mine` merged with `theirs` (the settings on disk now, perhaps written by another window)
/// against `base` (this window's settings when it last loaded or saved them). A key this window
/// left as it was in `base` takes `theirs`, so another window's change survives; a key it changed
/// keeps `mine`. Recent files and remembered views keep this window's list and add the files
/// another window added. Keys this version doesn't write are kept. Anything but two objects gives
/// `mine`.
pub(crate) fn merge(base: &Value, mut mine: Value, theirs: &Value) -> Value {
    let Some(theirs) = theirs.as_object() else { return mine };
    let recent_cap =
        mine["recent_limit"].as_u64().and_then(|n| usize::try_from(n).ok()).map_or(crate::RECENT_FILES_DEFAULT, |n| n.min(crate::RECENT_FILES_MAX));
    let Some(out) = mine.as_object_mut() else { return mine };
    for (key, their) in theirs {
        let merged = match out.get(key) {
            None => their.clone(),
            Some(own) if base.get(key) == Some(own) => their.clone(),
            Some(own) => match key.as_str() {
                "recent" => merge_files(base.get(key), own, their, recent_cap),
                "last_views" => merge_files(base.get(key), own, their, super::last_view::MAX_REMEMBERED),
                _ => continue,
            },
        };
        out.insert(key.clone(), merged);
    }
    mine
}

/// The largest settings file read back: drawn signatures and stamps make it large, never this large.
const MAX_SETTINGS_BYTES: u64 = 64 << 20;

/// The `pdfcraft` settings in eframe's settings file at `path` (a RON map of strings), or `None`
/// when it is missing, too large or unreadable (half written by another window, say).
pub(crate) fn read_saved(path: &std::path::Path) -> Option<Value> {
    use std::io::Read;
    let mut text = String::new();
    std::fs::File::open(path).ok()?.take(MAX_SETTINGS_BYTES.saturating_add(1)).read_to_string(&mut text).ok()?;
    if u64::try_from(text.len()).map_or(true, |n| n > MAX_SETTINGS_BYTES) {
        return None;
    }
    let map: std::collections::HashMap<String, String> = ron::from_str(&text).ok()?;
    serde_json::from_str(map.get("pdfcraft")?).ok()
}

/// This window's list of files (`mine`), then the entries of `theirs` for files that neither it
/// nor `base` names (added by another window), at most `cap`. A malformed list keeps `mine`.
fn merge_files(base: Option<&Value>, mine: &Value, theirs: &Value, cap: usize) -> Value {
    let (Some(mine), Some(theirs)) = (mine.as_array(), theirs.as_array()) else { return mine.clone() };
    let path = |e: &Value| e.get("path").and_then(Value::as_str).map(str::to_string);
    let base: Vec<String> = base.and_then(Value::as_array).map(|b| b.iter().filter_map(path).collect()).unwrap_or_default();
    let mut out = mine.clone();
    for entry in theirs {
        let Some(p) = path(entry) else { continue };
        if !base.contains(&p) && !out.iter().any(|o| path(o).as_ref() == Some(&p)) {
            out.push(entry.clone());
        }
    }
    out.truncate(cap);
    Value::Array(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_setting_another_window_changed_survives_this_windows_save() {
        let base = json!({ "theme": "Light", "units": "mm" });
        let mine = json!({ "theme": "Light", "units": "in" });
        let theirs = json!({ "theme": "Dark", "units": "mm" });
        assert_eq!(merge(&base, mine, &theirs), json!({ "theme": "Dark", "units": "in" }));
    }

    #[test]
    fn this_windows_change_wins_where_both_changed_a_setting() {
        let base = json!({ "theme": "Light" });
        assert_eq!(merge(&base, json!({ "theme": "System" }), &json!({ "theme": "Dark" })), json!({ "theme": "System" }));
    }

    #[test]
    fn recent_files_another_window_opened_are_kept_after_this_windows() {
        let entry = |p: &str| json!({ "name": p, "path": p, "pages": 1, "size": 1 });
        let base = json!({ "recent": [entry("old.pdf")], "recent_limit": 3 });
        let mine = json!({ "recent": [entry("b.pdf"), entry("old.pdf")], "recent_limit": 3 });
        let theirs = json!({ "recent": [entry("a.pdf"), entry("c.pdf"), entry("old.pdf")], "recent_limit": 3 });
        let merged = merge(&base, mine, &theirs);
        // Capped at this window's limit.
        assert_eq!(merged["recent"], json!([entry("b.pdf"), entry("old.pdf"), entry("a.pdf")]));
        // A file this window removed from the list stays removed.
        let mine = json!({ "recent": [entry("b.pdf")], "recent_limit": 3 });
        let theirs = json!({ "recent": [entry("old.pdf")], "recent_limit": 3 });
        assert_eq!(merge(&base, mine, &theirs)["recent"], json!([entry("b.pdf")]));
    }

    #[test]
    fn remembered_views_merge_by_file() {
        let view = |p: &str, page: u64| json!({ "path": p, "page": page, "zoom": "100%", "display": "single", "cover": false });
        let base = json!({ "last_views": [view("x.pdf", 0)] });
        let mine = json!({ "last_views": [view("x.pdf", 4)] });
        let theirs = json!({ "last_views": [view("y.pdf", 2), view("x.pdf", 0)] });
        assert_eq!(merge(&base, mine, &theirs)["last_views"], json!([view("x.pdf", 4), view("y.pdf", 2)]));
    }

    #[test]
    fn keys_this_version_does_not_write_are_kept() {
        let merged = merge(&json!({ "theme": "Light" }), json!({ "theme": "Dark" }), &json!({ "theme": "Light", "future": 7 }));
        assert_eq!(merged, json!({ "theme": "Dark", "future": 7 }));
    }

    /// eframe's settings file is a RON map of strings; the app's settings are one JSON string in it.
    #[test]
    fn the_settings_are_read_back_from_eframes_file() {
        let dir = std::env::temp_dir().join(format!("pdfcraft-settings-merge-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("app.ron");
        let mut map = std::collections::HashMap::new();
        map.insert("pdfcraft".to_string(), json!({ "theme": "Dark", "author": "\"quoted\" \\ ä" }).to_string());
        map.insert("egui".to_string(), "(zoom:1.0)".to_string());
        std::fs::write(&file, ron::ser::to_string_pretty(&map, ron::ser::PrettyConfig::default()).unwrap()).unwrap();
        assert_eq!(read_saved(&file), Some(json!({ "theme": "Dark", "author": "\"quoted\" \\ ä" })));
        // Half written, not RON, or without the app's key: nothing to merge with.
        std::fs::write(&file, "{\"pdfcraft\": \"{\\\"th").unwrap();
        assert_eq!(read_saved(&file), None);
        std::fs::write(&file, "{\"egui\": \"x\"}").unwrap();
        assert_eq!(read_saved(&file), None);
        assert_eq!(read_saved(&dir.join("missing.ron")), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn settings_that_are_not_objects_give_this_windows() {
        let mine = json!({ "theme": "Dark" });
        assert_eq!(merge(&json!({}), mine.clone(), &json!("garbage")), mine);
        assert_eq!(merge(&json!({}), mine.clone(), &Value::Null), mine);
        // A malformed list on disk keeps this window's.
        let mine = json!({ "recent": [{ "path": "a.pdf" }] });
        assert_eq!(merge(&json!({ "recent": [] }), mine.clone(), &json!({ "recent": "x" })), mine);
    }
}
