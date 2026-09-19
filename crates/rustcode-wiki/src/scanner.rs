use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use crate::error::WikiError;
use crate::model::*;

const DEFAULT_EXCLUDES: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "dist",
    "build",
    "vendor",
    ".rustcode",
    "out",
    "bin",
    "obj",
    "__pycache__",
    ".venv",
    "venv",
    ".idea",
    ".vscode",
    ".next",
    ".turbo",
    "coverage",
    ".cache",
    "tmp",
    "temp",
];

/// Options controlling how a project is scanned.
pub struct ScanOptions {
    pub max_files: usize,
    pub exclude_dirs: Vec<String>,
}

/// Walk `root` and build a [`ProjectModel`].
///
/// gitignore-aware (via the `ignore` crate) and skips a built-in set of heavy /
/// generated directories. Recognized project shapes: Cargo workspace (crates),
/// npm/pnpm/yarn (packages), Go modules, and Python packages (`__init__.py`).
pub fn scan(root: &Path, opts: &ScanOptions) -> Result<ProjectModel, WikiError> {
    // --- Collect files (gitignore-aware) -------------------------------------
    let files = collect_source_files(root, opts);

    // --- Discover modules from manifests -------------------------------------
    let mut modules: Vec<ModuleInfo> = Vec::new();
    let mut module_roots: Vec<(PathBuf, String, usize)> = Vec::new();
    let mut project_name: Option<String> = None;
    let mut kinds: HashSet<&'static str> = HashSet::new();
    let mut has_python_pkg = false;

    for f in &files {
        let name = f.file_name().and_then(|s| s.to_str()).unwrap_or("");
        match name {
            "Cargo.toml" => {
                if let Ok(text) = std::fs::read_to_string(f) {
                    if let Ok(v) = text.parse::<toml::Value>() {
                        if let Some(pkg) = v
                            .get("package")
                            .and_then(|p| p.get("name"))
                            .and_then(|n| n.as_str())
                        {
                            let dir = f.parent().unwrap_or(root).to_path_buf();
                            modules.push(ModuleInfo {
                                name: pkg.to_string(),
                                path: rel(root, &dir),
                                kind: "crate".to_string(),
                                files: Vec::new(),
                                loc: 0,
                                depends_on: collect_cargo_deps(&v),
                            });
                            module_roots.push((dir, pkg.to_string(), modules.len() - 1));
                            if project_name.is_none() {
                                project_name = Some(pkg.to_string());
                            }
                            kinds.insert("Rust");
                        }
                    }
                }
            }
            "package.json" => {
                if let Ok(text) = std::fs::read_to_string(f) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(pkg) = v.get("name").and_then(|n| n.as_str()) {
                            let dir = f.parent().unwrap_or(root).to_path_buf();
                            modules.push(ModuleInfo {
                                name: pkg.to_string(),
                                path: rel(root, &dir),
                                kind: "package".to_string(),
                                files: Vec::new(),
                                loc: 0,
                                depends_on: collect_npm_deps(&v),
                            });
                            module_roots.push((dir, pkg.to_string(), modules.len() - 1));
                            if project_name.is_none() {
                                project_name = Some(pkg.to_string());
                            }
                            kinds.insert("Node");
                        }
                    }
                }
            }
            "go.mod" => {
                if let Ok(text) = std::fs::read_to_string(f) {
                    if let Some(modpath) = parse_go_module(&text) {
                        let dir = f.parent().unwrap_or(root).to_path_buf();
                        let nm = modpath.rsplit('/').next().unwrap_or(&modpath).to_string();
                        modules.push(ModuleInfo {
                            name: nm.clone(),
                            path: rel(root, &dir),
                            kind: "module".to_string(),
                            files: Vec::new(),
                            loc: 0,
                            depends_on: parse_go_requires(&text),
                        });
                        module_roots.push((dir, nm, modules.len() - 1));
                        if project_name.is_none() {
                            project_name = Some(modpath);
                        }
                        kinds.insert("Go");
                    }
                }
            }
            "__init__.py" => {
                has_python_pkg = true;
                let dir = f.parent().unwrap_or(root).to_path_buf();
                let nm = dir
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("package")
                    .to_string();
                modules.push(ModuleInfo {
                    name: nm.clone(),
                    path: rel(root, &dir),
                    kind: "package".to_string(),
                    files: Vec::new(),
                    loc: 0,
                    depends_on: Vec::new(),
                });
                module_roots.push((dir, nm, modules.len() - 1));
                kinds.insert("Python");
            }
            _ => {}
        }
    }

    if has_python_pkg && project_name.is_none() {
        project_name = root
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());
    }

    // Sort module roots by depth descending so the deepest ancestor wins.
    module_roots.sort_by_key(|(dir, _, _)| std::cmp::Reverse(dir.components().count()));

    // --- Assign files to modules + tally LOC/languages ----------------------
    let mut languages: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_loc = 0usize;
    let mut assigned: Vec<Vec<FileInfo>> = vec![Vec::new(); modules.len()];
    let mut any_assigned = false;

    for f in &files {
        let ext = lang_ext(f);
        let loc = count_loc(f);
        *languages
            .entry(language_name(&ext).to_string())
            .or_insert(0) += 1;
        total_loc += loc;
        let relf = rel(root, f);
        let mut mod_idx = None;
        for (dir, _, mi) in module_roots.iter() {
            if f.starts_with(dir) {
                mod_idx = Some(*mi);
                break; // deepest match due to sort order
            }
        }
        if let Some(mi) = mod_idx {
            assigned[mi].push(FileInfo {
                rel_path: relf.to_string_lossy().into_owned(),
                loc,
                lang: ext,
            });
            any_assigned = true;
        }
    }

    if !any_assigned && !files.is_empty() {
        let nm = project_name.clone().unwrap_or_else(|| {
            root.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("project")
                .to_string()
        });
        modules.push(ModuleInfo {
            name: nm.clone(),
            path: PathBuf::from("."),
            kind: "directory".to_string(),
            files: Vec::new(),
            loc: 0,
            depends_on: Vec::new(),
        });
        let mut all = Vec::new();
        for f in &files {
            let ext = lang_ext(f);
            let loc = count_loc(f);
            all.push(FileInfo {
                rel_path: rel(root, f).to_string_lossy().into_owned(),
                loc,
                lang: ext,
            });
        }
        assigned.push(all);
    }

    // Move assigned files into modules + compute per-module loc.
    for (i, m) in modules.iter_mut().enumerate() {
        let mut fs = std::mem::take(&mut assigned[i]);
        fs.sort_by(|a, b| b.loc.cmp(&a.loc).then_with(|| a.rel_path.cmp(&b.rel_path)));
        m.loc = fs.iter().map(|f| f.loc).sum();
        m.files = fs;
    }

    // Resolve dependency edges by module name.
    let names: HashSet<String> = modules.iter().map(|m| m.name.clone()).collect();
    for m in &mut modules {
        m.depends_on.retain(|d| names.contains(d));
        m.depends_on.sort();
        m.depends_on.dedup();
    }

    let kind = if kinds.len() > 1 {
        ProjectKind::Mixed
    } else if kinds.contains("Rust") {
        ProjectKind::Rust
    } else if kinds.contains("Node") {
        ProjectKind::Node
    } else if kinds.contains("Python") {
        ProjectKind::Python
    } else if kinds.contains("Go") {
        ProjectKind::Go
    } else {
        ProjectKind::Unknown
    };

    let name = project_name.unwrap_or_else(|| {
        root.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project")
            .to_string()
    });

    // Deterministic output: sort modules by name and files by (loc desc, path) so
    // that `Home.md` / `Architecture.md` render identically across runs, keeping
    // `sync` genuinely idempotent.
    modules.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(ProjectModel {
        root: root.to_path_buf(),
        name,
        kind,
        modules,
        file_count: files.len(),
        loc: total_loc,
        languages,
    })
}

/// Enumerate every source file under `root` the same way [`scan`] does (same
/// gitignore handling, exclude list, symlink skipping and `max_files` cap), but
/// without reading or parsing any of their contents.
fn collect_source_files(root: &Path, opts: &ScanOptions) -> Vec<PathBuf> {
    let mut excludes: HashSet<String> = DEFAULT_EXCLUDES.iter().map(|s| (*s).to_string()).collect();
    for e in &opts.exclude_dirs {
        excludes.insert(e.trim_matches('/').to_string());
    }

    let mut files: Vec<PathBuf> = Vec::new();
    let root_buf = root.to_path_buf();
    let mut walker = ignore::WalkBuilder::new(root);
    walker
        .hidden(false)
        .parents(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(true)
        .require_git(false);
    walker.filter_entry(move |e| {
        if e.path() == root_buf {
            return true;
        }
        if e.file_type().map(|f| f.is_dir()).unwrap_or(false) {
            if let Some(name) = e.file_name().to_str() {
                if excludes.contains(name) {
                    return false;
                }
            }
        }
        true
    });
    for entry in walker.build() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        // Skip symlinks so we never read/follow paths outside the project root.
        if entry.path_is_symlink() {
            continue;
        }
        if entry.file_type().map(|f| f.is_file()).unwrap_or(false) {
            files.push(entry.path().to_path_buf());
            if files.len() >= opts.max_files {
                break;
            }
        }
    }
    files
}

/// Cheap, content-free snapshot of the project's source files.
///
/// Returns a map of relative path -> `(mtime_nanos, size_bytes)` for every file
/// [`scan`] would consider. Unlike [`scan`] this never reads file contents, so
/// it is dramatically cheaper on large repositories. `sync` uses it to detect
/// whether anything changed since the last run without paying for a full
/// re-scan; a mismatch (added / removed / modified / truncated file) falls
/// through to the full scan, which remains the authoritative change check.
pub fn scan_stats(root: &Path, opts: &ScanOptions) -> BTreeMap<String, (u64, u64)> {
    use std::time::UNIX_EPOCH;
    let mut map = BTreeMap::new();
    for f in collect_source_files(root, opts) {
        if let Ok(meta) = std::fs::metadata(&f) {
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            map.insert(
                rel(root, &f).to_string_lossy().into_owned(),
                (mtime, meta.len()),
            );
        }
    }
    map
}

fn rel(root: &Path, p: &Path) -> PathBuf {
    p.strip_prefix(root).unwrap_or(p).to_path_buf()
}

fn lang_ext(p: &Path) -> String {
    p.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_else(|| "unknown".to_string())
}

fn language_name(ext: &str) -> &'static str {
    match ext {
        "rs" => "Rust",
        "py" => "Python",
        "js" => "JavaScript",
        "jsx" => "JavaScript",
        "ts" => "TypeScript",
        "tsx" => "TypeScript",
        "go" => "Go",
        "java" => "Java",
        "c" | "h" => "C",
        "cpp" | "cc" | "cxx" | "hpp" => "C++",
        "rb" => "Ruby",
        "php" => "PHP",
        "cs" => "C#",
        "swift" => "Swift",
        "kt" => "Kotlin",
        "scala" => "Scala",
        "sh" | "bash" => "Shell",
        "html" => "HTML",
        "css" => "CSS",
        "scss" => "SCSS",
        "json" => "JSON",
        "toml" => "TOML",
        "yaml" | "yml" => "YAML",
        "md" => "Markdown",
        "sql" => "SQL",
        "unknown" => "Other",
        _ => "Other",
    }
}

fn count_loc(path: &Path) -> usize {
    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return 0,
    };
    if meta.len() > 4 * 1024 * 1024 {
        return 0;
    }
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return 0,
    };
    if bytes.contains(&0u8) {
        return 0; // binary
    }
    let n = bytes.iter().filter(|&&b| b == b'\n').count();
    if bytes.is_empty() {
        0
    } else {
        n + 1
    }
}

fn collect_cargo_deps(v: &toml::Value) -> Vec<String> {
    let mut deps = Vec::new();
    for key in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = v.get(key).and_then(|t| t.as_table()) {
            for k in table.keys() {
                deps.push(k.clone());
            }
        }
    }
    deps
}

fn collect_npm_deps(v: &serde_json::Value) -> Vec<String> {
    let mut deps = Vec::new();
    for key in ["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(obj) = v.get(key).and_then(|t| t.as_object()) {
            for k in obj.keys() {
                deps.push(k.clone());
            }
        }
    }
    deps
}

fn parse_go_module(text: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("module ") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

fn parse_go_requires(text: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut in_block = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("require (") {
            in_block = true;
            continue;
        }
        if in_block {
            if t == ")" {
                in_block = false;
                continue;
            }
            if let Some(dep) = t.split_whitespace().next() {
                if !dep.is_empty() {
                    deps.push(dep.to_string());
                }
            }
        } else if let Some(rest) = t.strip_prefix("require ") {
            if let Some(dep) = rest.split_whitespace().next() {
                deps.push(dep.to_string());
            }
        }
    }
    deps
}
