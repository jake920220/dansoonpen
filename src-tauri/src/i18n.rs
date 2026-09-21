use crate::model::Language;
use std::{collections::HashMap, sync::OnceLock};

// Share the small static catalog with the web UI; no network or OS locale polling.
pub fn tr(language: Language, key: &str) -> String {
    if language == Language::Ko {
        return key.into();
    }
    static ENGLISH: OnceLock<HashMap<String, String>> = OnceLock::new();
    ENGLISH
        .get_or_init(|| {
            serde_json::from_str(include_str!("../../src/locales/en.json"))
                .expect("valid bundled translations")
        })
        .get(key)
        .map_or_else(|| key.into(), Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn menu_labels_have_english_and_korean_variants() {
        for label in [
            "설정 열기",
            "그리기",
            "앱 조작으로 복귀 (입력 복구)",
            "모든 화면 필기 지우기",
            "커서가 있는 화면 필기 지우기",
            "방금 지운 필기 되돌리기",
            "My Brush 종료",
            "설정…",
            "My Brush 필기",
            "필기 잠시 숨기기",
            "숨긴 필기 다시 표시",
            "커서 강조 켜기",
            "커서 강조 끄기",
        ] {
            assert_eq!(tr(Language::Ko, label), label);
            assert_ne!(tr(Language::En, label), label);
        }
    }
}
