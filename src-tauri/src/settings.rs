use crate::model::AppSettings;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
pub fn load(path: &Path) -> Result<AppSettings, String> {
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let file = File::open(path).map_err(|e| format!("설정을 열지 못했습니다: {e}"))?;
    if file.metadata().map_err(|e| e.to_string())?.len() > 64 * 1024 {
        return Err("설정 파일이 너무 큽니다.".into());
    }
    let value: serde_json::Value =
        serde_json::from_reader(file).map_err(|e| format!("설정 파일을 읽지 못했습니다: {e}"))?;
    let adding_visibility = value.get("visibilityShortcut").is_none();
    let mut settings: AppSettings =
        serde_json::from_value(value).map_err(|e| format!("설정 파일을 읽지 못했습니다: {e}"))?;
    if adding_visibility {
        use std::str::FromStr;
        use tauri_plugin_global_shortcut::Shortcut;
        if let Ok(new) = Shortcut::from_str(&settings.visibility_shortcut) {
            if [&settings.toggle_shortcut, &settings.clear_shortcut]
                .iter()
                .any(|s| Shortcut::from_str(s).is_ok_and(|s| s.id() == new.id()))
            {
                settings.visibility_shortcut.clear();
            }
        }
    }
    if settings.version == 1 {
        // Upgrade the previous defaults, retaining individually customized settings.
        if settings.width == 5. {
            settings.width = 12.;
        }
        if settings.toggle_shortcut == "Alt+Shift+D" && settings.clear_shortcut == "Alt+Shift+X" {
            let defaults = AppSettings::default();
            settings.toggle_shortcut = defaults.toggle_shortcut;
            settings.clear_shortcut = defaults.clear_shortcut;
        }
        settings.version = 2;
    }
    if settings.version == 2 {
        settings.version = 3;
    }
    settings.validate()?;
    Ok(settings)
}
pub fn save(path: &Path, settings: &AppSettings) -> Result<(), String> {
    settings.validate()?;
    let parent = path.parent().ok_or("설정 경로가 올바르지 않습니다.")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("tmp");
    let result = (|| {
        let mut f = File::create(&tmp)?;
        f.write_all(&serde_json::to_vec_pretty(settings)?)?;
        f.sync_all()?;
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::Storage::FileSystem::{
                MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
            };
            let from: Vec<u16> = tmp.as_os_str().encode_wide().chain(Some(0)).collect();
            let to: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
            // Close the flushed temporary file before Windows replaces the destination.
            drop(f);
            if unsafe {
                MoveFileExW(
                    from.as_ptr(),
                    to.as_ptr(),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            } == 0
            {
                return Err(std::io::Error::last_os_error());
            }
        }
        #[cfg(not(target_os = "windows"))]
        fs::rename(&tmp, path)?;
        Ok::<_, std::io::Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(tmp);
    }
    result.map_err(|e| format!("설정 저장에 실패했습니다: {e}"))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn visibility_migration_preserves_an_existing_key_assignment() {
        let path =
            std::env::temp_dir().join(format!("my-brush-visibility-{}.json", std::process::id()));
        let mut json = serde_json::to_value(AppSettings::default()).unwrap();
        json["toggleShortcut"] = json["visibilityShortcut"].clone();
        json.as_object_mut().unwrap().remove("visibilityShortcut");
        std::fs::write(&path, serde_json::to_vec(&json).unwrap()).unwrap();
        let loaded = load(&path).unwrap();
        assert!(loaded.visibility_shortcut.is_empty());
        assert_eq!(
            loaded.toggle_shortcut,
            AppSettings::default().visibility_shortcut
        );
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn old_defaults_upgrade_without_discarding_custom_colors_or_keys() {
        let path =
            std::env::temp_dir().join(format!("my-brush-migration-{}.json", std::process::id()));
        let mut settings = AppSettings {
            version: 1,
            width: 5.,
            toggle_shortcut: "Alt+Shift+D".into(),
            clear_shortcut: "Alt+Shift+X".into(),
            ..AppSettings::default()
        };
        std::fs::write(&path, serde_json::to_vec(&settings).unwrap()).unwrap();
        assert_eq!(load(&path).unwrap(), AppSettings::default());
        settings.width = 19.;
        settings.color = "#123456".into();
        settings.toggle_shortcut = "Command+Shift+D".into();
        std::fs::write(&path, serde_json::to_vec(&settings).unwrap()).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded.version, 3);
        assert_eq!(loaded.width, 19.);
        assert_eq!(loaded.color, "#123456");
        assert_eq!(loaded.toggle_shortcut, settings.toggle_shortcut);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn independent_window_patches_preserve_the_latest_fields() {
        use crate::model::SettingsPatch;
        let first = SettingsPatch {
            color: Some("#57d9c6".into()),
            ..Default::default()
        }
        .apply(&AppSettings::default());
        let second = SettingsPatch {
            toggle_shortcut: Some("Control+Shift+D".into()),
            ..Default::default()
        }
        .apply(&first);
        let latest = SettingsPatch {
            width: Some(20.),
            ..Default::default()
        }
        .apply(&second);
        assert_eq!(latest.color, "#57d9c6");
        assert_eq!(latest.toggle_shortcut, "Control+Shift+D");
        assert_eq!(latest.width, 20.);
        latest.validate().unwrap();
    }
    #[test]
    fn v2_settings_gain_presets_without_changing_saved_defaults() {
        let path = std::env::temp_dir().join(format!("my-brush-v2-{}.json", std::process::id()));
        let mut json = serde_json::to_value(AppSettings::default()).unwrap();
        json["version"] = 2.into();
        json["width"] = 17.into();
        json["color"] = "#123456".into();
        json.as_object_mut().unwrap().remove("presets");
        std::fs::write(&path, serde_json::to_vec(&json).unwrap()).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded.version, 3);
        assert_eq!(loaded.width, 17.);
        assert_eq!(loaded.color, "#123456");
        assert_eq!(loaded.presets.len(), 3);
        save(&path, &loaded).unwrap();
        assert_eq!(load(&path).unwrap(), loaded);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn current_brush_changes_never_change_defaults_and_explicit_default_edits_apply() {
        use crate::model::{BrushPatch, BrushSettings, SettingsPatch, Tool};
        let defaults = AppSettings::default();
        let brush = BrushPatch {
            color: Some("#123456".into()),
            width: Some(4.),
            tool: Some(Tool::Text),
            ..Default::default()
        }
        .apply(&BrushSettings::from_defaults(&defaults));
        assert_eq!(defaults.width, 12.);
        assert_eq!(defaults.color, "#ffcf56");
        let patch = SettingsPatch {
            width: Some(18.),
            ..Default::default()
        };
        let brush = patch.brush_patch().apply(&brush);
        assert_eq!(brush.width, 18.);
        assert_eq!(brush.color, "#123456");
        assert_eq!(brush.tool, Tool::Text);
        let persisted = patch.apply(&defaults);
        assert_eq!(BrushSettings::from_defaults(&persisted).color, "#ffcf56");
    }
    #[test]
    fn preset_edits_preserve_other_slots_defaults_and_reject_invalid_values() {
        let settings = AppSettings::default();
        let modified = settings
            .replace_preset(
                0,
                Some("  강의용  ".into()),
                Some(settings.presets[1].brush.clone()),
            )
            .unwrap();
        assert_eq!(modified.presets[0].name, "강의용");
        assert_eq!(modified.presets[0].brush.width, 4.);
        assert_eq!(modified.presets[1..], settings.presets[1..]);
        assert_eq!(modified.width, 12.);
        assert!(settings.replace_preset(3, None, None).is_err());
        assert!(settings.replace_preset(0, Some(" ".into()), None).is_err());
        let mut invalid = settings.presets[0].brush.clone();
        invalid.width = f64::NAN;
        assert!(settings.replace_preset(0, None, Some(invalid)).is_err());
    }
    #[test]
    fn persistence_round_trip_and_corrupt_rejection() {
        let path =
            std::env::temp_dir().join(format!("my-brush-settings-{}.json", std::process::id()));
        let settings = AppSettings {
            width: 12.,
            ..AppSettings::default()
        };
        save(&path, &settings).unwrap();
        assert_eq!(load(&path).unwrap(), settings);
        std::fs::write(&path, b"invalid").unwrap();
        assert!(load(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }
}
