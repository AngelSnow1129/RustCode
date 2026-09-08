//! `2026-09-07-zh-docs-webui` 文案锁定测试（AC-17a / AC-17b / AC-17c / AC-17d + T-05 K9 / K10）。
//!
//! 背景：`01-design.md §4.4` 冻结了 `Msg::CliWebuiNotBuilt` 的精确文案；`02-tasks.md`
//! AC-17d 要求「中英文案中的命令行逐条相等」。T-05 落地时该结论由**人工比对表**给出，
//! 仓库里没有任何测试守护它 —— 任一语种被单独修改（或漏改）都会静默破坏双语种同构，
//! 用户按文案抄命令时才会暴露。本文件把这个不变量钉成可执行判定。
//!
//! 同理，T-03 的未验证项 U-1 记录过「`--help` 描述文本仍写 `127.0.0.1` 而 clap 默认值
//! 已是 `0.0.0.0`」的自相矛盾输出；T-05 的 K9/K10 修掉了它，本文件锁定该修复不被回退。
//!
//! 只读测试：一律走 `i18n::t_with(locale, msg)`，不触碰全局 locale，因此不需要
//! `i18n::test_lock()`，也不会与并发的 locale 测试互相干扰。

use rustcode_config::i18n::{t_with, Locale, Msg};

/// 文案里以 3 空格缩进引导的行即「命令行」（`01-design.md §4.4` 冻结的排版）。
fn command_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| line.starts_with("   ") && !line.trim().is_empty())
        .map(str::trim)
        .collect()
}

fn not_built(locale: Locale) -> String {
    t_with(locale, Msg::CliWebuiNotBuilt).into_owned()
}

fn help_host(locale: Locale) -> String {
    t_with(locale, Msg::CliHelpHost).into_owned()
}

/// AC-17a / AC-17b / AC-17c：两语种都必须指向新脚本、给出 `cargo clean`，
/// 且手工路径用 `npm ci`（`AGENTS.md:16` 口径），不得退回 `npm install`。
#[test]
fn cli_webui_not_built_guides_are_executable_in_both_locales() {
    for (locale, label) in [(Locale::ZhCn, "zh_CN"), (Locale::En, "en")] {
        let text = not_built(locale);
        assert!(
            text.contains("./scripts/build-webui.sh"),
            "[{label}] `Msg::CliWebuiNotBuilt` 必须指向 `./scripts/build-webui.sh`（AC-17a），实际：\n{text}"
        );
        assert!(
            text.contains("cargo clean -p rustcode-daemon"),
            "[{label}] `Msg::CliWebuiNotBuilt` 必须含 `cargo clean -p rustcode-daemon`（AC-17b），实际：\n{text}"
        );
        assert!(
            text.contains("npm ci"),
            "[{label}] 手工路径必须是 `npm ci`（AC-17c / AGENTS.md:16），实际：\n{text}"
        );
        assert!(
            !text.contains("npm install"),
            "[{label}] 手工路径不得再出现 `npm install`（已被 `npm ci` 取代），实际：\n{text}"
        );
    }
}

/// AC-17d：中英文案的命令行必须逐条相等（散文部分按语种自然不同，不计入）。
#[test]
fn cli_webui_not_built_command_lines_match_across_locales() {
    let zh_text = not_built(Locale::ZhCn);
    let en_text = not_built(Locale::En);
    let zh = command_lines(&zh_text);
    let en = command_lines(&en_text);

    assert!(
        !zh.is_empty(),
        "中文文案未解析出任何命令行，AC-17d 的比对本身失效（缩进排版被改动？）"
    );
    assert_eq!(
        zh, en,
        "AC-17d：`Msg::CliWebuiNotBuilt` 的中英文案命令行必须逐条相等；zh={zh:?} en={en:?}"
    );
    assert_eq!(
        zh,
        vec![
            "./scripts/build-webui.sh",
            "cargo clean -p rustcode-daemon",
            "cargo build -p rustcode",
            "cd webui && npm ci && npm run build",
            "cargo clean -p rustcode-daemon",
            "cargo build -p rustcode",
        ],
        "`Msg::CliWebuiNotBuilt` 的命令行集合与 `01-design.md §4.4` 冻结文本不一致"
    );
}

/// T-05 K9 / K10：`--host` 帮助文案必须体现新默认 `0.0.0.0`，
/// 不得回退成「默认值写 127.0.0.1」与 clap 默认值自相矛盾的输出（T-03 U-1）。
#[test]
fn cli_help_host_states_all_interfaces_default_in_both_locales() {
    for (locale, label) in [(Locale::ZhCn, "zh_CN"), (Locale::En, "en")] {
        let text = help_host(locale);
        assert!(
            text.contains("0.0.0.0"),
            "[{label}] `Msg::CliHelpHost` 必须体现新默认 `0.0.0.0`，实际：{text}"
        );
        assert!(
            !text.contains("（默认：127.0.0.1）") && !text.contains("(default: 127.0.0.1)"),
            "[{label}] `Msg::CliHelpHost` 不得再把默认值写成 `127.0.0.1`，实际：{text}"
        );
    }
}
