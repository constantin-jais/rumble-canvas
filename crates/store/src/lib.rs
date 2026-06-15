use std::fs;
use std::path::{Path, PathBuf};

use rumble_canvas_domain::SpecWorkspace;
use rumble_canvas_handoff::ImplementationHandoff;
use rumble_canvas_package::SpecPackage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const STORE_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("JSON error at {path}: {source}")]
    Json {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("unsupported store version {found}, expected {expected}")]
    UnsupportedVersion { found: u32, expected: u32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasStore {
    pub version: u32,
    pub workspace: SpecWorkspace,
    pub packages: Vec<SpecPackage>,
    pub handoffs: Vec<ImplementationHandoff>,
    pub validation_reports: Vec<StoredReport>,
    pub dry_run_plans: Vec<StoredReport>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredReport {
    pub id: String,
    pub handoff_id: String,
    pub created_at: String,
    pub source_command: Vec<String>,
    pub payload: Value,
}

impl CanvasStore {
    pub fn new(workspace: SpecWorkspace) -> Self {
        Self {
            version: STORE_VERSION,
            workspace,
            packages: Vec::new(),
            handoffs: Vec::new(),
            validation_reports: Vec::new(),
            dry_run_plans: Vec::new(),
        }
    }

    pub fn upsert_package(&mut self, package: SpecPackage) {
        upsert_by(&mut self.packages, package, |p| &p.package_id);
    }

    pub fn upsert_handoff(&mut self, handoff: ImplementationHandoff) {
        let id = handoff_id(&handoff).unwrap_or_default();
        if let Some(existing) = self
            .handoffs
            .iter_mut()
            .find(|candidate| handoff_id(candidate).as_deref() == Some(id.as_str()))
        {
            *existing = handoff;
        } else {
            self.handoffs.push(handoff);
        }
    }

    pub fn latest_handoff(&self) -> Option<&ImplementationHandoff> {
        self.handoffs.last()
    }

    pub fn push_validation_report(&mut self, report: StoredReport) {
        self.validation_reports.push(report);
    }

    pub fn push_dry_run_plan(&mut self, report: StoredReport) {
        self.dry_run_plans.push(report);
    }
}

pub struct JsonFileStore {
    path: PathBuf,
}

impl JsonFileStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<CanvasStore, StoreError> {
        let bytes = fs::read(&self.path).map_err(|source| StoreError::Io {
            path: self.path.display().to_string(),
            source,
        })?;
        let store: CanvasStore =
            serde_json::from_slice(&bytes).map_err(|source| StoreError::Json {
                path: self.path.display().to_string(),
                source,
            })?;
        if store.version != STORE_VERSION {
            return Err(StoreError::UnsupportedVersion {
                found: store.version,
                expected: STORE_VERSION,
            });
        }
        Ok(store)
    }

    pub fn save(&self, store: &CanvasStore) -> Result<(), StoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|source| StoreError::Io {
                path: parent.display().to_string(),
                source,
            })?;
        }
        let json = serde_json::to_vec_pretty(store).map_err(|source| StoreError::Json {
            path: self.path.display().to_string(),
            source,
        })?;
        let tmp = self.path.with_extension("tmp");
        fs::write(&tmp, json).map_err(|source| StoreError::Io {
            path: tmp.display().to_string(),
            source,
        })?;
        fs::rename(&tmp, &self.path).map_err(|source| StoreError::Io {
            path: self.path.display().to_string(),
            source,
        })?;
        Ok(())
    }
}

pub fn handoff_id(handoff: &ImplementationHandoff) -> Option<String> {
    handoff
        .source
        .get("handoff_id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn upsert_by<T, F>(items: &mut Vec<T>, item: T, key: F)
where
    F: Fn(&T) -> &str,
{
    let id = key(&item).to_string();
    if let Some(existing) = items.iter_mut().find(|candidate| key(candidate) == id) {
        *existing = item;
    } else {
        items.push(item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rumble_canvas_domain::sample_workspace;
    use rumble_canvas_handoff::build_handoff;
    use rumble_canvas_package::build_package;

    #[test]
    fn saves_and_loads_store() {
        let dir = tempfile::tempdir().unwrap();
        let file = JsonFileStore::new(dir.path().join("canvas.json"));
        let mut store = CanvasStore::new(sample_workspace());
        let package = build_package(&store.workspace).unwrap();
        let handoff = build_handoff(&store.workspace, &package).unwrap();
        store.upsert_package(package);
        store.upsert_handoff(handoff);
        file.save(&store).unwrap();

        let loaded = file.load().unwrap();
        assert_eq!(loaded.version, STORE_VERSION);
        assert_eq!(loaded.packages.len(), 1);
        assert_eq!(loaded.handoffs.len(), 1);
    }
}
