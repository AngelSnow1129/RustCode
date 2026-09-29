//! `/upgrade` on the row-assembled screen, and the status row's "a newer build
//! is out".
//!
//! **Why this is here now.** `/upgrade` used to be answered with "run
//! `atomcode upgrade`" (`tui_elsewhere::IN_THE_CLI`), on the reasoning that the
//! binary replaces itself and doing it again in the UI only repeats the CLI. The
//! user reversed that on 2026-09-29: the classic screen upgrades in place, with
//! progress, and restarts itself, and a person who moved to this screen should
//! not have to leave it for the same thing. The CLI subcommand stays.
//!
//! **Why a launcher row.** Downloading and replacing a binary, reading the
//! staged-upgrade pointer and restarting the process are all things the screen
//! cannot do (`gates/tui-layers.sh`). The screen draws; this row does. The
//! restart itself happens after the screen has put the terminal back:
//! [`take_restart`] is read by the launcher once `tui_front::run` returns.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use atomcode_config::i18n::{t, Msg};
use atomcode_harness::seams::{UiSvc, UserInterface};
use atomcode_plexus::{Context, Plugin};
use atomcode_tui::command::{Command, CommandSet, Outcome};
use atomcode_tui::plugin::{CommandsSvc, UpdateCheckSvc};
use atomcode_tui::update::UpdateCheck;
use atomcode_updater::UpgradeEvent;
use serde_json::Value;

/// The row's name.
pub const ROW: &str = "tui-upgrade";

/// The command this row answers.
pub const COMMAND: &str = "upgrade";

pub fn row_layer() -> String {
    format!("[[insert]]\nname = \"{ROW}\"\n")
}

/// The binary to start again once the screen has closed, when an upgrade or a
/// rollback asked for it.
static RESTART: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Take the restart an upgrade asked for, if any. The launcher calls this after
/// the screen has closed and re-executes into the path it gets.
pub fn take_restart() -> Option<PathBuf> {
    RESTART.lock().ok()?.take()
}

/// This build's version as the updater writes versions.
fn current() -> String {
    format!("v{}", env!("CARGO_PKG_VERSION"))
}

/// Mounts `/upgrade` and the update check.
pub struct UpgradeRow;

#[async_trait]
impl Plugin for UpgradeRow {
    fn name(&self) -> &'static str {
        ROW
    }
    fn provides(&self) -> &'static [&'static str] {
        &["tui-update-check"]
    }
    fn inject(&self) -> &'static [&'static str] {
        &["tui-commands"]
    }
    fn description(&self) -> &'static str {
        "upgrading this binary in place (`/upgrade`) and saying on the status row when a newer build is out"
    }
    async fn apply(&self, ctx: &Context, _config: &Value) -> Result<(), String> {
        let _ = ctx
            .provide::<UpdateCheckSvc>(Arc::new(Latest))
            .map_err(|e| e.to_string())?;
        let commands = ctx.require::<CommandsSvc>().map_err(|e| e.to_string())?;
        let ui = ctx.require::<UiSvc>().map_err(|e| e.to_string())?;
        commands.add(Arc::new(Upgrade {
            ui,
            running: Arc::new(AtomicBool::new(false)),
        }))?;
        Ok(())
    }
}

/// "Is a newer build out", as the classic screen's footer answers it.
struct Latest;

#[async_trait]
impl UpdateCheck for Latest {
    async fn available(&self) -> Option<String> {
        // A download already staged by an earlier run is news without asking
        // anyone; the network is only asked when there is none, and never when
        // offline.
        let version = match atomcode_updater::read_pending().ok().flatten() {
            Some(pending) => pending.version,
            None if atomcode_config::config::offline::is_offline_active() => return None,
            None => {
                atomcode_updater::fetch_manifest_if_newer(&current())
                    .await
                    .ok()
                    .flatten()?
                    .version
            }
        };
        Some(hint_for(&version, atomcode_updater::is_package_managed()))
    }
}

/// The status row's words for a newer build. A package-managed install is
/// upgraded by its package manager, so it is told that instead of `/upgrade`.
fn hint_for(version: &str, package_managed: bool) -> String {
    if package_managed {
        t(Msg::StatusUpgradeHintPm { version }).into_owned()
    } else {
        t(Msg::StatusUpgradeHint { version }).into_owned()
    }
}

/// What `/upgrade` was asked to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Asked {
    Upgrade { force: bool },
    Rollback,
}

impl Asked {
    /// `None` for an argument it does not take.
    fn of(args: &str) -> Option<Self> {
        match args.trim().to_ascii_lowercase().as_str() {
            "" => Some(Asked::Upgrade { force: false }),
            "--force" | "-f" => Some(Asked::Upgrade { force: true }),
            "rollback" => Some(Asked::Rollback),
            _ => None,
        }
    }
}

/// One step of an upgrade, as the person reads it — the classic screen's words,
/// step for step — and whether it ends in a restart.
///
/// Download progress is said at the quarter marks only, as tuix does: a line per
/// chunk would be hundreds of lines. `last_pct` carries that between events.
fn said_for(event: UpgradeEvent, last_pct: &mut i32) -> (Option<String>, Option<PathBuf>) {
    match event {
        UpgradeEvent::ManifestFetched { version } => {
            *last_pct = -1;
            (
                Some(t(Msg::UpgradeManifestFetched { version: &version }).into_owned()),
                None,
            )
        }
        UpgradeEvent::Downloading { bytes, total } => {
            let pct = if total == 0 {
                0
            } else {
                ((bytes * 100) / total) as i32
            };
            if pct == *last_pct {
                return (None, None);
            }
            *last_pct = pct;
            let said = matches!(pct, 25 | 50 | 75 | 100)
                .then(|| t(Msg::UpgradeDownloading { pct, bytes, total }).into_owned());
            (said, None)
        }
        UpgradeEvent::Verifying => (Some(t(Msg::UpgradeVerifying).into_owned()), None),
        UpgradeEvent::Replacing => (Some(t(Msg::UpgradeReplacing).into_owned()), None),
        UpgradeEvent::Done {
            version,
            backup,
            exe,
        } => (
            Some(
                t(Msg::UpgradeDone {
                    version: &version,
                    backup: &backup.display().to_string(),
                })
                .into_owned(),
            ),
            Some(exe),
        ),
        UpgradeEvent::Failed(message) => (Some(failed(&message)), None),
        UpgradeEvent::RolledBack { exe, backup } => (
            Some(
                t(Msg::UpgradeRolledBack {
                    exe: &exe.display().to_string(),
                    backup: &backup.display().to_string(),
                })
                .into_owned(),
            ),
            Some(exe),
        ),
    }
}

/// A failed upgrade, told apart the way the classic screen tells it apart:
/// "a package manager owns this install" and "already on the latest" are not
/// failures a person should read in red.
fn failed(message: &str) -> String {
    if message.contains(atomcode_updater::PACKAGE_MANAGED) {
        t(Msg::UpgradePackageManaged).into_owned()
    } else if message.contains(atomcode_updater::ALREADY_LATEST) {
        let (current, latest) =
            atomcode_updater::already_latest_versions(message).unwrap_or(("?", "?"));
        t(Msg::UpgradeAlreadyLatest { current, latest }).into_owned()
    } else {
        t(Msg::UpgradeFailed { error: message }).into_owned()
    }
}

struct Upgrade {
    ui: Arc<dyn UserInterface>,
    /// One upgrade at a time: two would race to replace the same file.
    running: Arc<AtomicBool>,
}

#[async_trait]
impl CommandSet for Upgrade {
    fn id(&self) -> &'static str {
        ROW
    }

    fn commands(&self) -> Vec<Command> {
        vec![Command::said_taking(
            COMMAND,
            "[--force | rollback]".into(),
            t(Msg::CmdDescUpgrade),
        )]
    }

    async fn run(&self, name: &str, args: &str, _ctx: &Context) -> Outcome {
        if name != COMMAND {
            return Outcome::Quiet;
        }
        let Some(asked) = Asked::of(args) else {
            return Outcome::Refused(t(Msg::UpgradeUnknownArg { arg: args.trim() }).into_owned());
        };
        if self.running.swap(true, Ordering::SeqCst) {
            return Outcome::Quiet;
        }
        let ui = self.ui.clone();
        let running = self.running.clone();
        match asked {
            Asked::Rollback => {
                let event = match atomcode_updater::run_rollback() {
                    Ok(summary) => UpgradeEvent::RolledBack {
                        exe: summary.exe,
                        backup: summary.backup,
                    },
                    Err(e) => UpgradeEvent::Failed(format!("{e:#}")),
                };
                running.store(false, Ordering::SeqCst);
                let (said, restart) = said_for(event, &mut -1);
                finish(ui.as_ref(), restart);
                match said {
                    Some(said) => Outcome::Said(said),
                    None => Outcome::Quiet,
                }
            }
            Asked::Upgrade { force } => {
                tokio::spawn(async move {
                    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<UpgradeEvent>();
                    let failures = tx.clone();
                    let driver = tokio::spawn(async move {
                        if let Err(e) = atomcode_updater::run_upgrade(current(), force, tx).await {
                            let _ = failures.send(UpgradeEvent::Failed(format!("{e:#}")));
                        }
                    });
                    let mut last_pct = -1;
                    let mut restart = None;
                    while let Some(event) = rx.recv().await {
                        let (said, then) = said_for(event, &mut last_pct);
                        if let Some(said) = said {
                            ui.say(&said);
                        }
                        if then.is_some() {
                            restart = then;
                        }
                    }
                    let _ = driver.await;
                    running.store(false, Ordering::SeqCst);
                    finish(ui.as_ref(), restart);
                });
                Outcome::Said(t(Msg::CmdCheckingUpdate).into_owned())
            }
        }
    }
}

/// A new binary is in place: leave it to the launcher to start it once this
/// screen has closed, and close the screen the way `/quit` does.
fn finish(ui: &dyn UserInterface, restart: Option<PathBuf>) {
    let Some(exe) = restart else {
        return;
    };
    if let Ok(mut slot) = RESTART.lock() {
        *slot = Some(exe);
    }
    ui.run_slash("/quit");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The arguments the classic screen takes, and nothing else.
    #[test]
    fn upgrade_takes_force_and_rollback() {
        assert_eq!(Asked::of(""), Some(Asked::Upgrade { force: false }));
        assert_eq!(Asked::of(" --force "), Some(Asked::Upgrade { force: true }));
        assert_eq!(Asked::of("-f"), Some(Asked::Upgrade { force: true }));
        assert_eq!(Asked::of("ROLLBACK"), Some(Asked::Rollback));
        assert_eq!(Asked::of("now"), None);
    }

    /// Progress at the quarter marks only, each said once; a finished upgrade
    /// or rollback asks for a restart into the binary it names; nothing else
    /// does.
    #[test]
    fn each_step_is_said_once_and_only_a_finished_one_restarts() {
        let mut last = -1;
        let (said, restart) = said_for(
            UpgradeEvent::ManifestFetched {
                version: "v9.9.9".into(),
            },
            &mut last,
        );
        assert!(said.is_some_and(|s| s.contains("v9.9.9")));
        assert!(restart.is_none());

        let mut quarters = 0;
        for bytes in [0u64, 10, 25, 25, 26, 50, 75, 99, 100] {
            let (said, restart) =
                said_for(UpgradeEvent::Downloading { bytes, total: 100 }, &mut last);
            quarters += usize::from(said.is_some());
            assert!(restart.is_none());
        }
        assert_eq!(quarters, 4, "25, 50, 75 and 100 — each once");

        let exe = PathBuf::from("/opt/atomcode/bin/atomcode");
        let (said, restart) = said_for(
            UpgradeEvent::Done {
                version: "v9.9.9".into(),
                backup: PathBuf::from("/opt/atomcode/bin/atomcode.bak"),
                exe: exe.clone(),
            },
            &mut last,
        );
        assert!(said.is_some());
        assert_eq!(restart, Some(exe.clone()));

        let (_, restart) = said_for(
            UpgradeEvent::RolledBack {
                exe: exe.clone(),
                backup: PathBuf::from("/opt/atomcode/bin/atomcode.bak"),
            },
            &mut last,
        );
        assert_eq!(restart, Some(exe));

        let (_, restart) = said_for(UpgradeEvent::Failed("boom".into()), &mut last);
        assert!(restart.is_none());
    }

    /// "Already on the latest" and "a package manager owns this" are told in
    /// their own words, not as a failure.
    #[test]
    fn the_calm_outcomes_are_not_failures() {
        let latest = failed(&format!(
            "{}: already on v5.2.0 (latest is v5.2.0). Pass --force to reinstall.",
            atomcode_updater::ALREADY_LATEST
        ));
        assert_eq!(
            latest,
            t(Msg::UpgradeAlreadyLatest {
                current: "v5.2.0",
                latest: "v5.2.0"
            })
            .into_owned()
        );
        assert_eq!(
            failed(atomcode_updater::PACKAGE_MANAGED),
            t(Msg::UpgradePackageManaged).into_owned()
        );
        assert_eq!(
            failed("disk full"),
            t(Msg::UpgradeFailed { error: "disk full" }).into_owned()
        );
    }

    /// A package-managed install is pointed at its package manager, not `/upgrade`.
    #[test]
    fn the_hint_names_what_will_actually_upgrade_this_install() {
        assert_eq!(
            hint_for("v9.9.9", false),
            t(Msg::StatusUpgradeHint { version: "v9.9.9" }).into_owned()
        );
        assert_eq!(
            hint_for("v9.9.9", true),
            t(Msg::StatusUpgradeHintPm { version: "v9.9.9" }).into_owned()
        );
    }
}
