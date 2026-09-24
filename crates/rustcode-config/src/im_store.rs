//! IM identity mapping: which chat maps to which project and session.
//!
//! An IM channel receives messages identified only by a platform-side
//! `(platform, chat_id)` pair. Before that message can drive the agent, the
//! pair must resolve to a **project** (the agent identity in this fork) and to a
//! **session** (so the conversation keeps its context across messages).
//!
//! Storage is one small JSON file per binding, under
//! `$RUSTCODE_HOME/im/bindings/`, mirroring the `[schedule]` store's
//! "one record per file, corrupt files skipped" shape.
//!
//! # Why the file name is a hash
//!
//! `chat_id` arrives from the network and is therefore untrusted. Using it
//! verbatim in a path would let a crafted id (`../../etc/passwd`) escape the
//! store directory. The `[schedule]` store solved the same problem with a
//! conservative alphabet check on its ids; here the id space is wider (platform
//! chat ids can contain almost anything), so the value is **hashed** instead of
//! whitelisted. The original pair is kept inside the file -- the hash is only
//! ever a file name, never an identity.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// A resolved binding between one IM conversation and one project session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImBinding {
    /// Platform spelling (`dingtalk` / `feishu` / `wecom`).
    pub platform: String,
    /// Platform-side conversation identifier, as received from the network.
    /// Stored verbatim so the binding can be resolved back on the next message.
    pub chat_id: String,
    /// Absolute working directory this chat drives. The project *is* the agent
    /// identity here, since this fork has no hosted-agent concept.
    pub project: String,
    /// RustCode session id this chat's conversation lives in. Reused across
    /// messages so the agent keeps context; see [`upsert_preserving_session`].
    pub session_id: String,
    /// Unix seconds.
    pub created_at: i64,
    /// Unix seconds; refreshed on every message.
    pub updated_at: i64,
}

/// Root of the binding store.
pub fn bindings_root() -> PathBuf {
    crate::config::Config::config_dir()
        .join("im")
        .join("bindings")
}

/// Unix seconds, or 0 if the clock is before the epoch.
fn now_epoch_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Platforms are minted by us, not by the network, so a strict alphabet check is
/// both sufficient and more debuggable than hashing: a typo shows up as an
/// obviously-wrong file name rather than an opaque digest.
fn valid_platform(platform: &str) -> bool {
    !platform.is_empty()
        && platform.len() <= 32
        && platform
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// File stem for a binding: `<platform>-<sha256(chat_id) first 16 hex chars>`.
///
/// Truncating to 16 hex chars (64 bits) keeps names readable while making an
/// accidental collision between two chats infeasible in practice; the full
/// `chat_id` is compared on load, so even a collision cannot silently merge two
/// different conversations.
fn binding_stem_in(platform: &str, chat_id: &str) -> std::io::Result<String> {
    if !valid_platform(platform) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid IM platform {platform:?}"),
        ));
    }
    if chat_id.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty IM chat id".to_string(),
        ));
    }
    let digest = Sha256::digest(chat_id.as_bytes());
    let mut hex = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    Ok(format!("{platform}-{hex}"))
}

fn binding_path_in(root: &Path, platform: &str, chat_id: &str) -> std::io::Result<PathBuf> {
    Ok(root.join(format!("{}.json", binding_stem_in(platform, chat_id)?)))
}

fn save_in(root: &Path, binding: &ImBinding) -> std::io::Result<()> {
    let path = binding_path_in(root, &binding.platform, &binding.chat_id)?;
    std::fs::create_dir_all(root)?;
    let bytes = serde_json::to_vec_pretty(binding)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, bytes)
}

/// Load one binding. `Ok(None)` when the chat has never been seen.
///
/// The stored `chat_id` is compared against the requested one: a hash collision
/// (or a file hand-copied to a different name) must never resolve to the wrong
/// conversation.
fn load_in(root: &Path, platform: &str, chat_id: &str) -> std::io::Result<Option<ImBinding>> {
    let path = binding_path_in(root, platform, chat_id)?;
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let binding: ImBinding = serde_json::from_slice(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    if binding.platform != platform || binding.chat_id != chat_id {
        return Ok(None);
    }
    Ok(Some(binding))
}

fn list_in(root: &Path) -> Vec<ImBinding> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else {
        return out;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(bytes) = std::fs::read(&p) {
            if let Ok(b) = serde_json::from_slice::<ImBinding>(&bytes) {
                out.push(b); // corrupt files are skipped, matching the store's siblings
            }
        }
    }
    out.sort_by_key(|b| b.created_at);
    out
}

fn remove_in(root: &Path, platform: &str, chat_id: &str) -> std::io::Result<()> {
    match std::fs::remove_file(binding_path_in(root, platform, chat_id)?) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// Resolve `(platform, chat_id)` under the default root.
pub fn load(platform: &str, chat_id: &str) -> std::io::Result<Option<ImBinding>> {
    load_in(&bindings_root(), platform, chat_id)
}

/// Persist a binding under the default root.
pub fn save(binding: &ImBinding) -> std::io::Result<()> {
    save_in(&bindings_root(), binding)
}

/// Every binding under the default root (corrupt files skipped).
pub fn list() -> Vec<ImBinding> {
    list_in(&bindings_root())
}

/// Delete a binding. Absent is success -- deletion is idempotent.
pub fn remove(platform: &str, chat_id: &str) -> std::io::Result<()> {
    remove_in(&bindings_root(), platform, chat_id)
}

/// All bindings bound to one project, for tooling that manages a project's
/// channels (e.g. showing which chats currently drive it).
pub fn list_for_project(project: &str) -> Vec<ImBinding> {
    list_in(&bindings_root())
        .into_iter()
        .filter(|b| b.project == project)
        .collect()
}

/// Record that a chat drove the agent, creating the binding if new.
///
/// Semantics that matter for correctness:
///
/// - An **existing** binding keeps its `session_id`: that is what gives the IM
///   conversation continuity across messages. Only `updated_at` (and, if the
///   user reconfigured the channel, `project`) is refreshed.
/// - A **new** binding mints the caller's `session_id` and stamps `created_at`.
///
/// Returns the effective binding, so the caller does not have to re-read.
pub fn upsert_preserving_session(
    platform: &str,
    chat_id: &str,
    project: &str,
    new_session_id: &str,
) -> std::io::Result<ImBinding> {
    upsert_preserving_session_in(
        &bindings_root(),
        platform,
        chat_id,
        project,
        new_session_id,
        now_epoch_secs(),
    )
}

fn upsert_preserving_session_in(
    root: &Path,
    platform: &str,
    chat_id: &str,
    project: &str,
    new_session_id: &str,
    now: i64,
) -> std::io::Result<ImBinding> {
    let effective = match load_in(root, platform, chat_id)? {
        Some(mut existing) => {
            // Re-binding a chat to a different project is a user action (they
            // moved the channel in config), so honour it -- but keep the session
            // so the conversation is not silently dropped.
            existing.project = project.to_string();
            existing.updated_at = now;
            existing
        }
        None => ImBinding {
            platform: platform.to_string(),
            chat_id: chat_id.to_string(),
            project: project.to_string(),
            session_id: new_session_id.to_string(),
            created_at: now,
            updated_at: now,
        },
    };
    save_in(root, &effective)?;
    Ok(effective)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(platform: &str, chat_id: &str, project: &str, session: &str) -> ImBinding {
        ImBinding {
            platform: platform.into(),
            chat_id: chat_id.into(),
            project: project.into(),
            session_id: session.into(),
            created_at: 100,
            updated_at: 100,
        }
    }

    #[test]
    fn roundtrips_a_binding() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let b = binding("dingtalk", "cid-123", "/tmp/proj", "sess-1");
        save_in(root, &b).unwrap();
        let back = load_in(root, "dingtalk", "cid-123").unwrap().unwrap();
        assert_eq!(back, b);
        assert_eq!(list_in(root).len(), 1);
        remove_in(root, "dingtalk", "cid-123").unwrap();
        assert!(list_in(root).is_empty());
        // Removal is idempotent.
        remove_in(root, "dingtalk", "cid-123").unwrap();
    }

    #[test]
    fn unseen_chat_loads_as_none_not_error() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(load_in(tmp.path(), "dingtalk", "never-seen")
            .unwrap()
            .is_none());
    }

    #[test]
    fn chat_ids_cannot_escape_the_store_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // A hostile platform is rejected outright (it is minted by us, not the
        // network, so there is no reason to accept odd shapes).
        for bad in ["../evil", "a/b", "", "Dingtalk", "ding talk"] {
            assert!(
                binding_path_in(root, bad, "cid").is_err(),
                "platform {bad:?} must be rejected"
            );
        }
        // A hostile chat id is hashed, so it can never touch the path -- the file
        // stays inside the store and no traversal target is created.
        let hostile = binding("dingtalk", "../../../../etc/passwd", "/tmp/p", "s");
        save_in(root, &hostile).unwrap();
        let escaped = tmp.path().parent().unwrap().join("etc").join("passwd");
        assert!(!escaped.exists(), "traversal escaped the store");
        // It is still retrievable by its exact id.
        assert_eq!(
            load_in(root, "dingtalk", "../../../../etc/passwd")
                .unwrap()
                .unwrap()
                .session_id,
            "s"
        );
        // And the on-disk name is the digests, not the raw id.
        let name = binding_stem_in("dingtalk", "../../../../etc/passwd").unwrap();
        assert!(!name.contains(".."));
        assert!(!name.contains('/'));
    }

    #[test]
    fn chat_id_is_not_recoverable_from_the_file_name() {
        // The file name must not leak the conversation identifier.
        let stem = binding_stem_in("feishu", "oc_secret_chat_id").unwrap();
        assert!(!stem.contains("secret"));
        assert!(stem.starts_with("feishu-"));
    }

    #[test]
    fn same_chat_id_on_different_platforms_does_not_collide() {
        let a = binding_stem_in("dingtalk", "same").unwrap();
        let b = binding_stem_in("feishu", "same").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn differing_chat_ids_produce_differing_files() {
        assert_ne!(
            binding_stem_in("dingtalk", "a").unwrap(),
            binding_stem_in("dingtalk", "b").unwrap()
        );
    }

    #[test]
    fn upsert_creates_then_preserves_the_session() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // First message mints the session.
        let first =
            upsert_preserving_session_in(root, "dingtalk", "cid", "/tmp/p", "sess-A", 10).unwrap();
        assert_eq!(first.session_id, "sess-A");
        assert_eq!(first.created_at, 10);

        // Later messages must KEEP it, or the agent loses the conversation.
        let second =
            upsert_preserving_session_in(root, "dingtalk", "cid", "/tmp/p", "sess-B", 20).unwrap();
        assert_eq!(second.session_id, "sess-A", "session must survive reuse");
        assert_eq!(second.created_at, 10, "created_at is not rewritten");
        assert_eq!(
            second.updated_at, 20,
            "updated_at tracks the latest message"
        );
        assert_eq!(list_in(root).len(), 1, "reuse must not add a second file");
    }

    #[test]
    fn upsert_honours_a_project_change_but_keeps_the_session() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        upsert_preserving_session_in(root, "dingtalk", "cid", "/tmp/old", "sess-A", 10).unwrap();
        let moved = upsert_preserving_session_in(root, "dingtalk", "cid", "/tmp/new", "sess-B", 20)
            .unwrap();
        assert_eq!(moved.project, "/tmp/new");
        assert_eq!(moved.session_id, "sess-A");
    }

    #[test]
    fn mismatched_file_contents_do_not_resolve_to_the_wrong_chat() {
        // Guards the hash-collision / hand-copied-file case: the stored pair must
        // match the requested one.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let b = binding("dingtalk", "real-chat", "/tmp/p", "s");
        save_in(root, &b).unwrap();
        // Copy the file under another chat's stem.
        let src = binding_path_in(root, "dingtalk", "real-chat").unwrap();
        let dst = binding_path_in(root, "dingtalk", "other-chat").unwrap();
        std::fs::copy(&src, &dst).unwrap();
        assert!(
            load_in(root, "dingtalk", "other-chat").unwrap().is_none(),
            "a mismatched payload must not resolve"
        );
    }

    #[test]
    fn list_for_project_filters_by_project() {
        let tmp = tempfile::tempdir().unwrap();
        // list_for_project reads the default root, so exercise the filter logic
        // via list_in + filter to stay hermetic.
        let root = tmp.path();
        save_in(root, &binding("dingtalk", "c1", "/tmp/a", "s1")).unwrap();
        save_in(root, &binding("feishu", "c2", "/tmp/a", "s2")).unwrap();
        save_in(root, &binding("dingtalk", "c3", "/tmp/b", "s3")).unwrap();
        let for_a: Vec<_> = list_in(root)
            .into_iter()
            .filter(|b| b.project == "/tmp/a")
            .collect();
        assert_eq!(for_a.len(), 2);
    }

    #[test]
    fn corrupt_files_are_skipped_not_fatal() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root).unwrap();
        std::fs::write(root.join("broken.json"), b"{ not json").unwrap();
        save_in(root, &binding("dingtalk", "good", "/tmp/p", "s")).unwrap();
        assert_eq!(
            list_in(root).len(),
            1,
            "corrupt file must not abort the scan"
        );
    }

    #[test]
    fn non_json_files_are_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root).unwrap();
        std::fs::write(root.join("notes.txt"), b"hello").unwrap();
        assert!(list_in(root).is_empty());
    }
}
