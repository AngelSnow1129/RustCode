//! Integration tests for `rustcode-wiki`.
//!
//! All tests exercise only the public API (`WikiEngine`, `WikiOptions`,
//! `module_page`, `module_enrich_prompt`, `safe_name`). Temporary projects are
//! created under `std::env::temp_dir()` with a unique, per-run path and removed
//! on drop so the repository is never polluted.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use rustcode_wiki::{
    module_enrich_prompt, module_file_names, module_page, safe_name, write_module_summary,
    FileInfo, ModuleInfo, ProjectKind, ProjectModel, WikiEngine, WikiLang, WikiManifest,
    WikiOptions,
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A temporary project root under `env::temp_dir()`. Removed when dropped.
struct TempProject {
    root: std::path::PathBuf,
}

impl TempProject {
    fn new(tag: &str) -> Self {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut root = std::env::temp_dir();
        root.push(format!(
            "rustcode-wiki-test/{}-{}-{}",
            tag,
            std::process::id(),
            n
        ));
        // Clean any leftover from a crashed previous run with the same path.
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp project root");
        TempProject { root }
    }

    fn path(&self, rel: &str) -> std::path::PathBuf {
        self.root.join(rel)
    }

    fn write(&self, rel: &str, content: &str) {
        let p = self.path(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).expect("create parent dir");
        }
        std::fs::write(&p, content).expect("write fixture file");
    }

    fn remove(&self, rel: &str) {
        let p = self.path(rel);
        if p.is_dir() {
            let _ = std::fs::remove_dir_all(&p);
        } else {
            let _ = std::fs::remove_file(&p);
        }
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.path(rel)).expect("read generated file")
    }

    fn exists(&self, rel: &str) -> bool {
        self.path(rel).exists()
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn opts_for(root: &Path) -> WikiOptions {
    WikiOptions {
        root: root.to_path_buf(),
        out_dir: None,
        title: None,
        force: false,
        max_files: 10_000,
        exclude_dirs: Vec::new(),
        langs: Vec::new(),
        assume_yes: true,
    }
}

/// A Rust crate at the root plus an npm package in a subdirectory => >= 2 modules.
fn write_sample_project(p: &TempProject) {
    p.write(
        "Cargo.toml",
        "[package]\nname = \"mycrate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
    );
    p.write(
        "src/lib.rs",
        "// mycrate root module\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    );
    p.write(
        "web/package.json",
        "{\n  \"name\": \"mypkg\",\n  \"version\": \"1.0.0\"\n}\n",
    );
    p.write("web/index.js", "// mypkg entry\nconsole.log('hi');\n");
}

#[test]
fn test_generate_creates_wiki_files_and_default_out_dir() {
    let proj = TempProject::new("generate");
    write_sample_project(&proj);

    let opts = opts_for(&proj.root);
    let result = WikiEngine::generate(&opts).expect("generate");

    // Default output directory is `<root>/.rustcode/wiki`.
    assert_eq!(result.out_dir, proj.root.join(".rustcode").join("wiki"));

    // Two manifests => at least two modules.
    assert!(
        result.modules >= 2,
        "expected >=2 modules, got {}",
        result.modules
    );

    let expected = [
        "Home.md",
        "Architecture.md",
        "README.md",
        "Modules/mycrate.md",
        "Modules/mypkg.md",
    ];
    for lang in ["zh", "en"] {
        for f in expected {
            let p = format!(".rustcode/wiki/{lang}/{f}");
            assert!(proj.exists(&p), "expected wiki file {p} to exist");
        }
    }

    // `render_mermaid` is private; validate the Mermaid shape indirectly via
    // the generated Architecture.md (Chinese copy is the primary one).
    let arch = proj.read(".rustcode/wiki/zh/Architecture.md");
    assert!(
        arch.contains("```mermaid"),
        "Architecture.md should contain a mermaid fenced block"
    );
    assert!(
        arch.contains("graph TD"),
        "mermaid diagram should be `graph TD`"
    );
    assert!(
        arch.contains("mycrate"),
        "diagram should mention the mycrate module"
    );
    assert!(
        arch.contains("mypkg"),
        "diagram should mention the mypkg module"
    );
}

#[test]
fn test_sync_is_idempotent() {
    let proj = TempProject::new("sync");
    write_sample_project(&proj);

    WikiEngine::generate(&opts_for(&proj.root)).expect("generate");

    let result = WikiEngine::sync(&opts_for(&proj.root)).expect("sync");
    assert!(
        result.unchanged > 0,
        "sync on an unchanged project should report unchanged files"
    );
    assert!(
        result.created.is_empty(),
        "idempotent sync should create nothing, got {:?}",
        result.created
    );
    assert!(
        result.updated.is_empty(),
        "idempotent sync should update nothing, got {:?}",
        result.updated
    );
}

#[test]
fn test_sync_detects_modification() {
    let proj = TempProject::new("modify");
    write_sample_project(&proj);

    WikiEngine::generate(&opts_for(&proj.root)).expect("generate");
    let manifest_before = proj.read(".rustcode/wiki/manifest.json");

    // Modify a source file: change its line count so the rendered module page
    // (which shows LOC per file) actually differs.
    proj.write(
        "src/lib.rs",
        "// mycrate root module (updated)\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\npub fn sub(a: i32, b: i32) -> i32 {\n    a - b\n}\n",
    );

    let result = WikiEngine::sync(&opts_for(&proj.root)).expect("sync");

    let mycrate_page = proj.root.join(".rustcode/wiki/zh/Modules/mycrate.md");
    let mypkg_page = proj.root.join(".rustcode/wiki/zh/Modules/mypkg.md");

    assert!(
        result.updated.contains(&mycrate_page),
        "changed module page should be in `updated`, got {:?}",
        result.updated
    );
    assert!(
        !result.updated.contains(&mypkg_page),
        "unchanged module page should NOT be in `updated`"
    );

    // The manifest must be rewritten to reflect the new source hash.
    let manifest_after = proj.read(".rustcode/wiki/manifest.json");
    assert_ne!(
        manifest_before, manifest_after,
        "manifest should be updated after a source change"
    );
    assert!(
        manifest_after.contains("src/lib.rs"),
        "manifest should still track src/lib.rs"
    );
}

#[test]
fn test_safe_name() {
    // Deterministic, ASCII-relevant transformations.
    let exact: &[(&str, &str)] = &[
        ("a/b", "a_b"),
        ("a\\b", "a_b"),
        ("my module", "my_module"),
        ("..", "__"),
        (".", "_"),
        ("foo@bar!", "foo_bar_"),
        ("crate::name", "crate__name"),
    ];
    for (input, expected) in exact {
        let got = safe_name(input);
        assert_eq!(&got, expected, "safe_name({input:?})");
    }

    // Core safety contract: the result must never contain a path separator,
    // regardless of input (CJK, traversal attempts, glob chars, ...).
    let no_sep: &[&str] = &[
        "a/b\\c",
        "../x",
        "模块",
        "a/../b",
        "weird:name?*",
        "  leading/trailing  ",
    ];
    for input in no_sep {
        let got = safe_name(input);
        assert!(
            !got.contains('/'),
            "safe_name({input:?}) must not contain '/': {got:?}"
        );
        assert!(
            !got.contains('\\'),
            "safe_name({input:?}) must not contain '\\\\': {got:?}"
        );
    }
}

fn dummy_model() -> ProjectModel {
    ProjectModel {
        root: Path::new("/tmp/dummy").to_path_buf(),
        name: "dummy".to_string(),
        kind: ProjectKind::Rust,
        modules: Vec::new(),
        file_count: 0,
        loc: 0,
        languages: Default::default(),
    }
}

#[test]
fn test_module_page_summary_vs_placeholder() {
    let model = dummy_model();
    let module = ModuleInfo {
        name: "auth".to_string(),
        path: Path::new("src/auth").to_path_buf(),
        kind: "crate".to_string(),
        files: vec![FileInfo {
            rel_path: "src/auth/mod.rs".to_string(),
            loc: 42,
            lang: "rust".to_string(),
        }],
        loc: 42,
        depends_on: vec![],
    };

    let without = module_page(&model, &module, None, WikiLang::Zh);
    let with = module_page(
        &model,
        &module,
        Some("Auth handles login and sessions."),
        WikiLang::Zh,
    );

    assert!(
        without.contains("自然语言摘要"),
        "placeholder should be present (in Chinese) when no summary is supplied"
    );
    assert!(
        with.contains("Auth handles login and sessions."),
        "supplied summary text should appear in the page"
    );
    assert!(
        !with.contains("自然语言摘要"),
        "a supplied summary should replace the placeholder"
    );
    assert_ne!(
        without, with,
        "summary vs no-summary renderings must differ"
    );
}

#[test]
fn test_module_enrich_prompt_contains_name() {
    let model = dummy_model();
    let module = ModuleInfo {
        name: "payments".to_string(),
        path: Path::new("crates/payments").to_path_buf(),
        kind: "crate".to_string(),
        files: vec![FileInfo {
            rel_path: "crates/payments/src/lib.rs".to_string(),
            loc: 10,
            lang: "rust".to_string(),
        }],
        loc: 10,
        depends_on: vec![],
    };
    let prompt = module_enrich_prompt(&model, &module, WikiLang::En);
    assert!(
        prompt.contains("payments"),
        "enrich prompt should mention the module name"
    );
    assert!(
        prompt.contains("Module:"),
        "enrich prompt should include a `Module:` line"
    );
}

#[test]
fn test_sync_removes_deleted_module() {
    let proj = TempProject::new("remove");
    write_sample_project(&proj);

    WikiEngine::generate(&opts_for(&proj.root)).expect("generate");
    assert!(
        proj.exists(".rustcode/wiki/zh/Modules/mypkg.md"),
        "mypkg page should exist before removal"
    );

    // Delete the npm package module entirely (manifest + sources).
    proj.remove("web/package.json");
    proj.remove("web/index.js");
    proj.remove("web");

    let result = WikiEngine::sync(&opts_for(&proj.root)).expect("sync");

    let mypkg_page = proj.root.join(".rustcode/wiki/zh/Modules/mypkg.md");
    assert!(
        result.removed.contains(&mypkg_page),
        "removed module page should appear in `removed`, got {:?}",
        result.removed
    );
    assert!(
        !proj.exists(".rustcode/wiki/zh/Modules/mypkg.md"),
        "removed module page file should be deleted from disk"
    );
    assert!(
        proj.exists(".rustcode/wiki/zh/Modules/mycrate.md"),
        "the remaining module page should still exist"
    );
}

/// GAP ②: deleting a module must also delete its LLM summary sidecar(s)
/// (`<lang>/Modules/<base>.summary.md`). Left behind, a stale sidecar would be
/// silently re-embedded should a same-named module reappear later.
#[test]
fn test_sync_removes_deleted_module_summary_sidecar() {
    let proj = TempProject::new("removesidecar");
    write_sample_project(&proj);
    WikiEngine::generate(&opts_for(&proj.root)).expect("generate");

    let opts = opts_for(&proj.root);
    let model = WikiEngine::analyze(&opts).expect("analyze");
    let name_map = module_file_names(&model);
    let out = proj.root.join(".rustcode").join("wiki");
    let mypkg = model
        .modules
        .iter()
        .find(|m| m.name == "mypkg")
        .expect("mypkg module");
    let mycrate = model
        .modules
        .iter()
        .find(|m| m.name == "mycrate")
        .expect("mycrate module");

    // Simulate the `--llm` enrichment pass: per-language sidecars for the module
    // that will be deleted, plus one for the module that survives.
    for lang in [WikiLang::Zh, WikiLang::En] {
        write_module_summary(&out, &model, &name_map, lang, mypkg, "MYPKG_STALE_SUMMARY")
            .expect("write mypkg summary");
    }
    write_module_summary(
        &out,
        &model,
        &name_map,
        WikiLang::Zh,
        mycrate,
        "MYCRATE_KEEP_SUMMARY",
    )
    .expect("write mycrate summary");

    for lang in ["zh", "en"] {
        assert!(
            proj.exists(&format!(".rustcode/wiki/{lang}/Modules/mypkg.summary.md")),
            "mypkg {lang} sidecar should exist before removal"
        );
    }

    // Delete the npm package module entirely (manifest + sources).
    proj.remove("web/package.json");
    proj.remove("web/index.js");
    proj.remove("web");

    let result = WikiEngine::sync(&opts).expect("sync");

    for lang in ["zh", "en"] {
        let sidecar = format!(".rustcode/wiki/{lang}/Modules/mypkg.summary.md");
        assert!(
            !proj.exists(&sidecar),
            "removed module's {lang} summary sidecar must be deleted, still at {sidecar}"
        );
        assert!(
            result
                .removed
                .iter()
                .any(|p| p.ends_with("Modules/mypkg.summary.md")),
            "removed sidecar should be reported in `removed`, got {:?}",
            result.removed
        );
    }

    // The surviving module's sidecar and embedded summary must be untouched.
    assert!(
        proj.exists(".rustcode/wiki/zh/Modules/mycrate.summary.md"),
        "surviving module's sidecar must remain on disk"
    );
    assert!(
        proj.read(".rustcode/wiki/zh/Modules/mycrate.md")
            .contains("MYCRATE_KEEP_SUMMARY"),
        "surviving module's page must still embed its summary"
    );
}

#[test]
fn test_user_edit_preserved_unless_forced() {
    let proj = TempProject::new("preserve");
    write_sample_project(&proj);

    WikiEngine::generate(&opts_for(&proj.root)).expect("generate");

    // Simulate a user manually editing a generated page.
    let edited = ".rustcode/wiki/zh/Home.md";
    proj.write(edited, "# MY CUSTOM HOME PAGE\n");

    // Sync without `--force`: the edit must survive and be reported as preserved.
    let res = WikiEngine::sync(&opts_for(&proj.root)).expect("sync");
    let preserved_home = proj.root.join(edited);
    assert!(
        res.preserved.contains(&preserved_home),
        "manually edited page should be preserved, got {:?}",
        res.preserved
    );
    assert_eq!(
        proj.read(edited),
        "# MY CUSTOM HOME PAGE\n",
        "user edit must not be clobbered on a normal sync"
    );

    // Sync with `--force`: the edit is overwritten by the regenerated content.
    let mut forced = opts_for(&proj.root);
    forced.force = true;
    let res = WikiEngine::sync(&forced).expect("sync with force");
    assert!(
        res.preserved.is_empty(),
        "no files should be preserved when force is set, got {:?}",
        res.preserved
    );
    assert_ne!(
        proj.read(edited),
        "# MY CUSTOM HOME PAGE\n",
        "force must overwrite the user edit"
    );
}

/// Build a model containing exactly one module, for provider-independent tests.
fn model_with_module(name: &str) -> (ProjectModel, ModuleInfo) {
    let module = ModuleInfo {
        name: name.to_string(),
        path: Path::new("src/auth").to_path_buf(),
        kind: "crate".to_string(),
        files: vec![FileInfo {
            rel_path: "src/auth/mod.rs".to_string(),
            loc: 10,
            lang: "rust".to_string(),
        }],
        loc: 10,
        depends_on: vec![],
    };
    let model = ProjectModel {
        root: Path::new("/tmp/wiki-test-model").to_path_buf(),
        name: "x".to_string(),
        kind: ProjectKind::Rust,
        modules: vec![module.clone()],
        file_count: 1,
        loc: 10,
        languages: Default::default(),
    };
    (model, module)
}

#[test]
fn test_write_module_summary_writes_sidecar_and_page() {
    let proj = TempProject::new("writemod");
    let out = proj.path(".rustcode/wiki");
    let (model, module) = model_with_module("auth");
    let name_map = module_file_names(&model);

    let sidecar = write_module_summary(
        &out,
        &model,
        &name_map,
        WikiLang::Zh,
        &module,
        "AUTH_SUMMARY_TEXT",
    )
    .expect("write_module_summary");

    assert!(sidecar.exists(), "sidecar file should be created");
    assert_eq!(
        proj.read(".rustcode/wiki/zh/Modules/auth.summary.md"),
        "AUTH_SUMMARY_TEXT",
        "sidecar must contain the summary verbatim"
    );
    let page = proj.read(".rustcode/wiki/zh/Modules/auth.md");
    assert!(
        page.contains("AUTH_SUMMARY_TEXT"),
        "the module page should embed the summary"
    );
    assert!(
        !page.contains("自然语言摘要"),
        "the placeholder must be replaced once a summary is present"
    );
}

#[test]
fn test_module_summary_survives_sync() {
    let proj = TempProject::new("survivesync");
    write_sample_project(&proj);
    WikiEngine::generate(&opts_for(&proj.root)).expect("generate");

    // Hand a summary to one module exactly as the LLM pass would.
    let opts = opts_for(&proj.root);
    let model = WikiEngine::analyze(&opts).expect("analyze");
    let name_map = module_file_names(&model);
    let mycrate = model
        .modules
        .iter()
        .find(|m| m.name == "mycrate")
        .expect("mycrate module");
    let out = proj.root.join(".rustcode").join("wiki");
    write_module_summary(
        &out,
        &model,
        &name_map,
        WikiLang::Zh,
        mycrate,
        "PERSIST_ME_XYZ",
    )
    .expect("write_module_summary");

    assert!(
        proj.read(".rustcode/wiki/zh/Modules/mycrate.md")
            .contains("PERSIST_ME_XYZ"),
        "page should contain the summary right after writing it"
    );

    // A subsequent sync must re-read the sidecar and keep the summary instead of
    // reverting to the placeholder.
    WikiEngine::sync(&opts).expect("sync");
    let page_after = proj.read(".rustcode/wiki/zh/Modules/mycrate.md");
    assert!(
        page_after.contains("PERSIST_ME_XYZ"),
        "summary must survive a sync via its sidecar, got: {page_after}"
    );
    assert!(
        proj.exists(".rustcode/wiki/zh/Modules/mycrate.summary.md"),
        "sidecar must remain on disk after sync"
    );
}

#[test]
fn test_sync_records_source_stats_and_is_cheap_when_unchanged() {
    let proj = TempProject::new("statsfast");
    write_sample_project(&proj);
    WikiEngine::generate(&opts_for(&proj.root)).expect("generate");

    // The manifest must now carry the cheap source snapshot.
    let manifest =
        WikiManifest::load(&proj.path(".rustcode/wiki/manifest.json")).expect("manifest loads");
    assert!(
        !manifest.source_stats.is_empty(),
        "source_stats should be recorded after generate"
    );
    assert!(
        manifest.source_stats.contains_key("src/lib.rs"),
        "source_stats should track the crate source file"
    );

    // A `sync` on the unchanged project must still report everything unchanged
    // (the cheap fast path skips the full re-scan but yields the same result).
    let res = WikiEngine::sync(&opts_for(&proj.root)).expect("sync");
    assert!(
        res.unchanged > 0,
        "unchanged project should report unchanged pages via the fast path"
    );
    assert!(res.created.is_empty() && res.updated.is_empty() && res.removed.is_empty());
}

#[test]
fn test_sync_stats_detect_real_change() {
    let proj = TempProject::new("statschange");
    write_sample_project(&proj);
    WikiEngine::generate(&opts_for(&proj.root)).expect("generate");

    // Modify a source file so its line count changes (this alters the rendered
    // module page, and changes both mtime and size so the cheap stats check must
    // notice and trigger a real re-scan).
    proj.write(
        "web/index.js",
        "// mypkg entry (changed)\nconsole.log('changed');\nconsole.log('extra line');\n",
    );

    let res = WikiEngine::sync(&opts_for(&proj.root)).expect("sync");
    assert!(
        res.updated.iter().any(|p| p.ends_with("Modules/mypkg.md")),
        "the changed module page should be regenerated, got {:?}",
        res.updated
    );

    // The updated manifest must continue to track the changed file's stats.
    let manifest =
        WikiManifest::load(&proj.path(".rustcode/wiki/manifest.json")).expect("manifest loads");
    assert!(
        manifest.source_stats.contains_key("web/index.js"),
        "source_stats should track the changed file"
    );
}

/// Failure-mode guard for the cheap fast path: when a *generated* page is deleted
/// but no source file changed, the fast path must NOT report "up to date" -- the
/// missing page has to be regenerated.
///
/// Both fast paths in `engine::run` gate on every recorded `wiki_files` entry still
/// existing on disk; this test pins that guard.
#[test]
fn test_sync_regenerates_deleted_generated_page() {
    let proj = TempProject::new("regenmissing");
    write_sample_project(&proj);

    WikiEngine::generate(&opts_for(&proj.root)).expect("generate");

    let page = ".rustcode/wiki/zh/Home.md";
    let original = proj.read(page);

    // Delete a generated page without touching any source file, so the recorded
    // `source_stats` snapshot still matches and only the existence/hash guard can
    // force a real regeneration.
    proj.remove(page);
    assert!(!proj.exists(page), "page should be gone before sync");

    let res = WikiEngine::sync(&opts_for(&proj.root)).expect("sync");

    let page_path = proj.root.join(page);
    assert!(
        res.created.contains(&page_path),
        "deleted generated page must be recreated, got created={:?} updated={:?}",
        res.created,
        res.updated
    );
    assert!(proj.exists(page), "page must exist again after sync");
    assert_eq!(
        proj.read(page),
        original,
        "regenerated page must be byte-identical to the original"
    );
}
