//! Shared test plumbing for the plugin module: [`isolated_home`] gives each
//! test its own user tree in a temp dir, handed to the code under test through
//! [`IsolatedHome::dirs`] — nothing reads the environment, so these tests need
//! no serialisation.

use std::path::{Path, PathBuf};

use crate::ProductDirs;

pub struct IsolatedHome {
    _tmp: tempfile::TempDir,
    path: PathBuf,
}

impl IsolatedHome {
    /// The user tree.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// This tree as the plugin API takes it, with a project dir named `.ours`.
    pub fn dirs(&self) -> ProductDirs {
        ProductDirs::new(&self.path, ".ours")
    }
}

/// A fresh temp user tree. Keep the returned value alive for the test.
pub fn isolated_home() -> IsolatedHome {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().to_path_buf();
    IsolatedHome { _tmp: tmp, path }
}
