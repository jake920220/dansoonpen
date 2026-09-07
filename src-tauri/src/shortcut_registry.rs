//! Shortcut changes are transactional. The tracked list always follows confirmed OS calls,
//! including partial rollback failures, so a later settings change can repair the registry.
use tauri_plugin_global_shortcut::Shortcut;
pub trait Registry {
    fn register(&mut self, key: Shortcut) -> Result<(), String>;
    fn unregister(&mut self, key: Shortcut) -> Result<(), String>;
}
fn apply(
    registry: &mut impl Registry,
    current: &mut Vec<Shortcut>,
    wanted: &[Shortcut],
) -> Result<(), String> {
    for key in wanted {
        if !current.iter().any(|k| k.id() == key.id()) {
            registry.register(*key)?;
            current.push(*key);
        }
    }
    for key in current.clone() {
        if !wanted.iter().any(|k| k.id() == key.id()) {
            registry.unregister(key)?;
            current.retain(|k| k.id() != key.id());
        }
    }
    Ok(())
}
pub fn replace(
    registry: &mut impl Registry,
    current: &mut Vec<Shortcut>,
    wanted: Vec<Shortcut>,
) -> Result<(), String> {
    let previous = current.clone();
    if let Err(error) = apply(registry, current, &wanted) {
        let rollback = apply(registry, current, &previous);
        return Err(match rollback {
            Ok(()) => format!("단축키 변경에 실패하여 이전 단축키를 유지했습니다. 다른 앱과의 충돌을 확인하세요: {error}"),
            Err(rollback) => format!("단축키 변경 및 복원에 실패했습니다. 트레이에서 입력을 복구하고 단축키를 다시 설정하세요: {error}; {rollback}"),
        });
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashSet, str::FromStr};
    #[derive(Default)]
    struct Fake {
        registered: HashSet<u32>,
        blocked: HashSet<u32>,
        fail_remove: Option<u32>,
    }
    impl Registry for Fake {
        fn register(&mut self, k: Shortcut) -> Result<(), String> {
            if self.blocked.contains(&k.id()) {
                Err("conflict".into())
            } else {
                self.registered.insert(k.id());
                Ok(())
            }
        }
        fn unregister(&mut self, k: Shortcut) -> Result<(), String> {
            if self.fail_remove == Some(k.id()) {
                self.fail_remove = None;
                Err("OS unregister error".into())
            } else {
                self.registered.remove(&k.id());
                Ok(())
            }
        }
    }
    fn key(s: &str) -> Shortcut {
        Shortcut::from_str(s).unwrap()
    }
    #[test]
    fn second_key_conflict_rolls_back_first_new_key() {
        let old = key("Alt+Shift+D");
        let new = key("Alt+Shift+A");
        let blocked = key("Alt+Shift+B");
        let mut backend = Fake::default();
        backend.register(old).unwrap();
        backend.blocked.insert(blocked.id());
        let mut current = vec![old];
        assert!(replace(&mut backend, &mut current, vec![new, blocked]).is_err());
        assert_eq!(current, vec![old]);
        assert_eq!(backend.registered, HashSet::from([old.id()]));
    }
    #[test]
    fn swapping_commands_reuses_existing_registrations() {
        let a = key("Alt+Shift+D");
        let b = key("Alt+Shift+X");
        let mut backend = Fake::default();
        backend.register(a).unwrap();
        backend.register(b).unwrap();
        let mut current = vec![a, b];
        replace(&mut backend, &mut current, vec![b, a]).unwrap();
        assert_eq!(backend.registered.len(), 2);
    }
    #[test]
    fn unregister_failure_restores_registry() {
        let a = key("Alt+Shift+D");
        let b = key("Alt+Shift+X");
        let mut backend = Fake::default();
        backend.register(a).unwrap();
        backend.fail_remove = Some(a.id());
        let mut current = vec![a];
        assert!(replace(&mut backend, &mut current, vec![b]).is_err());
        assert_eq!(current, vec![a]);
        assert_eq!(backend.registered, HashSet::from([a.id()]));
    }
}
