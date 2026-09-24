//! OS service registration for `rustcode im serve` (long-running channel).
//!
//! Sibling of `schedule_os.rs` but a DIFFERENT shape, and the difference is the
//! point: scheduled tasks are one-shot invocations the OS re-fires on a clock,
//! while an IM channel is a **long-lived WebSocket connection**. Registering it
//! as a timer would spawn a duplicate serve process every tick, each fighting
//! over the same platform connection -- so every backend here uses the
//! *service* shape instead: start at login, keep it alive, restart on failure.
//!
//! ```text
//! launchd  (macOS)      ~/Library/LaunchAgents/com.rustcode.im.<platform>.plist
//!                       KeepAlive=true + RunAtLoad=true
//! systemd  (Linux)      ~/.config/systemd/user/rustcode-im-<platform>.service
//!                       Restart=on-failure + RestartSec=5, enabled --now
//! schtasks (Windows)    rustcode\im\<platform>, /SC ONLOGON
//!                       (Task Scheduler has no restart-on-crash; a crash keeps
//!                        the channel down until the next logon -- documented,
//!                        not hidden)
//! ```
//!
//! The service name is keyed by **platform**, not a hash: platform spellings
//! are minted by us (lowercase alphabet, validated by
//! `rustcode_config::im_store`), so they are filesystem-safe AND readable --
//! a typo shows up as an obviously-wrong unit name rather than an opaque
//! digest. One service per platform serves that platform's first active
//! channel, matching the `rustcode im serve --platform <p>` selection rule.
//!
//! Reuses `schedule_os.rs`'s command-running and escaping helpers (single
//! implementation); only the unit rendering differs.

use std::path::PathBuf;
use std::sync::Arc;

use rustcode_config::i18n::{t, Msg};

use crate::schedule_os::{run_checked, CommandRunner, InstallState, RealCommandRunner};
// These two helpers are macOS-gated in schedule_os (they only matter for the
// plist backend), so the import must carry the same gate.
#[cfg(any(test, target_os = "macos"))]
use crate::schedule_os::{get_uid, xml_escape};

/// One IM service registration: serve `--platform <p>` bound to `--project`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImServiceSpec {
    /// Validated platform spelling (`dingtalk` / `feishu` / `wecom`).
    pub platform: String,
    /// Absolute project path the channel drives (passed through to `serve`).
    pub project: String,
}

/// Validates the platform alphabet up front: these strings become file names
/// and unit names, so a hostile spelling must never reach the filesystem.
fn valid_platform(platform: &str) -> bool {
    !platform.is_empty()
        && platform.len() <= 32
        && platform
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// Validate a platform spelling for service-name use.
///
/// `pub`: the CLI `im register` handler runs this before touching the
/// registrar, so a bad spelling fails before any filesystem effect.
pub fn checked_platform(platform: &str) -> anyhow::Result<String> {
    if valid_platform(platform) {
        Ok(platform.to_string())
    } else {
        Err(anyhow::anyhow!(
            "{}",
            t(Msg::ImAdminUnknownPlatform { platform }).into_owned()
        ))
    }
}

// ---- Registrar trait ----

pub trait ImServiceRegistrar {
    fn install(&self, spec: &ImServiceSpec) -> anyhow::Result<()>;
    fn uninstall(&self, platform: &str) -> anyhow::Result<()>;
    fn status(&self, platform: &str) -> InstallState;
}

// ---- launchd (macOS) ----

#[cfg(any(test, target_os = "macos"))]
pub struct LaunchdIm {
    pub root: PathBuf,
    pub runner: Arc<dyn CommandRunner + Send + Sync>,
}

#[cfg(any(test, target_os = "macos"))]
impl LaunchdIm {
    fn label(platform: &str) -> String {
        format!("com.rustcode.im.{platform}")
    }

    fn plist_path(&self, platform: &str) -> PathBuf {
        self.root.join(format!("{}.plist", Self::label(platform)))
    }

    /// Renders the plist for one IM channel service.
    ///
    /// `KeepAlive=true` restarts the channel whenever it exits (clean or
    /// crashed) and `RunAtLoad=true` starts it at login -- together they give
    /// the always-on behaviour a long connection needs. `RunAtLoad` replaces
    /// the schedule template's explicit `<false/>`: a channel the user asked
    /// to register should come up with the session, not wait to be poked.
    fn render_plist(spec: &ImServiceSpec, exe: &str) -> String {
        let label = xml_escape(&Self::label(&spec.platform));
        let exe = xml_escape(exe);
        let platform = xml_escape(&spec.platform);
        let project = xml_escape(&spec.project);
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>{label}</string>
	<key>ProgramArguments</key>
	<array>
		<string>{exe}</string>
		<string>im</string>
		<string>serve</string>
		<string>--platform</string>
		<string>{platform}</string>
		<string>--project</string>
		<string>{project}</string>
	</array>
	<key>KeepAlive</key>
	<true/>
	<key>RunAtLoad</key>
	<true/>
</dict>
</plist>
"#
        )
    }
}

#[cfg(any(test, target_os = "macos"))]
impl ImServiceRegistrar for LaunchdIm {
    fn install(&self, spec: &ImServiceSpec) -> anyhow::Result<()> {
        #[cfg(not(unix))]
        {
            anyhow::bail!("launchd is only available on macOS/Unix");
        }
        let exe = std::env::current_exe()?;
        let exe_str = exe.to_string_lossy();
        std::fs::create_dir_all(&self.root)?;
        let path = self.plist_path(&spec.platform);
        std::fs::write(&path, Self::render_plist(spec, &exe_str))?;
        let uid = get_uid();
        let label = Self::label(&spec.platform);
        // Best-effort bootout first (same idempotency trick as schedule_os):
        // booting an already-loaded label fails, and that failure is expected.
        let _ = self.runner.run(
            "launchctl",
            &["bootout".to_string(), format!("gui/{uid}/{label}")],
        );
        run_checked(
            self.runner.as_ref(),
            "launchctl",
            &[
                "bootstrap".to_string(),
                format!("gui/{uid}"),
                path.to_string_lossy().to_string(),
            ],
        )?;
        Ok(())
    }

    fn uninstall(&self, platform: &str) -> anyhow::Result<()> {
        let uid = get_uid();
        let domain_target = format!("gui/{uid}/{}", Self::label(platform));
        let _ = self
            .runner
            .run("launchctl", &["bootout".to_string(), domain_target]);
        let path = self.plist_path(platform);
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }

    fn status(&self, platform: &str) -> InstallState {
        if self.plist_path(platform).exists() {
            InstallState::Installed
        } else {
            InstallState::Missing
        }
    }
}

// ---- systemd (Linux) ----

#[cfg(any(test, not(any(target_os = "macos", target_os = "windows"))))]
pub struct SystemdIm {
    pub root: PathBuf,
    pub runner: Arc<dyn CommandRunner + Send + Sync>,
}

#[cfg(any(test, not(any(target_os = "macos", target_os = "windows"))))]
impl SystemdIm {
    fn service_name(platform: &str) -> String {
        format!("rustcode-im-{platform}.service")
    }

    fn service_path(&self, platform: &str) -> PathBuf {
        self.root.join(Self::service_name(platform))
    }

    /// Renders the user service for one IM channel.
    ///
    /// `Restart=on-failure` + a short backoff keeps a flaky long connection
    /// coming back without masking a persistent misconfiguration as a crash
    /// loop (a *clean* exit stays down -- an operator-disabled channel should
    /// not resurrect itself). `WantedBy=default.target` is the user-service
    /// equivalent of "start at login"; `timers.target` would be wrong here.
    fn render_service(spec: &ImServiceSpec, exe: &str) -> String {
        format!(
            "[Unit]\nDescription=Rustcode IM channel: {platform}\n\n\
             [Service]\nType=simple\nExecStart=\"{exe}\" im serve --platform {platform} --project {project}\n\
             Restart=on-failure\nRestartSec=5\n\n\
             [Install]\nWantedBy=default.target\n",
            platform = spec.platform,
            exe = exe,
            project = spec.project,
        )
    }
}

#[cfg(any(test, not(any(target_os = "macos", target_os = "windows"))))]
impl ImServiceRegistrar for SystemdIm {
    fn install(&self, spec: &ImServiceSpec) -> anyhow::Result<()> {
        let exe = std::env::current_exe()?;
        let exe_str = exe.to_string_lossy();
        std::fs::create_dir_all(&self.root)?;
        std::fs::write(
            self.service_path(&spec.platform),
            Self::render_service(spec, &exe_str),
        )?;
        run_checked(
            self.runner.as_ref(),
            "systemctl",
            &["--user".to_string(), "daemon-reload".to_string()],
        )?;
        run_checked(
            self.runner.as_ref(),
            "systemctl",
            &[
                "--user".to_string(),
                "enable".to_string(),
                "--now".to_string(),
                Self::service_name(&spec.platform),
            ],
        )?;
        Ok(())
    }

    fn uninstall(&self, platform: &str) -> anyhow::Result<()> {
        let _ = self.runner.run(
            "systemctl",
            &[
                "--user".to_string(),
                "disable".to_string(),
                "--now".to_string(),
                Self::service_name(platform),
            ],
        );
        match std::fs::remove_file(self.service_path(platform)) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        let _ = self.runner.run(
            "systemctl",
            &["--user".to_string(), "daemon-reload".to_string()],
        );
        Ok(())
    }

    fn status(&self, platform: &str) -> InstallState {
        if self.service_path(platform).exists() {
            InstallState::Installed
        } else {
            InstallState::Missing
        }
    }
}

// ---- TaskSched (Windows) ----

#[cfg(any(test, target_os = "windows"))]
pub struct TaskSchedIm {
    pub runner: Arc<dyn CommandRunner + Send + Sync>,
}

#[cfg(any(test, target_os = "windows"))]
impl TaskSchedIm {
    fn task_name(platform: &str) -> String {
        format!("rustcode\\im\\{platform}")
    }
}

#[cfg(any(test, target_os = "windows"))]
impl ImServiceRegistrar for TaskSchedIm {
    fn install(&self, spec: &ImServiceSpec) -> anyhow::Result<()> {
        let exe = std::env::current_exe()?;
        let exe_str = exe.to_string_lossy();
        let tr = format!(
            "\"{}\" im serve --platform {} --project {}",
            exe_str, spec.platform, spec.project
        );
        // /SC ONLOGON starts the channel at logon. Task Scheduler has no
        // restart-on-crash trigger, so crash recovery waits for the next
        // logon -- weaker than the Unix backends and stated plainly here
        // rather than papered over.
        run_checked(
            self.runner.as_ref(),
            "schtasks",
            &[
                "/Create".to_string(),
                "/F".to_string(),
                "/TN".to_string(),
                Self::task_name(&spec.platform),
                "/TR".to_string(),
                tr,
                "/SC".to_string(),
                "ONLOGON".to_string(),
            ],
        )?;
        Ok(())
    }

    fn uninstall(&self, platform: &str) -> anyhow::Result<()> {
        let _ = self.runner.run(
            "schtasks",
            &[
                "/Delete".to_string(),
                "/F".to_string(),
                "/TN".to_string(),
                Self::task_name(platform),
            ],
        );
        Ok(())
    }

    fn status(&self, platform: &str) -> InstallState {
        let result = self.runner.run(
            "schtasks",
            &[
                "/Query".to_string(),
                "/TN".to_string(),
                Self::task_name(platform),
            ],
        );
        match result {
            Ok(out) if out.status.success() => InstallState::Installed,
            _ => InstallState::Missing,
        }
    }
}

// ---- current() -- platform selector (same cfg split as schedule_os) ----

/// Build the registrar for the host platform, writing under the real user
/// config locations. Tests construct the backend structs directly with a
/// tempdir root and a fake runner instead of going through here.
pub fn current() -> anyhow::Result<Box<dyn ImServiceRegistrar + Send + Sync>> {
    im_service_platform(Arc::new(RealCommandRunner))
}

#[cfg(target_os = "macos")]
fn im_service_platform(
    runner: Arc<dyn CommandRunner + Send + Sync>,
) -> anyhow::Result<Box<dyn ImServiceRegistrar + Send + Sync>> {
    let root = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?
        .join("Library/LaunchAgents");
    Ok(Box::new(LaunchdIm { root, runner }))
}

#[cfg(target_os = "linux")]
fn im_service_platform(
    runner: Arc<dyn CommandRunner + Send + Sync>,
) -> anyhow::Result<Box<dyn ImServiceRegistrar + Send + Sync>> {
    let root = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?
        .join(".config/systemd/user");
    Ok(Box::new(SystemdIm { root, runner }))
}

#[cfg(target_os = "windows")]
fn im_service_platform(
    runner: Arc<dyn CommandRunner + Send + Sync>,
) -> anyhow::Result<Box<dyn ImServiceRegistrar + Send + Sync>> {
    Ok(Box::new(TaskSchedIm { runner }))
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn im_service_platform(
    runner: Arc<dyn CommandRunner + Send + Sync>,
) -> anyhow::Result<Box<dyn ImServiceRegistrar + Send + Sync>> {
    // Fallback mirrors schedule_os: systemd user units as best-effort.
    let root = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?
        .join(".config/systemd/user");
    Ok(Box::new(SystemdIm { root, runner }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Local fake runner (schedule_os's lives inside its own test module and is
    /// not importable). Records each invocation as one joined string so
    /// assertions can grep for program and argument fragments alike.
    struct FakeRunner {
        calls: std::sync::Mutex<Vec<String>>,
    }

    impl FakeRunner {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                calls: std::sync::Mutex::new(Vec::new()),
            })
        }

        fn calls(&self) -> Vec<String> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl CommandRunner for FakeRunner {
        fn run(&self, p: &str, a: &[String]) -> std::io::Result<std::process::Output> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{p} {}", a.join(" ")));
            Ok(std::process::Output {
                status: Default::default(),
                stdout: vec![],
                stderr: vec![],
            })
        }
    }

    fn checked(platform: &str) -> ImServiceSpec {
        let platform = checked_platform(platform).unwrap();
        ImServiceSpec {
            platform,
            project: "/tmp/proj".to_string(),
        }
    }

    #[test]
    fn platform_names_are_validated_before_they_become_file_names() {
        // These strings land in unit/plist/task names -- a traversal or
        // mixed-case spelling must be rejected at the door.
        for bad in ["../evil", "DingTalk", "ding talk", "", "a/b"] {
            assert!(checked_platform(bad).is_err(), "{bad:?} must be rejected");
        }
        assert_eq!(checked_platform("dingtalk").unwrap(), "dingtalk");
        assert_eq!(checked_platform("feishu_2").unwrap(), "feishu_2");
    }

    #[cfg(any(test, not(any(target_os = "macos", target_os = "windows"))))]
    #[test]
    fn systemd_service_is_long_running_not_oneshot() {
        // The whole point of this module: a timer/oneshot shape would respawn
        // duplicate serve processes. The unit must be a simple long-running
        // service with restart-on-failure.
        let rendered = SystemdIm::render_service(&checked("dingtalk"), "/usr/bin/rustcode");
        assert!(rendered.contains("Type=simple"), "{rendered}");
        assert!(rendered.contains("Restart=on-failure"));
        assert!(rendered.contains("RestartSec=5"));
        assert!(rendered.contains("WantedBy=default.target"), "{rendered}");
        assert!(!rendered.contains("Type=oneshot"), "{rendered}");
        assert!(!rendered.contains("OnCalendar"), "{rendered}");
        // Exec line threads BOTH filters so the unit serves the right channel.
        assert!(
            rendered.contains(
                "ExecStart=\"/usr/bin/rustcode\" im serve --platform dingtalk --project /tmp/proj"
            ),
            "{rendered}"
        );
    }

    #[cfg(any(test, not(any(target_os = "macos", target_os = "windows"))))]
    #[test]
    fn systemd_install_writes_unit_and_enables_now() {
        let tmp = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        let svc = SystemdIm {
            root: tmp.path().to_path_buf(),
            runner: fake.clone(),
        };
        svc.install(&checked("dingtalk")).unwrap();

        let unit_path = tmp.path().join("rustcode-im-dingtalk.service");
        let unit = std::fs::read_to_string(&unit_path).unwrap();
        assert!(unit.contains("im serve --platform dingtalk"));

        let calls = fake.calls();
        assert!(
            calls.iter().any(|c| c.contains("daemon-reload")),
            "{calls:?}"
        );
        assert!(
            calls.iter().any(|c| c.contains("enable")
                && c.contains("--now")
                && c.contains("rustcode-im-dingtalk.service")),
            "{calls:?}"
        );
    }

    #[cfg(any(test, not(any(target_os = "macos", target_os = "windows"))))]
    #[test]
    fn systemd_uninstall_disables_and_removes_the_unit() {
        let tmp = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        let svc = SystemdIm {
            root: tmp.path().to_path_buf(),
            runner: fake.clone(),
        };
        svc.install(&checked("feishu")).unwrap();
        svc.uninstall("feishu").unwrap();

        assert!(!tmp.path().join("rustcode-im-feishu.service").exists());
        let calls = fake.calls();
        assert!(
            calls
                .iter()
                .any(|c| c.contains("disable") && c.contains("rustcode-im-feishu.service")),
            "{calls:?}"
        );
        // Uninstalling an absent unit is not an error (idempotent).
        svc.uninstall("feishu").unwrap();
    }

    #[cfg(any(test, not(any(target_os = "macos", target_os = "windows"))))]
    #[test]
    fn systemd_status_tracks_the_unit_file() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = SystemdIm {
            root: tmp.path().to_path_buf(),
            runner: FakeRunner::new(),
        };
        assert_eq!(svc.status("dingtalk"), InstallState::Missing);
        svc.install(&checked("dingtalk")).unwrap();
        assert_eq!(svc.status("dingtalk"), InstallState::Installed);
    }

    #[cfg(any(test, target_os = "macos"))]
    #[test]
    fn launchd_plist_keeps_the_channel_alive() {
        let rendered = LaunchdIm::render_plist(&checked("dingtalk"), "/Applications/rustcode");
        assert!(rendered.contains("<key>KeepAlive</key>"), "{rendered}");
        assert!(rendered.contains("<key>RunAtLoad</key>"), "{rendered}");
        // Arguments are threaded so the service serves the right channel.
        assert!(rendered.contains("<string>im</string>"), "{rendered}");
        assert!(rendered.contains("<string>serve</string>"), "{rendered}");
        assert!(
            rendered.contains("<string>--platform</string>"),
            "{rendered}"
        );
        assert!(rendered.contains("<string>dingtalk</string>"), "{rendered}");
        assert!(
            rendered.contains("<string>/tmp/proj</string>"),
            "{rendered}"
        );
        // Values are XML-escaped (the schedule template's invariant).
        let evil = ImServiceSpec {
            platform: checked_platform("dingtalk").unwrap(),
            project: "/tmp/a&b<c>".to_string(),
        };
        let rendered = LaunchdIm::render_plist(&evil, "/x");
        assert!(rendered.contains("&amp;"), "{rendered}");
        assert!(rendered.contains("&lt;"), "{rendered}");
    }

    #[cfg(any(test, target_os = "macos"))]
    #[test]
    fn launchd_install_bootstraps_after_a_best_effort_bootout() {
        let tmp = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        let svc = LaunchdIm {
            root: tmp.path().to_path_buf(),
            runner: fake.clone(),
        };
        svc.install(&checked("wecom")).unwrap();

        assert!(tmp.path().join("com.rustcode.im.wecom.plist").exists());
        let calls = fake.calls();
        // bootout is attempted (and its failure ignored) before bootstrap.
        let bootout = calls.iter().position(|c| c.contains("bootout")).unwrap();
        let bootstrap = calls.iter().position(|c| c.contains("bootstrap")).unwrap();
        assert!(bootout < bootstrap, "{calls:?}");
    }

    #[cfg(any(test, target_os = "macos"))]
    #[test]
    fn launchd_status_tracks_the_plist_file() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = LaunchdIm {
            root: tmp.path().to_path_buf(),
            runner: FakeRunner::new(),
        };
        assert_eq!(svc.status("dingtalk"), InstallState::Missing);
        svc.install(&checked("dingtalk")).unwrap();
        assert_eq!(svc.status("dingtalk"), InstallState::Installed);
        svc.uninstall("dingtalk").unwrap();
        assert_eq!(svc.status("dingtalk"), InstallState::Missing);
    }

    #[cfg(any(test, target_os = "windows"))]
    #[test]
    fn tasksched_install_creates_an_onlogon_task_with_the_serve_line() {
        let fake = FakeRunner::new();
        let svc = TaskSchedIm {
            runner: fake.clone(),
        };
        svc.install(&checked("dingtalk")).unwrap();

        let calls = fake.calls();
        let create = calls
            .iter()
            .find(|c| c.contains("/Create"))
            .expect("schtasks /Create must be called");
        assert!(create.contains("rustcode\\im\\dingtalk"), "{create}");
        assert!(create.contains("im serve --platform dingtalk"), "{create}");
        assert!(create.contains("ONLOGON"), "{create}");
    }

    #[cfg(any(test, target_os = "windows"))]
    #[test]
    fn tasksched_uninstall_deletes_the_task() {
        let fake = FakeRunner::new();
        let svc = TaskSchedIm {
            runner: fake.clone(),
        };
        svc.uninstall("dingtalk").unwrap();
        let calls = fake.calls();
        assert!(
            calls
                .iter()
                .any(|c| c.contains("/Delete") && c.contains("rustcode\\im\\dingtalk")),
            "{calls:?}"
        );
    }
}
