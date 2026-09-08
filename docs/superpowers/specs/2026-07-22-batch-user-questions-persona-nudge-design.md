# 引导模型把多个用户问题合并进一次 `request_user_input` 调用

**日期:** 2026-07-22
**状态:** 已批准，可进入实现计划
**范围:** 小 —— 只新增一段 persona 措辞，不引入新机制。

## 目标

让模型把若干个相关问题放进**一次** `request_user_input` 调用
（`questions[]` 数组 → 已经存在的 Tab 导航批量表单），而不是
发出 N 个互不相干的单问题调用，逼用户一次只能回答一个、
且没有回退的余地。

## 背景

多问题批量能力已经随版本发布（工具侧的 `questions[]` 数组、TUI 侧
带 Submit 停靠站的 Tab 导航表单、webui 侧的顺序步进器，最终汇总为
一次批量响应）。但在一次真实运行中，deepseek-v4-flash 发出了 **3 个彼此独立的**
`request_user_input` 调用（日志显示 "Running 3 request_user_input calls"），而不是一次
批量调用 —— 于是每一次都是一个孤立的单问题面板：回答、提交、
下一个，没有回退导航。批量 UI 从来没有被真正喂进过一批问题。

工具的 *description* 里其实已经提到了 `questions` 数组，但模型
（一个会低估工具描述权重的弱模型）直接忽略了它。

## opencode 与 codex 是怎么解决这个问题的（参考，已由本地源码核实）

两者都支持多问题表单，而且做法一致：**用一次工具调用携带全部问题**，
再叠加**提示词引导，告诉模型去用它** —— 双方都不在运行时把 N 次独立调用
合并起来。

- **opencode**（`packages/core/src/tool/question.ts`）：一个 `question` 工具，
  入参是 `questions: Array`；TUI 渲染一个可用 Tab/左右键切换的表单，并把
  所有答案一次性提交。权限是逐个授予的，不存在调用合并。
- **codex**（其模板位于 `codex-rs/.../collaboration-mode-templates/templates/default.md`）：
  一个 `request_user_input` 工具外加 MCP elicitation（一次请求、一个含多个
  property 的 JSON Schema → 一个多字段表单），另有一条明确的提示词规则：
  *"Never write a multiple choice question as a textual assistant message."*（codex 原文）

结论：业界的答案是**提示词引导**（也就是本设计），而不是运行时合并。
rustcode 的批量 UI 已经与 opencode 的 `question` 工具对齐，
唯一缺的一环，就是引导模型只发起一次调用。

## 设计

在已有的 `REQUEST_USER_INPUT_USAGE` 段落
（`## ASKING THE USER:`、位于 `crates/rustcode-coding/src/persona.rs`）里
新增一条批量规则；该段落本来就受 `request_user_input_enabled` 开关控制
（所以这段引导只会在工具真正挂载时出现）。规则的要旨如下：

> 当此时你向用户提出了**超过一个**问题时，把它们**全部**放进
> 同一次 `request_user_input` 调用的 `questions` 数组 —— 不要在同一轮里
> 发出多次 `request_user_input` 调用，也绝不要把选择题写成散文。
> 用户随后会在同一个表单里一次性回答它们。

属性：
- 位于那个已受开关控制的段落内部 → 工具被禁用时它会随之消失
  （`RUSTCODE_REQUEST_USER_INPUT=0`），绝不会把模型引向一个未挂载的工具。
- 对所有模型生效（该规则与模型无关；弱模型最需要它，
  强模型本来就会倾向于照做）。本轮不做按模型区分的开关。
- 不改动代码与机制 —— 端到端复用已发布的批量 UI。
- 措辞保持中立（按项目规则，代码与提交信息中不出现 opencode/codex 的名字）。

## 范围之外（延后）

- **在运行时把 N 次独立的 `request_user_input` 调用合并成同一个批量
  面板**（kernel 预扫描 + 合成/反合成的做法）。两个参考对象都没这么做；
  它更重也更危险（要动 turn 循环，要处理 tool_call/result-id 配对）。
  只有在真机测试表明加了 persona 规则之后 deepseek 仍然不肯批量时，
  才把它当作兜底方案保留。
- 按模型（仅 deepseek）强化 —— 先做与模型无关的版本；若基础规则
  在 deepseek 上被证明力度不够，再回来讨论。

## 测试

- Persona 单元测试：当 `request_user_input_enabled == true` 时，批量引导
  子串出现在 `coding_persona(...)` 中；为 `false` 时则不出现
  （沿用已有的开关）。
- 运行已有的 `rustcode-coding` persona 测试 —— 签名与调用点都没有变化。

## 诚实的局限

这只是一个提示词引导。它会提高模型去批量的倾向，但并不*保证*
弱模型照做（与此前几轮 persona 引导属于同一类风险）。成功标准是行为性的，
而且只能在真机上判定：deepseek/GLM 在被问到会浮现多个选项的问题时，
发出一次带 `questions[]` 的 `request_user_input`（→ Tab 表单），而不是 N 次调用。
若 deepseek 仍然不肯，就升级到已延后的运行时合并兜底方案。
