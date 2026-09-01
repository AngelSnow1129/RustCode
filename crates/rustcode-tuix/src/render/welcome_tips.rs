//! Curated, usage-informed pool of "getting started" tips for the welcome banner.
//! One tip is pinned first (BYO `/provider` in a neutral build, `/login` in a
//! managed distribution that ships sign-in); 3 more are chosen at random from
//! `POOL`, with the pinned command excluded so it never repeats.
//! The pool is a hand-edited const, refreshed per release from the usage dashboard.

use crate::i18n::Msg;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tip {
    pub cmd: &'static str,
    pub desc: Msg<'static>,
}

/// Managed sign-in tip -- only meaningful when a platform server is configured.
pub const LOGIN_TIP: Tip = Tip {
    cmd: "/login",
    desc: Msg::WelcomeTipLogin,
};

/// The pinned first tip for the running build. `/login` (managed sign-in) leads when a
/// platform server is configured; otherwise `/provider` leads, because a neutral build
/// ships no managed service and `/login` fail-closes -- so the bring-your-own-key setup
/// is the real first step and must not be hidden behind a dead-end sign-in tip.
fn pinned() -> Tip {
    // Single-source the predicate: the onboarding wizard delegates to
    // `rustcode_auth::managed_login_available()` (RAW server value, trimmed).
    // Reading `platform_server()` directly here drifted from that (no trim);
    // the divergence was unreachable but defense-in-depth wants one gate.
    if crate::modals::onboarding_wizard::managed_login_available() {
        LOGIN_TIP
    } else {
        Tip {
            cmd: "/provider",
            desc: Msg::WelcomeTipProvider,
        }
    }
}

/// Random pool (15). Filtered to onboarding-relevant commands; excludes
/// exit/clear/destructive and pure-utility commands. Edit + recompile to refresh.
pub const POOL: &[Tip] = &[
    Tip {
        cmd: "/provider",
        desc: Msg::WelcomeTipProvider,
    },
    Tip {
        cmd: "/model",
        desc: Msg::WelcomeTipModel,
    },
    Tip {
        cmd: "/resume",
        desc: Msg::WelcomeTipResume,
    },
    Tip {
        cmd: "/setup",
        desc: Msg::WelcomeTipSetup,
    },
    Tip {
        cmd: "/skills",
        desc: Msg::WelcomeTipSkills,
    },
    Tip {
        cmd: "/plugin",
        desc: Msg::WelcomeTipPlugin,
    },
    Tip {
        cmd: "/webui",
        desc: Msg::WelcomeTipWebui,
    },
    Tip {
        cmd: "/mcp",
        desc: Msg::WelcomeTipMcp,
    },
    Tip {
        cmd: "/plan",
        desc: Msg::WelcomeTipPlan,
    },
    Tip {
        cmd: "/session",
        desc: Msg::WelcomeTipSession,
    },
    Tip {
        cmd: "/loop",
        desc: Msg::WelcomeTipLoop,
    },
    Tip {
        cmd: "/goal",
        desc: Msg::WelcomeTipGoal,
    },
    Tip {
        cmd: "/init",
        desc: Msg::WelcomeTipInit,
    },
    Tip {
        cmd: "/language",
        desc: Msg::WelcomeTipLanguage,
    },
    Tip {
        cmd: "/usage",
        desc: Msg::WelcomeTipUsage,
    },
];

/// Tips for commands that only exist in managed distributions. These commands
/// are hidden from every discovery surface in a neutral build (see
/// `commands::command_visible`), so advertising them as tips would be a
/// dead-end: the typed command answers with a "not available in this build"
/// notice. Filtered from the random pool (and defensively from cached indices)
/// unless the running build ships managed sign-in.
const MANAGED_ONLY_TIP_CMDS: &[&str] = &["/usage"];

fn tip_allowed(tip: &Tip) -> bool {
    crate::modals::onboarding_wizard::managed_login_available()
        || !MANAGED_ONLY_TIP_CMDS.contains(&tip.cmd)
}

/// How many random tips to show below the pinned one.
const RANDOM_COUNT: usize = 3;

/// Pick `RANDOM_COUNT` distinct POOL indices (for caching a stable selection),
/// excluding the pinned command so it is never duplicated by a random pick.
pub fn choose_pool_indices(rng: &mut impl Rng) -> Vec<usize> {
    let pin_cmd = pinned().cmd;
    let mut idx: Vec<usize> = (0..POOL.len())
        .filter(|&i| POOL[i].cmd != pin_cmd && tip_allowed(&POOL[i]))
        .collect();
    idx.shuffle(rng);
    idx.truncate(RANDOM_COUNT.min(idx.len()));
    idx
}

/// Resolve cached indices back to `[pinned, ...selected]`.
pub fn tips_from_indices(indices: &[usize]) -> Vec<Tip> {
    let pin = pinned();
    let mut out = Vec::with_capacity(1 + indices.len());
    out.push(pin);
    for &i in indices {
        if let Some(t) = POOL.get(i) {
            if t.cmd == pin.cmd {
                continue; // never duplicate the pinned tip
            }
            if !tip_allowed(t) {
                continue; // managed-only tip cached by a different build kind
            }
            out.push(*t);
        }
    }
    out
}

/// `[PINNED, r1, r2, r3]` -- pinned first, then up to 3 distinct random picks.
pub fn choose_tips(rng: &mut impl Rng) -> Vec<Tip> {
    tips_from_indices(&choose_pool_indices(rng))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    fn fixed(seed: u64) -> StdRng {
        StdRng::seed_from_u64(seed)
    }

    #[test]
    fn pinned_is_first_and_byo_in_neutral_build() {
        // No platform server is configured in tests -> the neutral build must lead
        // with the bring-your-own-key /provider step, not the dead-end /login tip.
        let t = choose_tips(&mut fixed(1));
        assert_eq!(t[0], pinned());
        assert_eq!(t[0].cmd, "/provider");
    }

    #[test]
    fn login_tip_is_managed_only() {
        // The /login tip exists for managed builds but is never the neutral default.
        assert_eq!(LOGIN_TIP.cmd, "/login");
        assert_ne!(pinned().cmd, "/login");
    }

    #[test]
    fn returns_exactly_four() {
        assert_eq!(choose_tips(&mut fixed(1)).len(), 4);
    }

    #[test]
    fn random_three_are_distinct_and_not_pinned() {
        let t = choose_tips(&mut fixed(7));
        let pin_cmd = t[0].cmd;
        let rest = &t[1..];
        for w in rest {
            assert_ne!(w.cmd, pin_cmd, "random pick duplicates the pinned tip");
        }
        for i in 0..rest.len() {
            for j in (i + 1)..rest.len() {
                assert_ne!(rest[i].cmd, rest[j].cmd, "duplicate random tip");
            }
        }
    }

    #[test]
    fn deterministic_for_same_seed() {
        let a: Vec<_> = choose_tips(&mut fixed(42)).iter().map(|t| t.cmd).collect();
        let b: Vec<_> = choose_tips(&mut fixed(42)).iter().map(|t| t.cmd).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn pool_excludes_filtered_commands() {
        let banned = [
            "/quit",
            "/clear",
            "/status",
            "/cd",
            "/logout",
            "/delete_session",
            "/undo",
            "/stop",
            "/whoami",
            "/cost",
        ];
        for t in POOL {
            assert!(!banned.contains(&t.cmd), "{} must not be in POOL", t.cmd);
        }
    }

    #[test]
    fn managed_only_tips_never_surface_in_neutral_build() {
        // Tests run with no platform server -> neutral build. /usage is a
        // managed-account command (hidden from discovery; answers with a
        // "not available in this build" notice), so it must never be shown as
        // a tip -- neither fresh-picked nor resolved through cached indices.
        for seed in 0..64u64 {
            let tips = choose_tips(&mut fixed(seed));
            for tip in &tips {
                assert_ne!(
                    tip.cmd, "/usage",
                    "neutral build surfaced /usage tip (seed {seed})"
                );
            }
            let indices = choose_pool_indices(&mut fixed(seed));
            for tip in tips_from_indices(&indices) {
                assert_ne!(
                    tip.cmd, "/usage",
                    "neutral build resolved /usage tip (seed {seed})"
                );
            }
        }
    }
}
