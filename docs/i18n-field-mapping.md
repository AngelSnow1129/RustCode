# RustCode i18n 统一字段映射规范

> 本文档是 RustCode 全端 i18n 的**唯一对照源**。所有 UI 可见字符串必须通过
> 统一字段 key 查找翻译,禁止硬编码。各端(Rust / WebUI / VS Code / JetBrains)
> 的翻译表必须与本对照表保持键一致。

## 设计原则

1. **统一字段 key** — 同一概念在所有端使用同一个 key,跨端可追溯。
2. **穷尽 match(Rust)** — `Msg` 枚举 + 穷尽 `match` 保证编译期键完整性。
3. **fallback 链** — 缺失翻译时:用户语言 → 产品默认(zh_CN) → key 本身(最后手段)。
4. **占位符统一** — `{name}` 花括号占位符,各端用同一插值逻辑。
5. **术语保留** — 产品名/技术标识符保持英文,中英混排加半角空格。

## 端映射

| 端 | key 来源 | 翻译表 | fallback |
|----|---------|--------|----------|
| Rust (TUI/CLI/daemon) | `Msg` 枚举变体 | `en.rs` / `zh_cn.rs` | 编译期穷尽(无 fallback 需要) |
| WebUI (Preact) | `MsgKey` 字符串字面量 | `i18n.ts` zh/en 对象 | `table[key] ?? messages.zh[key] ?? key` |
| VS Code 扩展 | `package.nls.json` key | `package.nls.json` / `package.nls.zh-cn.json` | VS Code NLS 自动 fallback 到 `package.nls.json` |
| JetBrains 扩展 | `.properties` key | `RustCodeBundle.properties` / `RustCodeBundle_zh.properties` | IntelliJ ResourceBundle 自动 fallback |

## key 命名规范

### Rust 端(`Msg` 枚举)

```rust
pub enum Msg<'a> {
    // 模块前缀 + PascalCase
    WelcomeBannerLine1,           // WelcomeWizard 模块
    CpSetupHeader,               // CodingPlan 模块 (Cp 前缀)
    ChatAuthExpired,             // Chat 模块

    // 带参数的变体用 snake_case 字段
    CpLoggedIn { who: &'a str, username: &'a str, email: &'a str },
}
```

### WebUI 端(`MsgKey` 字符串)

```typescript
// 模块前缀 + camelCase,用点号分隔
'header.menu': '菜单',
'header.sessionList': '会话列表',
'sidebar.newChat': '新建对话',
'mcp.blockedUntrusted': '已拦截 {n} 个来自不受信任项目的 MCP server。',
```

### VS Code 扩展(NLS key)

```json
// 模块前缀 + camelCase,用点号分隔
"rustcode.displayName": "RustCode for VS Code",
"rustcode.commands.openSidebar.title": "RustCode: Open in Side Bar",
```

### JetBrains 扩展(.properties key)

```properties
# 模块前缀 + camelCase,用点号分隔
gear.connectStart=[*] Connect / Start
gear.provider=Provider
```

## 占位符规范

- 统一用 `{name}` 花括号(非 `${name}` / `%s` / `{0}`)。
- Rust 端:`format!()` 或 `Cow::Owned`。
- WebUI 端:`s.split('{name}').join(value)`。
- 占位符名用 snake_case(Rust)/ camelCase(WebUI),与所在端字段风格一致。

## fallback 链(优先级高 → 低)

1. 用户显式选择的语言(`Config.language` / localStorage `rustcode.lang` / VS Code locale)。
2. 环境检测(`LC_ALL` → `LC_MESSAGES` → `LANG`;`C`/`POSIX` = 无偏好 → 产品默认 zh_CN)。
3. **产品默认:zh_CN**(简体中文)。
4. 翻译表缺失键时:Rust 编译期报错;WebUI fallback 到 zh 再到 key 本身。

## 术语保留表(不翻译)

| 类别 | 术语 | 说明 |
|------|------|------|
| 产品名 | RustCode, Claude, Anthropic, Codex | 品牌名保持原文 |
| 模块名 | Provider, CodingPlan, Skill, MCP | 功能专有名词 |
| 技术标识 | API key, Base URL, token, model | 技术术语 |
| 命令 | `--provider`, `--model`, `/login` | CLI 参数/斜杠命令 |

**规则:** 中英混排时英文术语前后加半角空格。

## 一致性验证

### Rust 端(编译期保证)

```bash
# en.rs 和 zh_cn.rs 的 Msg:: 分支数必须相等,且等于 Msg 枚举变体数
grep -c "Msg::" crates/rustcode-config/src/i18n/en.rs
grep -c "Msg::" crates/rustcode-config/src/i18n/zh_cn.rs
# 编译时穷尽 match 保证无遗漏
cargo test -p rustcode-config --lib
```

### WebUI 端(运行时 fallback + 测试)

```bash
# zh 和 en 对象的 key 集合必须相等
node -e "const i = require('./webui/src/i18n.ts'); ..."
# i18n-regression.test.ts 验证 fallback 链
```

### VS Code 扩展

```bash
# package.nls.json 和 package.nls.zh-cn.json 的 key 集合必须相等
python3 -c "import json; ..."
```

### JetBrains 扩展

```bash
# RustCodeBundle.properties 和 RustCodeBundle_zh.properties 的 key 集合必须相等
```

## 当前基线(2026-09-01)

| 端 | zh keys | en keys | 对齐 |
|----|---------|---------|------|
| Rust (Msg 枚举) | 781 分支 | 781 分支 | PASS(编译期穷尽) |
| WebUI (i18n.ts) | 381 | 381 | PASS |
| VS Code (NLS) | 26 | 26 | PASS |
| JetBrains (Bundle) | 15 | 15 | PASS |

## 新增翻译流程

1. **Rust 端**:在 `messages.rs` 的 `Msg` 枚举加变体 → 在 `en.rs` 和 `zh_cn.rs`
   各加一个 `match` 分支(编译器会强制) → 运行 `cargo test -p rustcode-config`。
2. **WebUI 端**:在 `i18n.ts` 的 `zh` 和 `en` 对象各加一个 key → 运行
   `i18n-regression.test.ts`。
3. **VS Code 端**:在 `package.nls.json` 和 `package.nls.zh-cn.json` 各加一个 key。
4. **JetBrains 端**:在 `RustCodeBundle.properties` 和
   `RustCodeBundle_zh.properties` 各加一行。
5. **更新本文档的基线表**。
