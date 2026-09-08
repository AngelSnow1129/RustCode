# DeepSeek 的"先加载 skill"提醒（机制层）

**日期:** 2026-07-22
**状态:** 已批准，可进入实现计划
**范围:** 小 —— 只加一个 deepseek 专属的生命周期钩子，不新增子系统。

## 目标

让一个弱模型（deepseek）在任务的**开场轮**，于它开始探索代码库或提出方案
**之前**，真的先去加载匹配的过程类 skill（例如 `brainstorming`）。

## 背景 / 为什么只改 persona 不够

此前的一个修复（`08520767`）在 `coding_persona`
（`crates/rustcode-coding/src/persona.rs`）里给 deepseek 专属的
`FIRM_EXECUTION_DISCIPLINE` 段落加了一条 `SKILL/PROCESS FIRST` 要点。
真机测试（deepseek-v4-flash，11:36 构建的、包含该修复的二进制）表明它
**并没有奏效**：deepseek 依然以 "let me look at the project structure" 开场，
接着探索（List/Glob/Read）、写了一份完整的 A/B 方案分析，最后才通过
`request_user_input` 问了一个问题。它始终没有调用 `use_skill(brainstorming)`。

persona 方案失败的根源：对弱模型而言，一行**静态**系统提示词的即时性太弱 ——
它位于长提示词的靠前位置，离模型真正决定行动的时刻很远，还要与同一段落里
"FINISH THE JOB / act decisively" 这类 MANDATORY 要点竞争注意力。
最终合成的净信号读起来就是"开干"。

GLM-5.2 加载 skill 表现正常，不在本范围内 —— 它不会拿到
`FIRM_EXECUTION_DISCIPLINE`，而是遵循 `## SKILLS:` 那条较软的引导。

## 设计

新增一个**仅对 deepseek 生效的生命周期钩子**，在开场轮把一条强硬的
`<system-reminder>` 注入到请求的尾部（那里近因效应最强）—— 投递机制与
`StatusReminderHook`、`TodoHook` 已经在用的完全一致（每轮 `pre_request`
尾部注入），其效果显著强于一行静态 persona 文本。

### 组件：`SkillFirstHook`

`rustcode-coding` 里的新单元（例如 `crates/rustcode-coding/src/skill_first.rs`）。

- **状态（构造时一次性算出）：** `enabled: bool` =
  `model_needs_firm_execution(model)` 且 skill 目录非空。
  - 通过已有的 `model_needs_firm_execution` 谓词（`persona.rs`）把范围收窄到
    deepseek —— 把它暴露为 `pub(crate)`（或加一个很薄的 `pub(crate)` 包装）。
    GLM / frontier 模型永远拿不到这个钩子。
  - 未安装任何 skill（目录为空）时跳过 —— 绝不会把模型引向一个未挂载的
    `use_skill`/`brainstorming`，这与 `TodoHook` / `request_user_input` 的门控
    纪律一致。目录字符串在 `prepare()` 时就已经算好（会传给
    `SkillCatalogHook::new(skill_catalog)`），所以构造期即可做非空判断。
- **行为：** 实现 `LifecycleHooks::pre_request(&self, messages, ctx)`：
  - 若 `enabled` 为假则立即返回。
  - 只在**开场轮**触发：`ctx.turn_id == 1 && ctx.round == 1`。
    一次性；后续轮次与后续回合都不再加任何东西，因此正在进行的编码工作
    不会被每轮噪声打扰。
  - 在尾部追加一条 `<system-reminder>` 消息（照搬 `StatusReminderHook` 的
    注入方式：同样的包装辅助函数 / 消息角色）。
- **提醒文本（纯函数，可测）：**
  > 在你探索代码库、做计划或提出方案之前：先查看上方
  > "=== AVAILABLE SKILLS ===" 目录。如果本次请求命中了某个 skill 的
  > description —— 设计 / 构建 / "help me figure out / plan this" 这类请求
  > 命中的就是 `brainstorming` —— 你**必须**立刻用 `use_skill` 调用该 skill，
  > 并由它来主导：一次只问用户一个问题，不要预先把方案定死，也不要
  > 先去探索。若目录里没有任何匹配项，就按常规流程继续。

### 接线

在 `crates/rustcode-coding/src/parts.rs` 的 `prepare()` 中注册，与其他钩子
并列（放在 `TodoHook` 之后，约 539 行）。用解析后的模型（`cfg.model`）与
目录非空标志来构造。由于 TUI/CLI 与 daemon 的 `/chat`（webui）走的是
完全相同的 `CodingRuntime` → `prepare()` → `assemble()` 流水线，本钩子
能同时覆盖**两个**入口（与之前那个只改 persona 的做法不同，这里
不存在 webui 缺口）。

## 明确排除在范围之外

- **由 rustcode 自己做意图分类**（对用户消息做关键词 / 启发式匹配）。
  已否决：太脆弱（双语关键词表会腐烂；等于重新实现 skill 目录自己的
  description 匹配）。本提醒让模型自己去做匹配 —— 只是把"检查"这个动作
  强制压到决策点上。门控依据是轮次位置 + 模型，而不是猜测意图。
- **强制自动加载 skill**（在客户端展开 brainstorming 的内容并注入，
  绕过模型）。更重，且一旦误判，误触发的后果更糟。延后：只有当真机验证
  表明本提醒力度不够时才回来讨论。
- **所有模型 / GLM。** 仅 deepseek。
- **会话中途出现的设计请求**（设计类诉求出现在第 5 轮而非第 1 轮）。
  YAGNI —— 占绝对多数的情况就是开场那条消息。本轮不处理。

## 测试

- 纯提醒文本构造函数：单元测试断言它返回预期的字符串。
- 门控（构造期的 `enabled`）：deepseek + 目录非空 → 启用；
  GLM → 禁用；deepseek + 目录为空 → 禁用。
- `pre_request` 触发：在钩子启用的前提下，`turn_id==1 && round==1` 会追加
  且仅追加一条提醒消息；`turn_id==1 && round==2`、`turn_id==2`，或钩子处于
  禁用状态时，什么都不追加。（沿用 `StatusReminderHook` 既有测试的写法：
  构造一个 `TurnCtx`，再对消息尾部做断言。）
- 运行已有的 `rustcode-coding` 测试 —— 除新增这个钩子外，`coding_persona`
  与 `prepare` 的调用点签名都没有变化。

## 诚实的局限

该机制保证的是：在开场轮以很强的近因性把"先加载 skill"的指令摆到 deepseek 面前。
它**不**保证 deepseek 加载之后就会遵守 skill 的"一次一问 / 不要预先定方案"
纪律 —— 那部分仍取决于模型行为。若真机测试显示 deepseek 加载了 skill 但仍
违反其访谈纪律，那是另一个问题（skill 内部纪律），需另行处理。
**本次**改动的成功标准更窄：deepseek 在开场轮调用 `use_skill(brainstorming)`，
而不是一头扎进代码探索。
