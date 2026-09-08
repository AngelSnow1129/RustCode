# WebUI 阻塞式交互停靠栏实施计划

> **致 Claude：** 必需子技能：使用 superpowers:executing-plans 按任务逐条实施本计划。

**目标：** 用统一的 composer 区域交互停靠栏取代面向轮次阻塞型审批、用户提问与策略恢复选择的全屏遮罩，使会话上下文保持可见且可滚动。

**架构：** `/chat` 与 `/live` 的请求归属、响应 API 以及终态清理保持不变。只把呈现层移入 `Chat`：应用自有的 `/chat` 权限状态向下传递，而 live 权限、结构化用户输入与策略干预沿用各自现有的状态所有者。当存在一个待处理的阻塞交互时，共享的停靠栏外壳会取代常规 composer。

**技术栈：** Preact、TypeScript、CSS、Node test runner。

---

### 任务 1：用测试锁定交互外壳契约

**文件：**
- 修改：`webui/src/lib/userInputCard.test.ts`
- 新建：`webui/src/lib/interactionDock.test.ts`

1. 补充失败的源码契约测试，证明轮次阻塞型卡片不再使用 `.modal-overlay`。
2. 补充一个失败测试，证明停靠栏取代 composer 且具备内部可滚动的正文区。
3. 运行聚焦的 WebUI 测试，确认它们因预期的旧遮罩标记而失败。

### 任务 2：新增共享停靠栏呈现

**文件：**
- 新建：`webui/src/components/InteractionDock.tsx`
- 修改：`webui/src/components/PermissionCard.tsx`
- 修改：`webui/src/components/UserInputCard.tsx`
- 修改：`webui/src/components/PolicyInterventionCard.tsx`
- 修改：`webui/src/styles/app.css`

1. 新增非模态区域外壳，带固定 header/footer 与可滚动正文。
2. 把三张阻塞型卡片全部改为共享外壳，不改变决策与响应载荷。
3. 增加自适应高度上限与移动端安全的尺寸。
4. 运行组件/源码契约测试。

### 任务 3：把所有阻塞交互路由到 composer 席位

**文件：**
- 修改：`webui/src/components/Chat.tsx`
- 修改：`webui/src/app.tsx`
- 修改：`webui/src/lib/interactionDock.test.ts`

1. 把应用自有的 `/chat` 权限请求传入 `Chat`。
2. 从 `/chat` 权限、`/live` 权限、用户输入与策略恢复中选出当前活跃的阻塞交互。
3. 挂起期间用停靠栏替换落地页与常规 composer 内容；终态清理时恢复 composer。
4. 确认常规管理对话框仍使用 `.modal-overlay`。

### 任务 4：验证行为与回归面

1. 运行聚焦的 WebUI 测试。
2. 运行 WebUI 构建/类型检查。
3. 复查最终 diff，排查意外的 Rust 或管理对话框改动。
4. 报告说明：若无可用浏览器夹具，浏览器视觉/移动端行为仍需人工截图核对。
