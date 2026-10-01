# IM WebUI 增强 + 会话来源标识 -- 交接文档 (2026-10-01)

> **读者**: 接手推进的执行 agent。本文自包含:先读 §0/§1 了解现状,§2 是文件地图,§3 是验证状态,§4 是已知坑与未决项。所有 文件:行号 均已实测核对。

## 0. 基线 (必读)

- branch `dev`,起点 HEAD `c2b98aea`(提交前)。
- 本轮改动**全部未提交**,工作区**仅含本批 IM 改动**,无混杂非本轮文件(persona/team/schedule/updater 等上轮残留本次不在)。
- 提交纪律:本批作为**单组整体提交**(18 修改 + 2 新增),见 §5;不混入其它文件。
- 用户已裁决的"刻意不做"项见 AGENTS.md 注释与 §4:daemon 侧 OS 服务状态查询、IM 弹层内解绑按钮、微信/QQ 平台。

## 1. 改动摘要

### A. 会话来源标识 `SessionOrigin`(后端,加性 wire)

`SessionOrigin`(枚举 `manual`/`scheduled`/`im`,`#[serde(rename_all="lowercase")]`,`#[default]=Manual`) 既有类型,本次打通其 wire/聚合传播:

| 落点 | 改动 |
|---|---|
| `rustcode-capabilities::session::manager::CatalogEntry` | 新增 `origin` 字段;`catalog_entry()` native 聚合读 `SessionMeta.origin`,legacy-only 聚合固定读 `Manual`(旧会话无此字段) |
| `rustcode-daemon::SessionSummary` | 新增 `origin` 字段(`#[serde(default)]` 加性 wire);`catalog_entry_to_session_summary` 映射 `entry.origin` |
| `webui/src/api.ts::SessionMeta` | 前端字段 `origin?: string`(旧 daemon 不下发时按 manual) |
| `webui` 会话行 | `origin === 'im'` 渲染 `.session-origin-badge` 药丸(title 解释三平台) |

**兼容性**: 旧 webui bundle 忽略未知 JSON 键;旧目录条目(无 origin)读 `Manual`;旧 `.meta` 文件 `serde(default)` 落 `Manual`。零破坏性。

### B. WebUI IM 记录栏目增强(纯客户端,零后端改动)

1. **未读圆点** `webui/src/lib/imUnseen.ts`[新]: 比对树内最新 `updated_at` 与 localStorage 的 last-seen mark(`rustcode.im.lastSeen`),关闭弹层时写 `markImSeen()`;展开态与折叠 rail 的 IM 入口都渲染 `.im-unseen-dot`。
2. **平台状态药丸** `imPlatformStatus()`(configured+enabled→enabled / configured 但关→disabled / 未配→unconfigured) 在平台名旁显示三色药丸;顶部加 `可用渠道 n/total` 摘要行。
3. **面包屑返回键** `‹`(`.im-back`,32px 触达 + focus-visible 环,与 `.im-crumb` 共用焦点规则)。
4. **渠道弹窗凭据分组** `.im-cred-section` 虚线分组(标题写明 `$ENV` 引用契约);测试连接结果带 `[OK]/[FAIL]` 标记 + `endpoint_host` 底色行。
5. **删除渠道确认文案** 补齐影响说明(平台侧机器人仍在但消息不再接收、会话历史保留、其它渠道不受影响,对标 WorkBuddy)。

## 2. 文件地图

```text
后端 (加性字段 + 测试 struct 同步):
crates/rustcode-capabilities/src/session/manager.rs   +CatalogEntry.origin / native 读 meta.origin / legacy 读 Manual
crates/rustcode-capabilities/src/session/worklog.rs   test struct 同步
crates/rustcode-daemon/src/lib.rs                     +SessionSummary.origin wire 字段 + 聚合映射
crates/rustcode-daemon/src/legacy_convert.rs          test struct 同步
crates/rustcode-cli/src/acp/sessions.rs               test struct 同步
crates/rustcode-cli/src/main.rs                       test struct 同步  (注意实际包名 rustcode-clix)
crates/rustcode-tuix/src/session.rs                   test struct 同步

前端:
webui/src/api.ts                                     +SessionMeta.origin?
webui/src/components/Sidebar.tsx                      未读圆点/状态药丸/摘要行/返回键
webui/src/components/SettingsDialogs.tsx             凭据分组 / [OK][FAIL] / 删除确认文案
webui/src/i18n.ts                                    新增 zh/en 键 (originImTitle/back/credentialsSection/status*/summaryReady + removeConfirm 扩写)
webui/src/lib/imRecords.ts                            +imPlatformStatus
webui/src/lib/imUnseen.ts                             [新] 未读判定 + last-seen 持久化
webui/src/lib/imUnseen.test.ts                        [新] 契约测试
webui/src/lib/imChannelsStyles.test.ts                样式契约追加 (第二个测试锁新类)
webui/src/lib/imRecords.test.ts                       测试追加
webui/src/styles/app.css                              +137 行 IM 样式 (.im-unseen-dot/.im-status*/.im-back/.im-cred-section/.im-test-*)

文档:
AGENTS.md                                            记录 2026-10-01 增强说明 (§IM 记录栏目)
```

## 3. 验证状态

- `cargo check -p rustcode-capabilities -p rustcode-daemon -p rustcode-clix -p rustcode-tuix --all-targets`: **通过**(2026-10-01 实测,1m00s,0 错)。
- `cargo test -j 1 -p rustcode-capabilities -p rustcode-daemon -p rustcode-clix -p rustcode-tuix --lib`: **未能完整运行** —— `rustcode_capabilities` 在 8GB cgroup 下编译期 OOM(SIGKILL,即使 `-j 1` + `-C codegen-units=1` 仍触发),为**本环境硬限制,非代码缺陷**。编译正确性已由上条 `cargo check --all-targets` 覆盖(含 test target 类型检查);改动为加性字段 + 机械 struct 同步,零逻辑变更。建议在有 ≥16GB 内存的 CI 上跑 `cargo test -p rustcode-capabilities -p rustcode-daemon --lib` 闭环。
- `webui`: `npm test` **286/0 通过**(2026-10-01 实测);`npm run typecheck` 0 错(随 test 流程覆盖)。
- **触碰面标注**(按工作流协议 G1): 触碰 wire 协议(`SessionSummary.origin` 加性字段,向后兼容),持久化格式无新变更(`.meta` 沿用既有 `serde(default)`),无安全边界 / 运行时生命周期变更。

## 4. 已知坑 / 后续协作提示

- CLI 包名是 **`rustcode-clix`**(非 `rustcode-cli`,`-p rustcode-cli` 直接失败)。
- 全量测试必须 `cargo test -j 1`(8GB cgroup,默认并发 SIGBUS)。
- 非 React 模块间值导入**必须带 `.ts` 扩展名**;新 i18n 键 zh+en 双写,`i18n.test.ts` 的 parity 会拦单边。
- 样式契约测试范式参照 `composerStyles.test.ts`(读 TSX+CSS 做 assert.match,不需浏览器)。
- **刻意不做**(AGENTS.md 注释,翻案需用户确认): daemon 侧查 OS 服务状态(daemon 与 `rustcode im serve` 可能不同机,语义不成立)、IM 弹层内解绑按钮(记录面板 2026-09-23 裁决「只读,无增删」)、微信/QQ 平台(无一手协议文档)。
- **未决 / 待确认**: 本批改了 daemon 源码,若部署侧依赖预构建 `dist`,需 `cargo clean -p rustcode-daemon` 后重 build 才会生效;但 IM 记录面板读 live daemon API(非 bundle 内嵌),本地 dev 直接生效。

## 5. 提交计划

- **单组整体提交**(不分拆): 18 修改 + 2 新增,message 中文 Conventional Commit。
- 范围建议: 后端 origin wire 归 `feat(im)`,前端增强归 `feat(webui)` —— 若保持单 commit 则用 `feat(im): WebUI IM 栏目增强与会话来源标识`;若分两个 commit 见下:
  1. `feat(im): 会话来源标识 SessionOrigin 打通 CatalogEntry→SessionSummary wire`
  2. `feat(webui): IM 记录栏目增强(未读圆点/状态药丸/返回键/凭据分组)`
- 尾注:`Co-Authored-By: RustCode (glm-5.3-flash) <noreply@rustcode.dev>`。
- **仅 commit + push dev,不 force、不混其它文件**。
- 推送前确认 `cargo test` 受影响 crate 全绿(见 §3)。
