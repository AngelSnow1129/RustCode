use std::collections::BTreeMap;
use std::path::PathBuf;

/// The dominant technology family detected for the project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectKind {
    Rust,
    Node,
    Python,
    Go,
    Mixed,
    Unknown,
}

/// Output language for the wiki's generated prose.
///
/// `Zh` is the primary language: by default the generator emits both a Chinese
/// and an English document tree (under `zh/` and `en/` respectively), with the
/// Chinese version being the primary, human-facing copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WikiLang {
    /// Simplified Chinese (primary language).
    Zh,
    /// English.
    En,
}

impl WikiLang {
    /// Directory name for this language's document subtree.
    pub fn dir(&self) -> &'static str {
        match self {
            WikiLang::Zh => "zh",
            WikiLang::En => "en",
        }
    }

    /// Parse a language code (`zh` / `zh_cn` / `chinese` / `en` / `eng` /
    /// `english`, case-insensitive). Returns `None` for unrecognized input.
    pub fn parse(s: &str) -> Option<WikiLang> {
        match s.trim().to_ascii_lowercase().as_str() {
            "zh" | "zh_cn" | "chinese" | "中文" => Some(WikiLang::Zh),
            "en" | "eng" | "english" => Some(WikiLang::En),
            _ => None,
        }
    }
}

/// The languages emitted by default when no explicit selection is provided.
/// Chinese first, then English, so the primary copy is generated first.
pub const DEFAULT_LANGS: &[WikiLang] = &[WikiLang::Zh, WikiLang::En];

/// A single source file within a module.
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Path relative to the project root.
    pub rel_path: String,
    /// Lines of code (text files only; binaries and huge files count as 0).
    pub loc: usize,
    /// Language / extension label.
    pub lang: String,
}

/// A logical unit of the project exposed as its own wiki page.
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    /// Module name (crate name, package name, Go module leaf, or directory name).
    pub name: String,
    /// Module directory, relative to the project root.
    pub path: PathBuf,
    /// Kind label: `crate` / `package` / `module` / `package`(python) / `directory`.
    pub kind: String,
    /// Files belonging to this module.
    pub files: Vec<FileInfo>,
    /// Total lines of code across the module's files.
    pub loc: usize,
    /// Names of modules this one depends on (resolved to in-repo modules).
    pub depends_on: Vec<String>,
}

/// The full structural model of a scanned project.
#[derive(Debug, Clone)]
pub struct ProjectModel {
    pub root: PathBuf,
    pub name: String,
    pub kind: ProjectKind,
    pub modules: Vec<ModuleInfo>,
    pub file_count: usize,
    pub loc: usize,
    /// Language label -> number of files.
    pub languages: BTreeMap<String, usize>,
}
