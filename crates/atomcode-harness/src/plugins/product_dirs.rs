//! `product-dirs`: where the product keeps its data, said once for the tree.
//!
//! Every row that persists or guards something — skills, memory, MCP, sessions,
//! plugins, the credential gates, the walkers that skip our own project dir —
//! reads the same [`ProductDirs`] from this row's seam. They used to resolve
//! `$ATOMCODE_HOME` each for itself, with a literal `.atomcode` fallback, so a
//! program embedding this harness could only relocate them by exporting the
//! variable before every entry point; the one entry point that forgot read its
//! login from the wrong tree and told a logged-in person they were not.
//!
//! Now a row that needs the dirs and does not find them fails to mount
//! ([`crate::product_dirs`]), the same way a row that needs the model does
//! without the `llm` seam — never a quiet read from somewhere else.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use atomcode_capabilities::ProductDirs;
use atomcode_plexus::{Context, Plugin};
use serde::Deserialize;
use serde_json::Value;

use crate::seams::ProductDirsSvc;

#[derive(Debug, Deserialize, Default)]
struct ProductDirsRow {
    /// The user-level tree. Absent: this build's default (`$ATOMCODE_HOME`,
    /// else `~/<distribution::HOME_DIR_NAME>`) — the harness launcher acting as
    /// the host. A program that embeds the harness writes its own.
    #[serde(default)]
    user: Option<String>,
    /// The per-project dir's name. Absent: `distribution::PROJECT_DIR_NAME`.
    #[serde(default)]
    project_dir_name: Option<String>,
    /// The tree's default name under a home, when it differs from the project
    /// dir's. Absent: `distribution::HOME_DIR_NAME`.
    #[serde(default)]
    home_dir_name: Option<String>,
}

/// The `product-dirs` row. First in the tree: every row that reads its seam
/// comes after it.
pub struct ProductDirsPlugin;

#[async_trait]
impl Plugin for ProductDirsPlugin {
    fn name(&self) -> &'static str {
        "product-dirs"
    }
    fn provides(&self) -> &'static [&'static str] {
        &["product-dirs"]
    }
    fn description(&self) -> &'static str {
        "where the product keeps its data: the user tree and the per-project dir"
    }
    async fn apply(&self, ctx: &Context, config: &Value) -> Result<(), String> {
        let row: ProductDirsRow = if config.is_null() {
            ProductDirsRow::default()
        } else {
            serde_json::from_value(config.clone()).map_err(|e| format!("bad config: {e}"))?
        };
        let dirs = resolve(row);
        let _ = ctx
            .provide::<ProductDirsSvc>(Arc::new(dirs))
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn resolve(row: ProductDirsRow) -> ProductDirs {
    use atomcode_config::distribution::{HOME_DIR_NAME, PROJECT_DIR_NAME};
    let user = row
        .user
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(crate::home);
    ProductDirs::new(
        user,
        row.project_dir_name
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| PROJECT_DIR_NAME.to_string()),
    )
    .with_home_dir_name(
        row.home_dir_name
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| HOME_DIR_NAME.to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_the_row_says_is_what_every_row_gets() {
        let dirs = resolve(ProductDirsRow {
            user: Some("/elsewhere/fork-tree".into()),
            project_dir_name: Some(".fork".into()),
            home_dir_name: Some(".fork".into()),
        });
        assert_eq!(dirs.user(), std::path::Path::new("/elsewhere/fork-tree"));
        assert_eq!(dirs.project_dir_name(), ".fork");
        assert_eq!(dirs.home_dir_name(), ".fork");
    }
}
