# 把头脑风暴阶段的问题接入 `request_user_input`（persona 桥接）

**日期:** 2026-07-22
**状态:** 已批准，可进入实现计划
**范围:** 小 —— 一处 persona 措辞增量，不涉及结构性代码。

## 目标

让头脑风暴 / 设计细化流程中提出的问题，能够通过已有的 `request_user_input`
工具直接在 UI（TUI 面板 / webui 弹窗）里作答，而不是被写成一段散文，
逼用户在输入框里敲自由格式的回复。

头脑风暴过程中记录下来的用户约束：

- **保持简单，只做头脑风暴。** 不是"把每一个澄清问题都接到 UI 上" ——
  现阶段只处理头脑风暴这一种场景。
- **机制上采用 persona 引导**（不去改外部 superpowers skill 文件，
  也不新增 skill 检测 / 注入钩子）。

## 现状（这些都已经存在 —— 这里不需要新建任何东西）

端到端的整套机制已经就位并接好线了：

- **工具：** `request_user_input` —— `crates/rustcode-capabilities/src/tools/request_user_input.rs`。
  模式有 `single` / `multiple` / `text`；字段有 `header`、`question`、`mode`、`options`。
- **界面：** TUI 底部面板（`rustcode-tuix` 的 `UserInputPanel`、`render/retained.rs`）
  与 webui 弹窗（`webui/src/components/UserInputCard.tsx`）。两者都已经能渲染
  请求并把答案回传。
- **往返链路：** kernel 的 `AgentEvent::Request` → driver → `AgentCommand::Respond`
  （`crates/rustcode-kernel/src/request.rs`）。拒绝 / 超时会退化成一个
  非错误的 "no answer" 结果。
- **环境变量开关：** `RUSTCODE_REQUEST_USER_INPUT`（默认开启）。辅助函数
  `request_user_input_enabled_from_env` 位于 `crates/rustcode-config/src/config/mod.rs:592`；
  在 `crates/rustcode-capabilities/src/tools/mod.rs:191-214` 还有一份刻意的重复实现。
- **Persona 接线：** `coding_persona(model, todo_enabled, request_user_input_enabled)`
  —— 位于 `crates/rustcode-coding/src/persona.rs:78`。3 个调用点都已经传入
  `request_user_input_switch_enabled()`：
  - `crates/rustcode-coding/src/assemble.rs:71`
  - `crates/rustcode-coding/src/parts.rs:821`
  - `crates/rustcode-coding/src/parts.rs:994`
- **已有的 persona 段落：**
  - `## SKILLS:`（`SKILLS_USAGE`，persona.rs:288）—— 让模型**先**加载匹配的
    skill（例如 brainstorming），并"由它来主导提问"。
  - `## ASKING THE USER:`（`REQUEST_USER_INPUT_USAGE`，persona.rs:303）—— 受
    `request_user_input_enabled` 控制；引导模型把工具用在"确实该由
    用户自己拿主意"的决策上。

## 真正的缺口

在头脑风暴过程中，模型会读到两段互不相通的内容：

- `## SKILLS:` —— 由 brainstorming 来主导提问。
- `## ASKING THE USER:` —— `request_user_input` 被视为一道**稀缺闸门**：
  "确实该由用户自己拿主意"、"只问你确实无法自行决断、查证或核实的东西"、
  "凡是快速查一下就能回答的，绝不拿来问"。

而头脑风暴的职责恰恰与"稀缺"相反：它**刻意**要问很多探索性的澄清问题
（目的、约束、方案偏好），"一次问一个，优先用选择题"。模型并没有把这些问题
与 `request_user_input` 联系起来 —— "少问"的框架读起来反而像是一个
*不要*去用它的理由 —— 于是它就把问题写成了散文。这两段之间需要一座桥。

## 改动

只改 `REQUEST_USER_INPUT_USAGE` 里的一处措辞（`crates/rustcode-coding/src/persona.rs:303`）：
追加一句桥接条款，要旨如下：

> 当某个 skill（例如 brainstorming）正在主导一轮用于细化设计的澄清 / 访谈式
> 问答时，也要通过本工具把它**自己**的问题抛出来 —— 选择题用带上具体
> `options` 的 `single`/`multiple`，开放题用 `text` —— 这样用户是在 UI 里作答，
> 而不是去读一段散文式提问。上面那条"少问 / 只问你自己无法决定的事"的引导
> 约束的是你自己的临时提问；它**不**限制一个 skill 的结构化访谈。

属性：

- 位于 `REQUEST_USER_INPUT_USAGE` **内部**，因此自动受已有的
  `request_user_input_enabled` 开关管辖 —— 工具被禁用时该条款随之消失
  （绝不会把模型引向一个未挂载的工具；与 TodoHook 的门控原则一致）。
- 不新增参数、不新增调用点、不改外部 superpowers skill。
- 对所有模型生效（保持简单），挂在已有的条件段落上。

可选：可以在 `SKILLS_USAGE`（persona.rs:288）里加一句指针，让两段互相
交叉引用。这属于锦上添花，不是必需项；计划中应把它当作可选且低风险的改动。

## 明确排除在范围之外（延后）

- **webui 的 `/chat` 路径。** daemon 的 `/chat` 流式端点通过
  `build_api_system_prompt`（`crates/rustcode-daemon/src/lib.rs:3378`）构造系统提示词，
  它用的是 `rustcode_config::config::prompt_sections` / `UNIFIED_PROMPT`，**不会**
  调用 `coding_persona`。因此本引导覆盖不到在 webui 里进行的头脑风暴。
  与之前已延后的 todo-nudge daemon 缺口属于同一性质。记录在案即可，
  本轮不修。
- 修改本地 / vendored 的 superpowers `brainstorming` skill markdown。
- 把非头脑风暴场景下的临时澄清问题也接到 UI 上（"所有澄清问题都走 UI"
  这个更宽的范围已被考虑过并否决，最终选择了"保持简单，只做头脑风暴"）。

## 测试

- Persona 单元测试：断言当 `request_user_input_enabled == true` 时，桥接条款
  出现在 `coding_persona(...)` 的输出里；为 `false` 时不出现。
- 运行已有的 `rustcode-coding` persona 测试（`persona` / `parts`）—— 签名与
  调用点都没有变化，所以它们应当继续保持通过。
- 真正的验证只能靠手工（启动一次 TUI 头脑风暴会话，确认选择题会弹出
  请求面板）。按项目惯例，本改动以"未真机"状态交付 —— 由用户在真实终端上验证。

## 风险 / 备注

- 纯提示词改动：风险是行为性的（模型可能过度或过少地触发该工具），
  而非结构性的。缓解手段是把条款的适用范围限定为"某个 skill 正在主导这轮问答"，
  而不是去放松那条通用的稀缺规则。
- 若新增条款让模型过于急切地为它*自己*的临时问题调用 `request_user_input`，
  就把"不限制 skill 的结构化访谈 / 不放松你自己的临时提问"这句措辞收紧。
