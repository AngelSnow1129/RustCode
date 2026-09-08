# 多问题的 `request_user_input`（批量表单）

**日期:** 2026-07-22
**状态:** 已批准，可进入实现计划
**范围:** 中 —— 工具 schema + TUI 批量界面 + webui 顺序兜底 + daemon 端点。不动 kernel。

## 目标

让一次 `request_user_input` 工具调用能提出最多 4 个问题，并在一次交互中
完成作答：在 TUI 里用户用 Tab 在问题之间切换、逐题作答、然后一起提交；
在 webui 里问题以一次一张卡片的方式逐步推进，最后作为一个批次提交。
这是 [头脑风暴 → request_user_input 的 persona 桥接](2026-07-22-brainstorming-request-user-input-design.md)
在结构上的后续动作，终于给"多问题访谈"配上了像样的一次性交互表单。

头脑风暴期间记录下来的约束：
- **范围 B**：TUI 拿到完整的多问题 Tab 界面；webui *降级*为一次一题地
  逐步推进（复用当前的单问题卡片），最后提交一次批量响应。本轮不做
  webui 的并排 / 表单式界面。
- **允许部分提交（B）**：用户可以只回答了部分问题就提交；未作答的问题
  以 `declined` 返回。`Esc` 则拒绝整批。（与当前单问题场景"Esc 即拒绝"
  的哲学一致。）
- **向后兼容**：扩展已有的 `request_user_input` 工具，不新增工具。

## 非目标（延后）

- webui 里并排的多问题**表单**（本轮只做顺序步进器）。
- 强推模型去"批量提问"。工具 description 里会提到这个能力
  （"you may ask up to 4 related questions at once"），但批不批量由模型自己决定。
  不会去硬拧头脑风暴"一次一问"的默认行为。
- 单次调用超过 4 个问题（会被截断到 4 个）。

## 架构

kernel 的请求 / 响应接缝在两个方向上传递的都是不透明的 `serde_json::Value`
（`AgentEvent::Request { payload }` / `AgentCommand::Respond { value }`），
因此**不需要改动 kernel**。批量形态完全存在于工具载荷与 driver 响应之中。

数据流：
```
tool sends {questions:[...]}  →  driver (TUI/webui) collects answers
                              →  driver responds {responses:[...]}
                              →  tool formats per-question result for the model
```

### 单元 1 —— 工具层（`crates/rustcode-capabilities/src/tools/request_user_input.rs`）

- **Schema：** 新增一个可选的顶层 `questions` 数组。每一项沿用当前的
  `{ header, question, mode, options }` 形态。当 `questions` 存在且非空 → 走批量；
  否则沿用当前的顶层单问题形态（legacy）。
- **解析：** 把两种形态都归一化成内部的 `Vec<UserInputRequest>`
  （长度 `1..=4`；若模型发来更多，则截断到前 4 个）。
- **发给 driver 的载荷：** 批量 → `{ "questions": [ <UserInputRequest>, ... ] }`；
  单题（经由 legacy 形态传来的 1 个问题）→ 保持当前的扁平
  `{ header, question, mode, options }`，这样既有 driver 的解析逻辑不受影响。
  （由 legacy 形态构造出来的、恰好只有 1 题的批次，在线路上仍保持 legacy。）
- **来自 driver 的响应：** 批量 → `{ "responses": [ <UserInputResponse>, ... ] }`
  （每题一个，按序排列）；单题 → 当前的扁平 `UserInputResponse`。
- **`format_result`（批量）：** 每题一行，以该题的 `header` 为键，例如
  `Q1 (Approvals): User selected "approval request"` / `Q2 (Shape): User answered "…"`
  / `Q3 (Triggers): No answer (declined)`。单题的格式化方式不变。
- **空值 / 拒绝：** 空响应（driver 崩溃 / 自动跳过）或整批全部 declined，
  都会退化为既有的"没有答案，按你自己的最佳判断继续"引导。

### 单元 2 —— TUI 批量状态与导航（`crates/rustcode-tuix/src/state.rs`）

- 引入 `UserInputBatch { request_id: u64, questions: Vec<UserInputPanel>, current: usize }`。
  `UserInputPanel`（当前的单问题状态 —— cursor、checked、text、custom_text、
  Other 行）被**逐字复用为每题的状态**；它的 `request_id` 字段上移到批次层
  （每个问题的面板不再各自持有该字段）。
- `current` 的取值范围是 `0..=questions.len()`：`0..questions.len()` 是各题面板，
  而 `questions.len()` 是**Submit 停靠站**。
- 导航方法：`next_question()` / `prev_question()` 会环绕经过 Submit 停靠站
  （Tab / Shift+Tab）；当前问题自身的 `move_up/down`、`toggle`、
  `push/pop_custom` 则作用于 `questions[current]`。
- `build_batch_response() -> Vec<UserInputResponse>`：通过既有的单题
  `build_response()` 映射每个面板；用户从未碰过的问题产出 `declined`
  响应（即部分提交语义）。
- **N==1 时精确退化为今天的行为**：没有 Submit 停靠站、没有导航条；`Enter`
  立即提交（与当前单问题路径一致）。单问题的渲染 / 事件处理保持像素级与
  按键级一致；只有 N>1 才会多出导航条 + Tab + Submit 停靠站。

### 单元 3 —— TUI 渲染（`crates/rustcode-tuix/src/render/`）

- 一个 `UserInputBatchView`（或扩展既有视图）：当 N>1 时，先渲染顶部导航行
  `Question {current+1}/{N}`，带每题的状态字形（已答 `✓` / 未动 `○`，
  经既有的字形兜底逻辑做 ASCII 降级），然后用**既有的 `build_user_input_rows`
  单题逻辑**渲染当前题目的各行，再在 `current == questions.len()` 时渲染
  Submit 行，最后渲染一条提到 Tab 的提示。
- N==1 时输出与今天逐字节一致（没有导航条，也没有批量外框）。

### 单元 4 —— TUI 事件处理（`crates/rustcode-tuix/src/event_loop/mod.rs`）

- 批量场景下的 `handle_user_input_key`：`Tab`/`Shift+Tab` → `next_question`/`prev_question`；
  `↑↓`、`Space`、数字键、字符键、`Backspace` 作用于 `questions[current]`，行为与今天
  完全一致；在 single 模式的问题上按 `Enter` 会选中并前进到下一题（若已是最后一题，
  则前进到 Submit 停靠站）；在 Submit 停靠站按 `Enter` 会构造
  `build_batch_response()` 并投递；`Esc` 拒绝整批（所有问题 → declined）。
  N==1 时保持今天"Enter 立即提交"的行为。
- 请求解析：当载荷带有非空的 `questions` 数组时，构造 `UserInputBatch`；
  否则构造今天的单个 `UserInputPanel`。
- `deliver_user_input` 新增一条批量路径，响应 `{ "responses": [...] }`。

### 单元 5 —— webui 顺序兜底（`webui/src/components/UserInputCard.tsx`、`webui/src/api.ts`）

- 当 `user_input_request` SSE 事件携带 `questions` 时，卡片会**一次一题地
  复用今天的单问题渲染**逐步推进：回答 Q1 → 本地暂存 → 展示 Q2 → …… →
  到最后一题时，把累积的整批一次性 POST 为
  `{ request_id, responses: [ {declined,selected,text}, ... ] }`。
- 单问题（legacy 载荷）仍原样 POST 当前的扁平 body。
- kernel 对每个 `request_id` 只等**一次**响应，所以 webui 必须在本地累积，
  并且最终只发一次 POST。

### 单元 6 —— daemon 端点（`crates/rustcode-daemon/src/live_api.rs`）

- `UserInputAnswerReq` 新增一个可选的 `responses: Vec<UserInputResponse-shape>`。
  `live_user_input`：当 `responses` 存在时，响应 `{ "responses": [...] }`；
  否则沿用当前的扁平 `{ declined, selected, text }`。
- `AgentEvent::Request` → `LiveWireEvent::UserInputRequest` 的投影也要在
  `questions` 数组存在时把它转发出去（这样 webui 才能收到）。

## 测试

- **工具：** 两种形态都能解析；超过 4 个截断到 4；`format_result` 的批量输出
  （以 header 为键的每题一行，含一道被 declined 的题目）；批量响应的
  反序列化；单问题路径不变。
- **TUI 状态：** `UserInputBatch` 的导航（Tab 环绕经过 Submit 停靠站、
  prev/next 的边界）、`build_batch_response`（未碰过的问题 → declined；
  已答 / 未答混合的情况）、N==1 的退化与单问题路径一致。
- **TUI 事件：** Tab/Shift+Tab 在问题之间移动；Enter 先前进，到 Submit 停靠站时
  提交；Esc 拒绝全部；数字键 / 空格 / 字符键都路由到当前问题。
- **webui：** 步进器累积每题答案，并在最后一题触发且仅触发一次批量 POST；
  单问题路径仍 POST 扁平 body。
- 运行已有的 `request_user_input` / tuix 面板测试 —— 单问题路径不得回归。

## 风险 / 备注

- **单问题回归风险**是主要隐患（现有面板被广泛使用）。缓解手段：N==1 复用
  同一套渲染 / 处理代码，并由专门的测试断言其输出与按键逐字节一致；
  批量外框只在 N>1 时出现。
- **线路兼容性**：批量新增了一种载荷 / 响应形态；不认识 `questions` 的 driver
  会解析失败 —— 这可以接受，因为两个 driver（TUI + webui）在同一次改动里
  都会被更新，且单问题调用仍保留 legacy 的单题形态。
- 模型究竟会不会真的把问题批量化，不在本范围内（那是模型的选择）；
  本次改动只提供这个能力，外加在工具 description 里提一句。
