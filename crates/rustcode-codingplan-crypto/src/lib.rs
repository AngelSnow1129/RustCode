//! Open-source placeholder. The real signing crate is overlaid by a
//! distribution build that ships the closed-source managed-signing overlay;
//! this stub exists only so the public workspace compiles and carries no
//! signing logic.

#![deny(unsafe_code)]

/// Sentinel only. `0` = "unavailable" (real algorithms start at 1). Never
/// read in an open-source build -- the `codingplan-crypto` feature is off,
/// so this crate is not linked.
pub const ALGORITHM_VERSION: u8 = 0;

/// Placeholder matching the real crate's signature so the workspace
/// type-checks. Unreachable in open-source builds; the closed-source overlay
/// replaces this file wholesale.
// Signature mirrors the closed-source overlay; argument count is fixed.
#[allow(clippy::too_many_arguments)]
pub fn sign_v1(
    _method: &str,
    _path: &str,
    _body: &[u8],
    _oauth_token: &str,
    _user_id: &str,
    _timestamp_unix: u64,
    _nonce: &[u8; 16],
    _client_version: &str,
) -> Vec<(&'static str, String)> {
    unreachable!("request signing requires a distribution build that ships the closed-source signing overlay")
}
