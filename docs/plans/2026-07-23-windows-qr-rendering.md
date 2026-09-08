# Windows QR 渲染实施计划

> **致 Claude：** 必需子技能：使用 superpowers:executing-plans 按任务逐条实施本计划。

**目标：** 确保引导流程要么渲染出符合标准极性、可扫描的 QR 码，要么回退为可操作的 URL，而绝不显示残缺的伪 QR。

**架构：** 把可靠的紧凑 QR 渲染与通用的 UTF-8 编码支持分离。现代模拟器获得显式着色的半块（half-block）渲染器；旧版 Windows 控制台仅在完整 QR 可容纳时才使用不依赖字体的 ANSI 背景空格渲染器，否则引导流程展示 URL。所有渲染路径在返回行之前都校验宽度与高度。

**技术栈：** Rust、crossterm retained-cell SGR 解析、qrcode 0.14、rustcode-tuix 单元/集成测试。

---

### 任务 1：钉住终端能力语义

**文件：**
- 修改：`crates/rustcode-tuix/src/terminal.rs`

1. 移除代码页探测，使 UTF-8 编码无法把某个控制台判定为现代模拟器。
2. 让强制 Unicode 与旧版 conhost 分类相互独立。
3. 运行终端能力测试。

### 任务 2：新增可靠的有界 QR 渲染器

**文件：**
- 修改：`crates/rustcode-tuix/src/modals/qr.rs`

1. 补充测试：标准白底黑码极性、四模块静默区、SGR 复位、旧版回退中禁用的方块字形，以及宽度/高度拒绝。
2. 把依赖主题的 `Dense1x2` 输出替换为显式的黑/白前景色与背景色。
3. 把旧版的 `██` 回退替换为带背景色的空格。
4. 当选中的表示形式无法完整容纳时返回 `None`。
5. 运行 QR 渲染器测试。

### 任务 3：把终端边界接入引导流程

**文件：**
- 修改：`crates/rustcode-tuix/src/modals/onboarding_wizard.rs`

1. 向渲染器传入终端行数、QR 内容预算，以及 QR 专用的半块可靠性信号。
2. 当没有可靠的 QR 表示形式可容纳时，显示明确的浏览器回退文案。
3. 更新引导流程测试：紧凑 QR、旧版渲染器、80×24 URL 回退，以及强制 Unicode 的旧版 conhost。
4. 运行引导流程测试。

### 任务 4：验证与审计

**文件：**
- 仅验证上述文件；保留全部既有的脏文件。

1. 运行 `cargo test -p rustcode-tuix`。
2. 运行 `git diff --check`。
3. 检查最终 diff 是否出现 SGR 泄漏、边界错误、主题依赖，以及与用户改动的意外重叠。
