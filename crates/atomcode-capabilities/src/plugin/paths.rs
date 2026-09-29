use std::path::{Component, Path, PathBuf};

use super::state::InstallScope;

use crate::ProductDirs;

/// Root directory: `<user tree>/plugins/`.
///
/// `Option` for the callers' `?`/`ok_or` shape; always `Some`.
pub fn plugins_root(user_dir: &Path) -> Option<PathBuf> {
    Some(user_dir.join("plugins"))
}

pub fn marketplaces_root(user_dir: &Path) -> Option<PathBuf> {
    Some(plugins_root(user_dir)?.join("marketplaces"))
}

pub fn marketplaces_file(user_dir: &Path) -> Option<PathBuf> {
    Some(plugins_root(user_dir)?.join("marketplaces.json"))
}

pub fn installed_plugins_file(user_dir: &Path) -> Option<PathBuf> {
    Some(plugins_root(user_dir)?.join("installed_plugins.json"))
}

/// Project-level plugins directory for a working directory and scope.
///
/// - `Project` scope: `<project dir>/plugins/`
/// - `Local` scope: `<project dir>/plugins/local/`
///
/// Returns `None` for `User` scope (user scope uses the global `plugins_root()`).
pub fn project_plugins_root(
    dirs: &ProductDirs,
    working_dir: &Path,
    scope: &InstallScope,
) -> Option<PathBuf> {
    let root = dirs.project(working_dir).join("plugins");
    match scope {
        InstallScope::Project => Some(root),
        InstallScope::Local => Some(root.join("local")),
        InstallScope::User => None,
    }
}

/// Project-level `installed_plugins.json` path for a given scope.
pub fn project_installed_plugins_file(
    dirs: &ProductDirs,
    working_dir: &Path,
    scope: &InstallScope,
) -> Option<PathBuf> {
    project_plugins_root(dirs, working_dir, scope).map(|root| root.join("installed_plugins.json"))
}

/// True when a project/local scope's `installed_plugins.json` resolves to the
/// same file as the user-scope one.
///
/// This happens when the project dir under `working_dir` IS the user tree (e.g.
/// running from `$HOME` with the default layout), where its `plugins/` is the
/// same directory as the global `plugins_root()` — the same state file would
/// otherwise be read once per scope and every plugin enumerated twice. Callers
/// (asset/status iteration, `plugin list`) should skip such scopes.
pub fn scope_state_file_aliases_user_scope(
    dirs: &ProductDirs,
    working_dir: &Path,
    scope: &InstallScope,
) -> bool {
    let Some(scope_file) = project_installed_plugins_file(dirs, working_dir, scope) else {
        return false;
    };
    let Some(user_file) = installed_plugins_file(dirs.user()) else {
        return false;
    };
    let (scope_file, user_file) = match (
        std::fs::canonicalize(&scope_file),
        std::fs::canonicalize(&user_file),
    ) {
        (Ok(a), Ok(b)) => (a, b),
        // State files need not exist yet (for example before the first install),
        // so compare normalized absolute paths rather than raw PathBuf values.
        _ => (
            normalize_for_compare(&scope_file),
            normalize_for_compare(&user_file),
        ),
    };
    paths_equal(&scope_file, &user_file)
}

fn normalize_for_compare(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn paths_equal(a: &Path, b: &Path) -> bool {
    #[cfg(windows)]
    {
        a.to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        a == b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugins_live_under_the_user_tree_handed_in() {
        let tree = Path::new("/elsewhere/tree");
        assert_eq!(plugins_root(tree).unwrap(), tree.join("plugins"));
    }

    fn dirs() -> ProductDirs {
        ProductDirs::new("/elsewhere/tree", ".ours")
    }

    #[test]
    fn project_plugins_root_project_scope() {
        let dir = std::path::Path::new("/tmp/myproject");
        let root = project_plugins_root(&dirs(), dir, &InstallScope::Project).unwrap();
        assert_eq!(
            root,
            std::path::PathBuf::from("/tmp/myproject/.ours/plugins")
        );
    }

    #[test]
    fn project_plugins_root_local_scope() {
        let dir = std::path::Path::new("/tmp/myproject");
        let root = project_plugins_root(&dirs(), dir, &InstallScope::Local).unwrap();
        assert_eq!(
            root,
            std::path::PathBuf::from("/tmp/myproject/.ours/plugins/local")
        );
    }

    #[test]
    fn project_plugins_root_user_scope_returns_none() {
        let dir = std::path::Path::new("/tmp/myproject");
        assert!(project_plugins_root(&dirs(), dir, &InstallScope::User).is_none());
    }

    #[test]
    fn scope_state_file_aliases_user_scope_when_cwd_is_home() {
        // The user tree is <home>/.ours; with cwd == <home> the project scope's
        // installed_plugins.json IS the user scope's.
        let tmp = tempfile::tempdir().unwrap();
        let home_dir = tmp.path().join("home");
        let dirs = ProductDirs::new(home_dir.join(".ours"), ".ours");
        assert!(scope_state_file_aliases_user_scope(
            &dirs,
            &home_dir,
            &InstallScope::Project
        ));
        // Local scope lives in <home>/.ours/plugins/local/, a different file.
        assert!(!scope_state_file_aliases_user_scope(
            &dirs,
            &home_dir,
            &InstallScope::Local
        ));
        // User scope has no project state file.
        assert!(!scope_state_file_aliases_user_scope(
            &dirs,
            &home_dir,
            &InstallScope::User
        ));
    }

    #[test]
    fn scope_state_file_aliases_user_scope_false_for_other_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let home_dir = tmp.path().join("home");
        let dirs = ProductDirs::new(home_dir.join(".ours"), ".ours");
        let other = tmp.path().join("projects/myproj");
        assert!(!scope_state_file_aliases_user_scope(
            &dirs,
            &other,
            &InstallScope::Project
        ));
    }

    #[test]
    fn normalize_for_compare_resolves_relative_dot_and_parent_components() {
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(
            normalize_for_compare(Path::new("project/../.ours/plugins")),
            cwd.join(".ours/plugins")
        );
        assert_eq!(
            normalize_for_compare(Path::new("./.ours/plugins")),
            cwd.join(".ours/plugins")
        );
    }
}
