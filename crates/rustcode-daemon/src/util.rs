//! daemon 内部通用小工具。
//!
//! 目前只有 [`open_browser`]：用系统默认浏览器打开一个 URL 的平台适配。它此前
//! 住在 auth crate 的 `oauth` 模块，但它与 OAuth 毫无关系——只是"打开 URL"这一
//! 通用能力（daemon 用它打开本机 WebUI，见 `lib.rs` 的 webui 启动路径）。把逻辑
//! 落在 daemon 本地后，daemon 不再引用那个即将整体删除的 auth crate。
//!
//! 只依赖 `std::process::Command`，不引入 `open` 之类的第三方 crate。

use anyhow::{Context, Result};

/// 用系统默认浏览器打开 `url`。
///
/// 失败一律返回 `Err`，不静默吞掉：调用方（webui 启动）据此把提示从"已打开浏览器"
/// 降级为"请手动打开该 URL"，而不是拿一个假成功去骗用户。
///
/// 只保证"命令被派生"，不保证浏览器真的起来了（例如 Linux/WSL 上 `xdg-open`
/// 常常静默失败），所以调用方仍需把 URL 显示给用户作为兜底。
#[cfg(target_os = "macos")]
pub fn open_browser(url: &str) -> Result<()> {
    std::process::Command::new("open")
        .arg(url)
        .spawn()
        .context("Failed to open browser")?;
    Ok(())
}

/// 见 [`open_browser`]。Linux 上 `xdg-open` 会把自身诊断写到 stderr，而 WebUI 的
/// URL 已经打印给用户了，这里显式丢弃以免污染 daemon 输出。
#[cfg(target_os = "linux")]
pub fn open_browser(url: &str) -> Result<()> {
    std::process::Command::new("xdg-open")
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Failed to open browser")?;
    Ok(())
}

/// 见 [`open_browser`]。
///
/// `cmd /C start "" "<url>"`：`start` 把第一个带引号的参数当成窗口标题，所以先给
/// 一个空标题再把 URL 整体加引号；加引号后 `&`（WebUI 的 `&sync=1`）落在 cmd 的
/// 引号内，不会被当成命令分隔符。失败（罕见：没有 cmd / 被策略禁用）时退回
/// `explorer <url>`。
#[cfg(target_os = "windows")]
pub fn open_browser(url: &str) -> Result<()> {
    use std::os::windows::process::CommandExt;

    if std::process::Command::new("cmd")
        .raw_arg(format!("/C start \"\" \"{}\"", url))
        .spawn()
        .is_ok()
    {
        return Ok(());
    }
    std::process::Command::new("explorer")
        .arg(url)
        .spawn()
        .context("Failed to open browser")?;
    Ok(())
}

/// 见 [`open_browser`]。未知平台没有已知的打开方式，显式报错而不是假装成功。
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn open_browser(_url: &str) -> Result<()> {
    anyhow::bail!("Unsupported platform for browser auto-open");
}
