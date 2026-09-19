//! `rustcode-wiki` — automatic project wiki generator.
//!
//! Analyzes a project's structure (crates / packages / modules, internal
//! dependencies, languages, lines of code) and renders an OpenWiki-style wiki:
//! a `Home.md` index, an `Architecture.md` with a Mermaid dependency graph, and
//! one page per module under `Modules/`. A content-hash manifest enables
//! incremental `sync` so the wiki tracks code changes automatically.
//!
//! Inspired by OpenWiki-style LLM wikis: the structural skeleton is produced
//! deterministically (always works offline), and an optional LLM pass can enrich
//! each module page with natural-language prose.

mod diagram;
mod docgen;
mod engine;
mod error;
mod manifest;
mod model;
mod scanner;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub use engine::{resolve_langs, WikiEngine, WikiOptions, WikiResult, WikiTargetState};
pub use error::WikiError;
pub use manifest::WikiManifest;
pub use model::{FileInfo, ModuleInfo, ProjectKind, ProjectModel, WikiLang};

/// Map each module name to a unique, filesystem-safe file base name (no extension),
/// so two modules whose names collide after `safe_name` (e.g. `foo/bar` and
/// `foo-bar`) never overwrite each other, and an empty name degrades to a stable
/// `module_<n>`.
pub fn module_file_names(model: &ProjectModel) -> BTreeMap<String, String> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut map = BTreeMap::new();
    for m in &model.modules {
        let mut cand = safe_name(&m.name);
        if cand.is_empty() {
            cand = format!("module_{}", map.len());
        }
        let entry = seen.entry(cand.clone()).or_insert(0);
        *entry += 1;
        let n = *entry;
        if n > 1 {
            cand = format!("{cand}_{n}");
        }
        map.insert(m.name.clone(), cand);
    }
    map
}

/// Render a module page (with an optional LLM `summary`) to Markdown,
/// in the requested `lang`. Used by callers that wire an LLM enrichment pass
/// on top of the wiki.
pub fn module_page(
    model: &ProjectModel,
    module: &ModuleInfo,
    summary: Option<&str>,
    lang: WikiLang,
) -> String {
    let name_map = module_file_names(model);
    docgen::render_module(model, module, summary, &name_map, lang)
}

/// Build the prompt used to ask an LLM for a natural-language summary of a
/// module, written in the requested `lang`. Used by callers that wire an LLM
/// enricher on top of the deterministic wiki.
pub fn module_enrich_prompt(model: &ProjectModel, module: &ModuleInfo, lang: WikiLang) -> String {
    docgen::module_enrich_prompt(model, module, lang)
}

/// Persist an LLM-generated `summary` for a single module and rewrite its page
/// to embed the prose. The summary is written to a per-language sidecar
/// (`<lang>/Modules/<base>.summary.md`) so a later `sync`/`generate` keeps it
/// instead of reverting to the placeholder.
///
/// This is the provider-independent half of LLM enrichment: callers perform the
/// network call and hand the resulting text here, which keeps the persistence
/// logic testable without a live provider.
pub fn write_module_summary(
    out_dir: &Path,
    model: &ProjectModel,
    name_map: &BTreeMap<String, String>,
    lang: WikiLang,
    module: &ModuleInfo,
    summary: &str,
) -> Result<PathBuf, WikiError> {
    let base = name_map
        .get(&module.name)
        .cloned()
        .unwrap_or_else(|| safe_name(&module.name));
    let modules_dir = out_dir.join(lang.dir()).join("Modules");
    std::fs::create_dir_all(&modules_dir)?;
    let summary_path = modules_dir.join(format!("{base}.summary.md"));
    std::fs::write(&summary_path, summary)?;
    let content = module_page(model, module, Some(summary), lang);
    let page_path = modules_dir.join(format!("{base}.md"));
    std::fs::write(&page_path, content)?;
    Ok(summary_path)
}

/// File-name-safe identifier for a module (used for `Modules/<name>.md`).
pub fn safe_name(s: &str) -> String {
    s.replace(['/', '\\', ' '], "_")
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
