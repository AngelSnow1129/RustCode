//! Bridging an inbound IM message to an agent turn, and the reply back out.
//!
//! The runtime-side wiring lives in the driver (it needs the headless entry
//! points, which only the binary owns). Everything that can be decided without
//! a live agent lives here, so it is testable in isolation:
//!
//! - [`split_reply`] -- chunking an answer to a platform's message-size limit.
//! - [`RecentMessages`] -- redelivery guard. Platforms retry unacked frames, and
//!   a retried message must not drive the agent a second time.
//!
//! Keeping these pure is deliberate: they are exactly the places where an
//! off-by-one silently truncates a user's answer, or a duplicated id silently
//! runs a turn twice.

/// Platform message-size limits, in **characters**, not bytes.
///
/// These are conservative values chosen to stay under each platform's cap after
/// the transport's own framing. They are deliberately not configurable: a user
/// raising the limit past the platform's real cap would turn a working reply
/// into a transport error, and the failure would look like a RustCode bug.
pub mod limits {
    /// DingTalk text messages.
    pub const DINGTALK: usize = 4000;
    /// Feishu text messages.
    pub const FEISHU: usize = 4000;
    /// WeCom text messages.
    pub const WECOM: usize = 2000;
}

/// Message-size limit for a platform spelling. Unknown platforms get the most
/// conservative limit rather than an unbounded one (fail small, not loud).
pub fn limit_for(platform: &str) -> usize {
    match platform.trim().to_ascii_lowercase().as_str() {
        "dingtalk" => limits::DINGTALK,
        "feishu" | "lark" => limits::FEISHU,
        "wecom" | "weixin" => limits::WECOM,
        _ => limits::WECOM,
    }
}

/// One piece of a chunked reply, with the platform it is destined for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgedReply {
    pub platform: String,
    pub chat_id: String,
    pub reply_token: Option<String>,
    /// Chunks in order. Send them sequentially; index order is presentation.
    pub chunks: Vec<String>,
}

impl BridgedReply {
    /// Build a reply, chunking `text` to the platform's limit.
    ///
    /// Returns `None` when there is nothing to send (empty or whitespace-only
    /// text): sending an empty message is a platform error on some transports
    /// and a confusing blank bubble on others.
    pub fn new(
        platform: &str,
        chat_id: &str,
        reply_token: Option<String>,
        text: &str,
    ) -> Option<Self> {
        if text.trim().is_empty() {
            return None;
        }
        Some(Self {
            platform: platform.to_string(),
            chat_id: chat_id.to_string(),
            reply_token,
            chunks: split_reply(text, limit_for(platform)),
        })
    }

    /// Whether the reply had to be split, for a caller that wants to warn or
    /// annotate (e.g. prefixing "part 1/N").
    pub fn is_chunked(&self) -> bool {
        self.chunks.len() > 1
    }
}

/// Split `text` into chunks no longer than `limit` **characters**.
///
/// Splitting prefers, in order: a blank line (`\n\n`), then a single newline,
/// then a space -- so code blocks and paragraphs survive intact where possible.
/// A single token longer than the limit is hard-split; there is no way to honour
/// both "keep words whole" and "never exceed the limit", and exceeding the limit
/// is the failure that loses data.
///
/// Invariant: concatenating the chunks reproduces every non-whitespace character
/// of the input, in order. Only whitespace at a split boundary is dropped.
/// Returns an empty vector for empty/whitespace-only input.
pub fn split_reply(text: &str, limit: usize) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    // A zero limit cannot make progress; treat it as "no chunking" rather than
    // looping forever or dropping the text.
    if limit == 0 {
        return vec![text.to_string()];
    }

    let mut out: Vec<String> = Vec::new();
    let mut rest = text;

    while rest.chars().count() > limit {
        let head = take_chars(rest, limit);
        // `head` is exactly `limit` chars, `rest` is longer, so head cannot be
        // the whole string: the split points below are always within bounds.
        let cut = best_split_point(head).unwrap_or(head.len()); // no natural break -> hard split
        let (chunk, tail) = rest.split_at(cut);
        let chunk = chunk.trim_end();
        if !chunk.is_empty() {
            out.push(chunk.to_string());
        }
        rest = tail.trim_start();
    }

    let tail = rest.trim_end();
    if !tail.is_empty() {
        out.push(tail.to_string());
    }
    out
}

/// Byte offset just past the first `n` characters of `s`.
fn take_chars(s: &str, n: usize) -> &str {
    match s.char_indices().nth(n) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

/// Best place to cut `head` (which is already at the limit), returned as a byte
/// offset. Prefers a paragraph break, then a line break, then a space.
fn best_split_point(head: &str) -> Option<usize> {
    // Paragraph break: cut AFTER the blank line, so the chunk ends with it.
    if let Some(idx) = head.rfind("\n\n") {
        return Some(idx + 2);
    }
    if let Some(idx) = head.rfind('\n') {
        return Some(idx + 1);
    }
    head.rfind(' ').map(|idx| idx + 1)
}

/// Tracks recently-seen platform message ids so a redelivered frame is ignored.
///
/// Bounded: the set keeps the newest `capacity` ids, so a long-lived adapter
/// cannot grow without limit. A plain FIFO eviction is acceptable here because
/// platforms redeliver within a short retry window, not arbitrarily later --
/// and the cost of a rare late duplicate is one extra turn, whereas unbounded
/// growth is a slow leak.
#[derive(Debug)]
pub struct RecentMessages {
    ids: std::collections::VecDeque<String>,
    seen: std::collections::HashSet<String>,
    capacity: usize,
}

impl RecentMessages {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            ids: std::collections::VecDeque::with_capacity(capacity),
            seen: std::collections::HashSet::with_capacity(capacity),
            capacity,
        }
    }

    /// Record `id`. Returns `true` when it is new (the caller should proceed),
    /// `false` when it was already seen (the caller must skip the message).
    ///
    /// An empty id is treated as always-new: an adapter that could not read an
    /// id must not have its messages silently dropped forever.
    pub fn observe(&mut self, id: &str) -> bool {
        if id.is_empty() {
            return true;
        }
        if self.seen.contains(id) {
            return false;
        }
        self.seen.insert(id.to_string());
        self.ids.push_back(id.to_string());
        while self.ids.len() > self.capacity {
            if let Some(old) = self.ids.pop_front() {
                self.seen.remove(&old);
            }
        }
        true
    }

    /// Number of ids currently remembered.
    pub fn len(&self) -> usize {
        self.seen.len()
    }

    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_is_one_chunk() {
        assert_eq!(split_reply("hello", 100), vec!["hello"]);
    }

    #[test]
    fn empty_and_whitespace_produce_no_chunks() {
        assert!(split_reply("", 100).is_empty());
        assert!(split_reply("   \n\t ", 100).is_empty());
    }

    #[test]
    fn every_chunk_respects_the_limit() {
        let text = "word ".repeat(500); // 2500 chars
        let chunks = split_reply(&text, 100);
        assert!(chunks.len() > 1);
        for c in &chunks {
            assert!(
                c.chars().count() <= 100,
                "chunk exceeded the limit: {} chars",
                c.chars().count()
            );
        }
    }

    #[test]
    fn splitting_loses_only_boundary_whitespace() {
        let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa";
        let chunks = split_reply(text, 12);
        // Reconstruct by concatenating (no spaces re-inserted) and compare the
        // non-whitespace character stream, which must be identical and in order.
        let joined: String = chunks.concat();
        let strip = |s: &str| -> String { s.chars().filter(|c| !c.is_whitespace()).collect() };
        assert_eq!(strip(&joined), strip(text));
    }

    #[test]
    fn prefers_paragraph_boundaries() {
        let text = format!("{}\n\n{}", "a".repeat(50), "b".repeat(50));
        let chunks = split_reply(&text, 60);
        assert_eq!(chunks.len(), 2);
        // The first chunk ends at the paragraph break, so the second is untouched.
        assert_eq!(chunks[0], "a".repeat(50));
        assert_eq!(chunks[1], "b".repeat(50));
    }

    #[test]
    fn hard_splits_a_single_oversized_token() {
        // No whitespace anywhere: the only way to stay under the limit is a hard
        // split, and losing data here would be worse than breaking the word.
        let text = "x".repeat(250);
        let chunks = split_reply(&text, 100);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks.concat(), text);
        for c in &chunks {
            assert!(c.chars().count() <= 100);
        }
    }

    #[test]
    fn multibyte_characters_are_never_split_mid_codepoint() {
        // Each CJK char is 3 bytes; a byte-based split would panic or corrupt.
        let text = "中".repeat(50);
        let chunks = split_reply(&text, 7);
        for c in &chunks {
            assert!(c.chars().count() <= 7);
        }
        assert_eq!(chunks.concat(), text, "multibyte text must survive intact");
    }

    #[test]
    fn zero_limit_does_not_loop_or_drop_text() {
        let chunks = split_reply("abc", 0);
        assert_eq!(chunks, vec!["abc"]);
    }

    #[test]
    fn limit_boundary_is_exact_not_off_by_one() {
        // Exactly at the limit -> a single chunk; one over -> two.
        let exact = "y".repeat(100);
        assert_eq!(split_reply(&exact, 100).len(), 1);
        let over = "y".repeat(101);
        assert_eq!(split_reply(&over, 100).len(), 2);
    }

    #[test]
    fn reply_is_none_for_blank_text() {
        assert!(BridgedReply::new("dingtalk", "cid", None, "   ").is_none());
        let r = BridgedReply::new("dingtalk", "cid", None, "hi").unwrap();
        assert!(!r.is_chunked());
        assert_eq!(r.chunks, vec!["hi"]);
    }

    #[test]
    fn reply_chunks_to_the_platform_limit() {
        let long = "z".repeat(limits::WECOM + 10);
        let r = BridgedReply::new("wecom", "cid", Some("tok".into()), &long).unwrap();
        assert!(r.is_chunked());
        assert_eq!(r.reply_token.as_deref(), Some("tok"));
        for c in &r.chunks {
            assert!(c.chars().count() <= limits::WECOM);
        }
    }

    #[test]
    fn unknown_platform_gets_the_conservative_limit() {
        assert_eq!(limit_for("nope"), limits::WECOM);
        assert_eq!(limit_for("dingtalk"), limits::DINGTALK);
        assert_eq!(limit_for("  DingTalk "), limits::DINGTALK);
        assert_eq!(limit_for("lark"), limits::FEISHU);
    }

    #[test]
    fn redelivered_message_is_ignored_once() {
        let mut recent = RecentMessages::new(8);
        assert!(recent.observe("m1"), "first sighting must proceed");
        assert!(!recent.observe("m1"), "redelivery must be skipped");
        assert!(recent.observe("m2"));
    }

    #[test]
    fn redelivery_guard_is_bounded_and_evicts_oldest() {
        let mut recent = RecentMessages::new(2);
        assert!(recent.observe("a"));
        assert!(recent.observe("b"));
        assert_eq!(recent.len(), 2);
        // "c" evicts "a"; the guard must not grow past its capacity.
        assert!(recent.observe("c"));
        assert_eq!(recent.len(), 2);
        // "a" was evicted, so it counts as new again -- acceptable, and bounded.
        assert!(recent.observe("a"));
    }

    #[test]
    fn empty_message_id_is_always_accepted() {
        // An adapter that failed to read an id must not have its messages dropped
        // forever; the guard degrades to "no dedupe" rather than "no messages".
        let mut recent = RecentMessages::new(4);
        assert!(recent.observe(""));
        assert!(recent.observe(""));
        assert!(recent.is_empty());
    }

    #[test]
    fn new_with_zero_capacity_still_works() {
        let mut recent = RecentMessages::new(0);
        assert!(recent.observe("x"));
        assert_eq!(recent.len(), 1);
    }
}
