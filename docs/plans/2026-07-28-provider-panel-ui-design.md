# /provider 面板重设计

**状态：** 已批准
**日期：** 2026-07-28
**范围：** 用全屏面板弹窗替换回滚区问答式的 `/provider` 向导，形制参照
`/plugin`（`PluginManager`），落在 provider 账号 / model
profiles 分支上。

## 1. 问题

当前的 `/provider` 以回滚区问答方式运行：每一步向下推一行提示，
答案在底部输入框中键入。用这种方式管理账号 + 每账号下的多个
模型很笨拙 —— 没有常驻列表，而且新增 api key / 模型是一条
线性提示链，无法就地复查或更正。

## 2. 目标

- 常驻的全屏面板弹窗（隐藏输入框，类似 `PluginManager`），让 provider
  配置可见且可导航。
- 视图用标签页（`账号` / `模型`），动作用按键绑定（与 `/plugin` 一致）。
- 面板内表单字段用于新增/编辑（api key、model、window）—— 在面板内捕获，
  而非主输入框。
- 以账号为中心：一个账号（连接 + 凭据）暴露多个模型 profile；旧的
  `[providers.*]` 仍然出现（带角标）并继续可用。
- 复用已建成的解析/持久化能力（`build_preset_entry`、
  `commit_preset_account`、感知 `resolve_model` 的 `set_default_provider_and_reload`、
  `logical_accounts`/`logical_models`）。

## 3. 非目标

- 模型发现 / 推荐（延后，父计划的 Task 9）。
- OAuth providers（GitHub Copilot 延后）。
- 重写旧版编辑语义 —— 旧条目继续保持就地编辑。

## 4. 模块

新增 `crates/rustcode-tuix/src/modals/provider_panel.rs` —— 一个 `ProviderPanel`
弹窗，形制参照 `PluginManager`。通过 `MenuKind::Plugin` 风格的常驻
页脚渲染，从而隐藏主输入框。一旦 `/provider` 指向该面板，旧的 `ProviderWizard`
即告退役；它的纯函数辅助（`build_preset_entry`、
`unique_account_id`、`commit_preset_account`、`DraftProvider::into_config`/
`apply_onto`）迁移到面板，或与面板共用。

### 状态

```
enum Tab { Accounts, Models }
enum Mode { List, Form(FormState), DeleteConfirm { target } }
struct ProviderPanel {
    tab: Tab,
    selected: usize,        // row in the current list
    mode: Mode,
    // form field state (focused field + text buffers + cursor) live in FormState
    close_requested: bool,
}
```

文本输入被捕获到获得焦点的 `FormState` 字段（照搬
`PluginManager.url_input` / `url_cursor` 的做法）。

## 5. 布局与按键

```
┌─ Provider 管理 ────────────────────────┐
【 账号 】 模型                    Tab/←→ 切换
─────────────────────────────────────────
 (current tab body)
─────────────────────────────────────────
 <context-specific key legend>          Esc
```

- `Tab` / `←` `→` 切换标签页。`↑` `↓` 移动选中项。
- **账号 tab：** `a` 新增 · `e` 编辑 · `d` 删除 · `↵` 展开（显示该账号的
  模型） · `Esc` 关闭。空态提示：`按 a 添加第一个 provider`。
- **模型 tab：** `a` 新增 · `e` 编辑 · `d` 删除 · `↵` 设为默认并切换本
  会话 · `Esc` 关闭。行按账号分组；当前生效的默认值标记为
  `● [默认]`。
- 在 Form 中：`Tab` 下一字段 · `↵` 保存 · `Esc` 返回列表。
- 在 DeleteConfirm 中：`y` / `n`。

## 6. 账号标签页

列出统一的账号目录（`logical_accounts()`）：新 schema 的账号
加上旧 provider（带 `[旧]` 角标）。每行显示账号 id、其
厂商/preset 与模型数量；持有当前生效 `default_model` 的那个账号会被标记。

- `a` 新增 → 进入新增 **Form**（§8）。
- `e` 编辑 → 预填的 Form。新 schema 账号 → 编辑账号字段
  （display_name、api_key、base_url、skip_tls）。旧 provider → 沿用
  现有字段集就地编辑（复用 `apply_onto`）；api key 留空则保留
  当前的密钥。
- `d` 删除 → DeleteConfirm；删除账号时说明随之删除哪些模型 profile。
  删除旧 provider 则移除 `[providers.*]` 条目。
- `↵` 展开 → 切到按该账号过滤的模型标签页（或内联子列表），
  让它的模型可见。

## 7. 模型标签页

列出全部模型 profile（`logical_models()`），按账号分组，排序
与 `/model` 一致。每行：`<account> · <model>`（+ `[默认]`）。

- `a` 新增模型 → 一个更短的 Form：选账号（在已有账号间循环） →
  模型名 → context_window（可选） → 设为默认`[ ]`。
- `e` 编辑模型 → 限额表单（model name、context_window、max_tokens、
  capable_model，thinking/reasoning 放在高级项下）。
- `d` 删除模型 → DeleteConfirm。
- `↵` 设为默认 → `set_default_provider_and_reload(selection_id)`（感知
  resolve；持久化 `default_model`、重载、切换会话）。

## 8. 新增表单（账号标签页）

面板内的一张表单，用于创建一个账号 + 它的首个模型，并（默认）
把它设为当前生效的默认值：

```
【添加账号】
厂商:   ‹ DeepSeek ›        (←→ 切预设 / 15 个)
api_key: sk-█________________  (留空则用 $DEEPSEEK_API_KEY)
模型:   ________________
窗口:   131072 (默认)
设为默认: [✓]
 Tab 下一项   ↵ 保存   Esc 取消
```

- `厂商` 是在 `provider_preset::PRESETS` 上用 `←`/`→` 循环的选择器。对于
  带内建端点的 preset，URL 隐藏；自定义兼容 preset
  （`*-compatible`）会显出必填的 `base_url` 字段。
- 保存时：通过 `build_preset_entry` 构建账号 + 模型；插入；若
  `设为默认` 已勾选，则把 `default_model` 设为新模型 id；`save_and_reload`。
  账号 id 的唯一性由 `unique_account_id` 保证。
- 无密钥的 preset（Ollama）隐藏 api_key 字段。

## 9. 持久化、复用与失败处理

- 所有变更都经由 `ConfigStore`（`save_and_reload`） —— 与当前向导所用的
  同一条 CAS 安全路径。重载失败时保留先前的运行时。
- 密钥：api-key 字段以掩码渲染；编辑留空则保留现有密钥。
- 复用 `build_preset_entry`、`unique_account_id`、`commit_preset_account`、
  `logical_accounts`/`logical_models`，以及感知 resolve 的
  `set_default_provider_and_reload`。

## 10. 落地（任务）

1. 面板骨架：弹窗结构体、标签页头部 + 列表渲染、`MenuKind::Plugin`
   输入框隐藏、按键路由、`/provider` → `ProviderPanel`。旧向导在对齐之前
   保留在幕后。
2. 账号标签页列表（账号 + 旧版角标、模型数量、默认标记） +
   空态。
3. 新增表单（preset 循环器、字段、经 `build_preset_entry` +
   `set-default` 保存），自定义端点 base_url 字段。
4. 账号的编辑 + 删除（新 schema + 旧条目就地编辑）。
5. 模型标签页：列表 + 设为默认（`↵`） + 新增/编辑/删除模型。
6. 退役回滚区向导；保留共用的纯函数辅助。

每个任务都要编译并通过测试（`cargo test -p rustcode-tuix provider_panel`），
且各自独立提交。真机交互需要实机验证。
