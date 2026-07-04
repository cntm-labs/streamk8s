use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SuspendedState {
    pub suspended_namespaces: HashSet<String>,
}

pub struct StateManager {
    file_path: PathBuf,
}

impl StateManager {
    pub fn new(app_data_dir: Option<PathBuf>) -> Self {
        let dir = app_data_dir.unwrap_or_else(|| PathBuf::from("."));
        let file_path = dir.join("suspended_state.json");
        Self { file_path }
    }

    pub fn load(&self) -> SuspendedState {
        if let Ok(contents) = fs::read_to_string(&self.file_path) {
            if let Ok(state) = serde_json::from_str(&contents) {
                return state;
            }
        }
        SuspendedState::default()
    }

    pub fn save(&self, state: &SuspendedState) -> Result<(), std::io::Error> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(state)?;
        fs::write(&self.file_path, contents)?;
        Ok(())
    }

    pub fn add_namespace(&self, namespace: &str) -> Result<(), std::io::Error> {
        let mut state = self.load();
        state.suspended_namespaces.insert(namespace.to_string());
        self.save(&state)
    }

    pub fn remove_namespace(&self, namespace: &str) -> Result<(), std::io::Error> {
        let mut state = self.load();
        state.suspended_namespaces.remove(namespace);
        self.save(&state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_add_and_remove_namespace() {
        let dir = tempdir().unwrap();
        let manager = StateManager::new(Some(dir.path().to_path_buf()));

        manager.add_namespace("test-ns-1").unwrap();
        let state = manager.load();
        assert!(state.suspended_namespaces.contains("test-ns-1"));

        manager.remove_namespace("test-ns-1").unwrap();
        let state = manager.load();
        assert!(!state.suspended_namespaces.contains("test-ns-1"));
    }

    #[test]
    fn test_load_non_existent() {
        let dir = tempdir().unwrap();
        let manager = StateManager::new(Some(dir.path().to_path_buf()));
        let state = manager.load();
        assert!(state.suspended_namespaces.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let manager = StateManager::new(Some(dir.path().to_path_buf()));

        let mut state = SuspendedState::default();
        state.suspended_namespaces.insert("ns1".to_string());

        manager.save(&state).unwrap();

        let loaded = manager.load();
        assert!(loaded.suspended_namespaces.contains("ns1"));
    }
}
