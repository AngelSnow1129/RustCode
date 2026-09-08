//! T-03 / AC-30 锁定测试：独立 `rustcode-daemon` 二进制的默认绑定地址。
//!
//! 安全背景（已冻结裁决 Q2，不得回退）：独立二进制由 VS Code / JetBrains 插件在
//! **不传 `--host`** 的情况下**无人值守**拉起。它虽带 token（`src/main.rs` 传入
//! `webui_tokens: Some(token_store)` ⇒ `enforce_token = true`，token 落盘于
//! `~/.rustcode/daemon-<port>.json`，任何能读到该文件的本地进程都可取用），但进程
//! 由 IDE 在用户无感知时启动、**没有审批交互方**。一旦默认地址被改成 `0.0.0.0`，
//! 该 token 会随监听面一起暴露到局域网，风险从「本机任意进程」放大到「同网段任意设备」。
//!
//! `DEFAULT_HOST` 是 `src/main.rs` 中 `fn main()` 内的局部常量，二进制 crate 无法被
//! 集成测试 `use`，因此这里对源码文本做断言：谁改这个常量，谁就必须同时改这个测试，
//! 让改动在 review 阶段暴露出来，而不是等插件用户被暴露到局域网才发现。

use std::path::PathBuf;

/// 独立二进制必须保持的默认绑定地址（回环）。
const EXPECTED_DEFAULT_HOST: &str = "127.0.0.1";

fn daemon_main_rs() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("main.rs")
}

#[test]
fn standalone_daemon_default_host_stays_loopback() {
    let path = daemon_main_rs();
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("读取 {} 失败: {e}", path.display()));

    let declaration = source
        .lines()
        .find(|line| line.contains("const DEFAULT_HOST"))
        .unwrap_or_else(|| {
            panic!(
                "{} 中找不到 `const DEFAULT_HOST` 声明。若该常量被移动或改名，\
                 本锁定测试必须同步更新——Q2 要求独立 daemon 的默认地址保持回环，\
                 不得改为 0.0.0.0",
                path.display()
            )
        });

    assert!(
        declaration.contains(&format!("\"{EXPECTED_DEFAULT_HOST}\"")),
        "独立 `rustcode-daemon` 的默认绑定地址必须保持 {EXPECTED_DEFAULT_HOST}（Q2 安全裁决）；\
         实际声明为：`{}`",
        declaration.trim()
    );
}
