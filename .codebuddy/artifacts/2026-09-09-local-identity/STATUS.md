# 2026-09-09-local-identity 看板

- 当前阶段：设计（G2）
- 基线：branch=dev commit=da3ee382（codingplan 移除已收尾）

## 目标
移除 OAuth 与第三方平台凭证依赖，建立完全独立的本地身份体系，不依赖 `RUSTCODE_PLATFORM_SERVER` 或任何外部平台。

## 用户裁决（G1 定档，2026-09-09）
| 项 | 裁决 |
| scope | **A 彻底删**：`rustcode-auth` crate 整体移除 + `endpoints::platform_server()` + `RUSTCODE_PLATFORM_SERVER` 环境变量 |
| /login、/logout | **直接删除**（凭据改由 `/provider` 或手改 config.toml 配置） |
| 本地身份 | **不引入身份 ID**，`/whoami` 只显示当前 provider / model / 配置目录 / 运行时状态 |
| 存量凭证 | **全部清除**（`~/.rustcode/auth.toml` 等） |

## 门禁
| 门禁 | 状态 | 依据 |
| G1 需求 | pass | 用户四项裁决 + 编排者实测事实底稿 |
| G2 设计 | in_progress | solution-architect 已派发 |
| G3/G4 | pending | 待 G2 |
| G5/G6 | pending | 待 G4 |

## 已实测事实（删除面）
- `crates/rustcode-auth/src/` 仅 `lib.rs` + `oauth.rs`；导出 `RUSTCODE_USER_AGENT`(:26)、`write_auth_file_secure`(:28)、`oauth` 模块(:17)
- 依赖方：capabilities(optional, `provider`/`atomgit` feature)、cli、daemon、tuix
- `rustcode_auth::` 使用点：capabilities/atomgit/mod.rs:59、atomgit/push_label_mw.rs:57、
  plugin/marketplace.rs:303/304/344/358/361/767；daemon/api_auth.rs:61、commands.rs:490/610/613/649/822、
  login_state.rs:3/69、login_state_tests.rs:7-8
- `endpoints.rs:33` PLATFORM_SERVER_ENV、:161 platform_server()、:299 测试、:64 注释
- `tuix/commands.rs:35` MANAGED_ONLY_COMMANDS = ["login","logout","whoami","usage"]
