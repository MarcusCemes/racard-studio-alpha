use std::{fmt, fs, io, path::Path};

use algorithm::{ProblemConfig, RefinementParameters, Solution, SolverParameters, Weights};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

/* === ProjectFile === */

/// Top-level project format. Serialized with MessagePack (.rcs).
///
/// The `version` field is incremented when the schema changes in a
/// backward-incompatible way, giving us a migration path.
///
/// `config` holds the compiled problem (with bank holidays baked into
/// overrides).  `raw` preserves the editable UI settings so loading a
/// project round-trips through the frontend without losing metadata.
#[derive(Serialize, Deserialize)]
pub struct ProjectFile {
    pub version: u32,
    pub config: ProblemConfig,
    pub raw: RawSettings,
    pub solution: Solution,
    pub solver: SolverParameters,
    pub refiner: RefinementParameters,
    pub weights: Weights,
    pub top_k: u32,
    pub checkpoints: Vec<CheckpointRecord>,
}

/// UI-editable settings that `ProblemConfig` does not capture.
#[derive(Serialize, Deserialize)]
pub struct RawSettings {
    /// Per-bank-holiday configuration (date, name, enabled, optional overrides).
    pub bank_holidays: Vec<RawBankHoliday>,
    /// Default bank holiday hours per weekday index (0=Mon … 6=Sun).
    pub bank_holiday_default_hours: Vec<[f32; 2]>,
    /// Manual date/role/hour overrides.
    pub custom_overrides: Vec<RawCustomOverride>,
}

#[derive(Serialize, Deserialize)]
pub struct RawBankHoliday {
    pub date: NaiveDate,
    pub name: String,
    pub enabled: bool,
    pub lead_hours: Option<f32>,
    pub support_hours: Option<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct RawCustomOverride {
    pub date: NaiveDate,
    pub role: String, // "Lead" | "Support"
    pub hours: f32,
}

#[derive(Serialize, Deserialize)]
pub struct CheckpointRecord {
    pub name: String,
    pub slots: Solution,
    pub timestamp: u64,
}

/* === Error === */

#[derive(Debug)]
pub enum ProjectError {
    Io(io::Error),
    Format(String),
}

impl From<io::Error> for ProjectError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<rmp_serde::encode::Error> for ProjectError {
    fn from(e: rmp_serde::encode::Error) -> Self {
        Self::Format(e.to_string())
    }
}

impl From<rmp_serde::decode::Error> for ProjectError {
    fn from(e: rmp_serde::decode::Error) -> Self {
        Self::Format(e.to_string())
    }
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Format(e) => write!(f, "Format error: {e}"),
        }
    }
}

impl Serialize for ProjectError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

/* === I/O === */

pub fn save_to_disk(path: &Path, project: &ProjectFile) -> Result<(), ProjectError> {
    let bytes = rmp_serde::to_vec(project)?;
    fs::write(path, &bytes)?;
    Ok(())
}

pub fn load_from_disk(path: &Path) -> Result<ProjectFile, ProjectError> {
    let bytes = fs::read(path)?;
    let project: ProjectFile = rmp_serde::from_slice(&bytes)?;
    Ok(project)
}

/* === Tauri commands === */

#[tauri::command]
pub async fn save_project(
    _app: AppHandle,
    path: String,
    project: ProjectFile,
) -> Result<(), ProjectError> {
    save_to_disk(Path::new(&path), &project)
}

#[tauri::command]
pub async fn load_project(_app: AppHandle, path: String) -> Result<ProjectFile, ProjectError> {
    load_from_disk(Path::new(&path))
}
