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
    let mut settings: AppSettings =
        serde_json::from_reader(file).map_err(|e| format!("설정 파일을 읽지 못했습니다: {e}"))?;
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
        assert_eq!(loaded.version, 2);
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
