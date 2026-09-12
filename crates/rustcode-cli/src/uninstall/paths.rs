//! Install-location detection mirroring scripts/install.sh and install.ps1.

// `Path` only appears in `unix_rc_paths_for_home`, which is gated to
// `cfg(unix)` for the production path and `cfg(all(not(unix), test))`
// for the cross-platform test stub. A Windows non-test build sees
// neither, so the import would be flagged unused.
#[cfg(any(unix, test))]
use std::path::Path;
use std::path::PathBuf;

/// Return the rustcode data root (`$RUSTCODE_HOME` when set, or
/// `~/.rustcode/` by default). Routes through [`rustcode_config::config::Config::config_dir`]
/// so install/setup/skill/plugin/uninstall all agree on a single root --
/// previously this module looked at a separate `RUSTCODE_HOME_OVERRIDE`
/// variable, which let users customise their data dir but then lose track
/// of it at uninstall time. One variable, one semantics.
pub fn rustcode_dir() -> PathBuf {
    rustcode_config::config::Config::config_dir()
}

/// Filenames inside `$RUSTCODE_HOME/` that the uninstaller knows about, grouped.
pub struct UninstallManifest {
    pub credential_files: &'static [&'static str],
    pub state_files: &'static [&'static str],
    pub state_dirs: &'static [&'static str],
    pub state_prefixes: &'static [&'static str],
}

pub fn uninstall_manifest() -> UninstallManifest {
    UninstallManifest {
        credential_files: &["mcp.json", "mcp_auth.toml", "config.toml", "RUSTCODE.md"],
        state_files: &[
            "history",
            "input_history.txt",
            "recent_dirs.txt",
            "codingplan_sync.json",
            "device_id",
        ],
        state_dirs: &["staged", "telemetry", "plugins", "commands", "skills"],
        state_prefixes: &["notice."],
    }
}

#[cfg(unix)]
pub struct UnixRcPaths {
    pub zshrc: PathBuf,
    pub bashrc: PathBuf,
}

#[cfg(unix)]
pub fn unix_rc_paths() -> UnixRcPaths {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    unix_rc_paths_for_home(&home)
}

#[cfg(unix)]
pub fn unix_rc_paths_for_home(home: &Path) -> UnixRcPaths {
    UnixRcPaths {
        zshrc: home.join(".zshrc"),
        bashrc: home.join(".bashrc"),
    }
}

// Test-only counterpart so the test compiles cross-platform.
#[cfg(all(not(unix), test))]
pub struct UnixRcPaths {
    pub zshrc: PathBuf,
    pub bashrc: PathBuf,
}
#[cfg(all(not(unix), test))]
pub fn unix_rc_paths_for_home(home: &Path) -> UnixRcPaths {
    UnixRcPaths {
        zshrc: home.join(".zshrc"),
        bashrc: home.join(".bashrc"),
    }
}

/// Default Windows install-dir candidates (matches install.ps1).
#[cfg(windows)]
pub fn windows_install_dir_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(p) = std::env::var_os("RUSTCODE_PREFIX") {
        out.push(PathBuf::from(p));
    }
    if let Some(p) = std::env::var_os("LOCALAPPDATA") {
        out.push(PathBuf::from(p).join(rustcode_config::distribution::WINDOWS_INSTALL_DIR));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn rustcode_dir_follows_home() {
        // `rustcode_dir()` is exactly the resolved config root = RUSTCODE_HOME.
        // (The unset->`~/.rustcode` fallback lives in core and is covered by
        // `Config::resolve_config_dir` tests; here we just verify delegation.)
        // RUSTCODE_HOME is always set in tests (by the test-support ctor, or by a
        // sibling test), so assert `rustcode_dir()` tracks it.
        let home = std::env::var("RUSTCODE_HOME").expect("RUSTCODE_HOME set in tests");
        assert_eq!(rustcode_dir(), std::path::PathBuf::from(home));
    }

    #[test]
    #[serial]
    fn rustcode_home_env_wins() {
        // Under unified semantics, RUSTCODE_HOME IS the data root.
        // The legacy RUSTCODE_HOME_OVERRIDE variable is gone.
        //
        // `set_var` is PROCESS-GLOBAL, so the override must be undone even if the
        // assertion panics: a leaked value is visible to every other test in this
        // binary that is not holding the serial lock.
        struct EnvGuard {
            prev: Option<std::ffi::OsString>,
        }
        impl Drop for EnvGuard {
            fn drop(&mut self) {
                match self.prev.take() {
                    Some(value) => std::env::set_var("RUSTCODE_HOME", value),
                    None => std::env::remove_var("RUSTCODE_HOME"),
                }
            }
        }
        let _guard = EnvGuard {
            prev: std::env::var_os("RUSTCODE_HOME"),
        };
        std::env::set_var("RUSTCODE_HOME", "/tmp/override");
        assert_eq!(rustcode_dir(), std::path::PathBuf::from("/tmp/override"));
    }

    #[test]
    fn manifest_groups_credentials_correctly() {
        let m = uninstall_manifest();
        for f in ["mcp.json", "mcp_auth.toml", "config.toml", "RUSTCODE.md"] {
            assert!(m.credential_files.contains(&f), "missing {f}");
        }
        // `auth.toml` came from the removed `rustcode-auth` crate and is no longer
        // a live credential (a startup migration deletes any legacy copy), so the
        // uninstaller must not treat it as one.
        assert!(!m.credential_files.contains(&"auth.toml"));
    }

    #[test]
    fn manifest_groups_state_correctly() {
        let m = uninstall_manifest();
        for f in [
            "history",
            "input_history.txt",
            "recent_dirs.txt",
            "codingplan_sync.json",
            "device_id",
        ] {
            assert!(m.state_files.contains(&f), "missing {f}");
        }
        for d in ["staged", "telemetry", "plugins", "commands", "skills"] {
            assert!(m.state_dirs.contains(&d), "missing {d}");
        }
    }

    #[test]
    fn rc_files_includes_zshrc_and_bashrc() {
        let rc = unix_rc_paths_for_home(std::path::Path::new("/Users/test"));
        assert_eq!(rc.zshrc, std::path::Path::new("/Users/test/.zshrc"));
        assert_eq!(rc.bashrc, std::path::Path::new("/Users/test/.bashrc"));
    }
}
