# `request_user_input` 界面：可选的自定义答案行 + Submit 行间距

**日期:** 2026-07-22
**状态:** 已批准，可进入实现计划
**范围:** 中 —— 对 `request_user_input` 面板的两处相关打磨（第 1 项涉及工具 + TUI + webui + daemon；第 2 项只涉及 TUI 渲染）。不动 kernel。

## 目标

对结构化问题面板的两处打磨：

1. **可选的自定义答案行。** 目前"输入自己的答案…"（Other 自由文本）这一行是
   **无条件**追加到每一个 single/multiple 问题上的。改为通过 `custom` 标志
   按题控制（默认 `true`，即当前行为）。当模型给出的 options 已经穷尽时，
   它可以设 `custom: false`，自由文本行随即消失。同时引导模型**不要**自己
   再塞一个"其他 / Other / catch-all"选项（目前这会与自动追加的那行重复）。
2. **Submit 行的间距。** 在 multiple 模式下，`✔ 提交` 这一行目前紧贴着
   最后一个选项，视觉上很挤。在其上方加一个空白间隔行，
   让 Submit 读起来与选项区分开。

## 背景 / 参考

- 现状：`UserInputPanel` 在 `options.len()` 这个下标上总有一个 Other 行
  （`crates/rustcode-tuix/src/state.rs`），而 `build_user_input_rows`
  （`crates/rustcode-tuix/src/render/retained.rs`）也总会把它渲染出来；
  工具 schema 里没有任何办法把它关掉。
- opencode 的 `question` 工具（已由本地源码核实）有一个按题生效的
  `custom: Boolean`（默认 true）—— 语义是"允许输入自定义答案" —— 它的工具
  description 里写着：*"当 `custom` 处于启用状态（默认）时，会自动追加一个
  'Type your own answer' 选项；不要再自行加入 'Other' 或兜底选项。"*
  本设计即照此对齐。

## 设计

### 第 1 项 —— `custom` 标志

**工具（`crates/rustcode-capabilities/src/tools/request_user_input.rs`）：**
- 给 `UserInputRequest` 加上 `pub custom: bool`，并用 `#[serde(default = "…true")]`，
  这样缺省的 `custom` 会被反序列化为 `true`（向后兼容：既有调用方与
  当前 UI 行为都不变）。
- 把 `custom` 加进按题的 JSON schema（`"type": "boolean"`），以及 `questions[]`
  的条目 schema。更新工具的 `description`：除非你设置了 `custom: false`，
  否则会自动追加一个自由文本的 "type your own answer" 行；当你的 `options`
  已经穷尽时请设 `custom: false`；**不要**自己再加一个 "Other" / 兜底选项。
- `custom` 会随载荷一起传递，扁平的单问题与批量 `questions[]` 数组的每一项
  都带上它（它是 `UserInputRequest` 的一个字段，本来就参与序列化）。

**TUI 状态（`crates/rustcode-tuix/src/state.rs`）：**
- `UserInputPanel` 增加一个 `custom: bool`（来自请求）。当 `custom == false` 时，
  Other 行不存在：
  - `other_index` / `last_row` / 光标范围 / `checked` 向量长度 /
    `build_response`（不再有自定义文本分支）/ `is_other_row` 都要考虑到它的缺失。
    当 `custom == true` 时，上述每一项都与当前行为逐字节一致。
- `UserInputBatch` 里的每题面板各自继承该题自己的 `custom`
  （它们都是由 `UserInputRequest` 经 `UserInputPanel::new` 构造出来的）。

**TUI 渲染（`crates/rustcode-tuix/src/render/retained.rs`）：**
- `build_user_input_rows` 只在 `custom == true` 时渲染 Other 行；
  `user_input_panel_row_count` 在 `custom == false` 时减掉 Other 行所占的行
  （以及它在 multiple 模式下占用的复选框槽位）。视图（`UserInputPanelView`）
  需要携带 `custom`。

**webui（涉及 `webui/src/components/UserInputCard.tsx`、`webui/src/api.ts`）：**
- `UserInputQuestion` / `UserInputRequestEvent` 增加 `custom?: boolean`
  （缺省 ⇒ 视为 `true`）。那个 "Other" 单选框 / 复选框加自由文本输入框，
  只在 `custom !== false` 时渲染。

**daemon（涉及 `crates/rustcode-daemon/src/live_api.rs`）：**
- 在单问题的 `user_input_request` 事件上转发 `custom`
  （批量路径已经通过 `questions` 数组带上它了）。

### 第 2 项 —— Submit 行的间距（仅 multiple 模式）

**仅涉及 TUI 渲染（`crates/rustcode-tuix/src/render/retained.rs`）：**
- 在 `build_user_input_rows` 里，对 multiple 模式在 Submit 行之前压入一个
  空白间隔行；并把 multiple 模式下的 `user_input_panel_row_count` 加 1，
  使行数不变式（`row_count == build_user_input_rows(..).len()`）继续成立。
- single 模式没有 Submit 行 → 不受影响。

## 范围之外（延后）

- 修改 multiple 模式的选择语义。
- 强制模型真的省掉它自己的 "Other" 选项 —— 重复只是观感问题，不致命；
  靠工具 description 里的引导来缓解。
- 给 `text` 模式也加 `custom` 控制（text 模式没有 options / Other 行，不适用）。

## 测试

- **工具：** 缺省时 `custom` 默认为 `true`；显式给出时能解析为 `false`；
  schema 中包含 `custom`。既有的解析 / 格式化测试不变。
- **TUI 状态：** `custom == false` 的 single 面板没有 Other 行（光标范围
  止于最后一个具体选项；`build_response` 没有自定义文本分支）；
  `custom == false` 的 multiple 面板，其 Submit 下标下移一位；
  `custom == true` 的面板保持原样。
- **TUI 渲染：** `custom == false` 时去掉 Other 行，且行数与
  `build_user_input_rows(..).len()` 相符；multiple 模式在 Submit 上方
  恰好多出一个空行，且行数相符；single/multiple 且 custom == true 的面板，
  除 multiple 模式新增的 +1 空行外，行数保持当前值。
- **webui：** `custom === false` 时隐藏 Other 行，否则显示。
- 运行已有的 `request_user_input` + tuix 面板测试 —— 单问题默认路径
  （`custom` 缺省 ⇒ true）除 multiple 模式新增的 Submit 空行外必须保持不变。

## 风险 / 备注

- **行数不变式**（`user_input_panel_row_count == build_user_input_rows.len()`）
  是主要隐患 —— `custom == false` 这条路径与 multiple 模式新增的空行，
  都必须同时反映到**两个**函数里。由既有的不变式测试加上新增用例来守住。
- **既有测试**会断言 multiple 模式的具体行数；新增的 +1 空行会改变这些
  期望值 —— 应作为本次改动的一部分同步更新。
- **工作区提示：** `crates/rustcode-tuix/src/state.rs` 目前带有与本工作无关的
  未提交改动。实现时只应暂存本次改动相关的 hunk（`git add -p`），
  绝不把那些无关的 WIP 一起提交。
