use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::WikiError;
use sha2::{Digest, Sha256};

/// Persisted state that lets `sync` detect source changes without regenerating
/// unchanged pages. Stored at `<wiki_out>/manifest.json`.
#[derive(Serialize, Deserialize, Default)]
pub struct WikiManifest {
    pub version: u32,
    pub generated_at: String,
    /// Identifier of the tool that produced this wiki. `rustcode-wiki` writes its
    /// own name here so a later run can tell an auto-generated wiki apart from a
    /// directory the user created by hand (which we must never overwrite).
    pub generator: String,
    /// Source file (relative path) -> sha256 content hash.
    pub source_hashes: BTreeMap<String, String>,
    /// Wiki files written by the last run (relative to the wiki output dir).
    pub wiki_files: Vec<String>,
    /// sha256 of the content we last wrote for each wiki file (relative path ->
    /// hash). Lets a later run detect when the user manually edited a generated
    /// file, so we can preserve those edits instead of silently clobbering them.
    #[serde(default)]
    pub wiki_file_hashes: BTreeMap<String, String>,
    /// Cheap, content-free snapshot of every source file: relative path ->
    /// (mtime in nanoseconds since the Unix epoch, file size in bytes). Lets a
    /// later `sync` skip the expensive full `scan()` (which reads and parses
    /// every source file) when nothing changed. Missing on manifests produced by
    /// older builds -- `sync` simply falls back to a full scan in that case.
    #[serde(default)]
    pub source_stats: BTreeMap<String, (u64, u64)>,
    /// Number of modules discovered in the project (reporting only).
    #[serde(default)]
    pub module_count: usize,
}

impl WikiManifest {
    pub fn load(path: &Path) -> Option<WikiManifest> {
        let text = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&text).ok()
    }

    pub fn save(&self, path: &Path) -> Result<(), WikiError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }
}

/// sha256 of a single file's contents.
pub fn hash_file(path: &Path) -> Result<String, WikiError> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    let digest = hasher.finalize();
    Ok(format!("{digest:x}"))
}

/// Hash a set of files (relative paths resolved against `root`).
pub fn hash_tree(root: &Path, rel_paths: &[String]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for rel in rel_paths {
        let p = root.join(rel);
        if let Ok(h) = hash_file(&p) {
            map.insert(rel.clone(), h);
        }
    }
    map
}
