<!-- crates/rustcode-tuix/tests/smoke.md -->
# TUIx 验收冒烟清单

每次构建后执行人工验收。全部条目通过，才算本次重建完成。

## 构建

- [ ] `cargo build --release 2>&1 | tail -3` → 输出 Finished，0 个错误
- [ ] `cargo test -p rustcode-tuix 2>&1 | tail -5` → 79 个全部通过
- [ ] `cargo clippy -p rustcode-tuix --all-targets 2>&1 | tail -10` → 无警告

## TTY 正常路径

- [ ] `./target/release/rustcode --tuix` 显示欢迎信息（logo + 模型 + 目录 + 提示）
- [ ] `❯ ` 提示符单独占一行
- [ ] 输入 `hello` + 回车 → "❯ hello" 进入回滚区，助手回复开始流式输出
- [ ] 流式输出期间 spinner 有动画（10 帧循环）
- [ ] TextDelta 清除 spinner，并以 `  │ ` 竖条前缀打印文本
- [ ] `▸ tool(detail)` 工具行正常渲染
- [ ] `✓ summary` / `✗ summary` 工具结果配色正确
- [ ] TurnComplete → `❯ ` 回到新的一行

## CJK（必测 —— 之前的实现在这里 panic 过）

- [ ] 输入 `你好，帮我看看这个文件` + 回车 → 不 panic，光标列正确
- [ ] 助手输出中文 → 无宽度错位

## 注入防护

- [ ] 把 `\x1b[2J\x1b[H` 粘贴进输入 → 屏幕**不会**被清空
- [ ] 工具输出包含 `\x1b]0;pwned\x07` → 终端标题不变
- [ ] Bash 工具输出 `\x1b[31mred\x1b[0m` → 要么按字面文本显示、要么被剥离；不影响后续渲染

## Ctrl+C（中断）

- [ ] 流式输出期间按 Ctrl+C → 发出 Cancel，渲染器显示 "(cancelled)"
- [ ] 空闲且缓冲区为空时按 Ctrl+C → 干净退出
- [ ] 空闲但缓冲区非空时按 Ctrl+C → 只清空缓冲区，不退出

## Ctrl+Z / fg（挂起与恢复）

- [ ] 流式输出期间按 Ctrl+Z → 回到 shell，终端状态正常（非 raw mode）
- [ ] `fg` → spinner 恢复，交互恢复正常

## 斜杠命令

- [ ] `/help` → 列出全部命令（由注册表自动生成）
- [ ] `/cd /tmp` → 工作目录切换，并通知 agent
- [ ] `/clear` → 清屏并重新渲染欢迎信息
- [ ] `/status` → 显示模型 / 目录 / 配置 / token
- [ ] `/quit` → 干净退出

## Tab 补全

- [ ] `/h<Tab>` → 补全为 `/help `
- [ ] `/z<Tab>` → 无补全（无匹配项）

## 历史记录

- [ ] 退出后再次进入，`↑` 可召回此前的消息
- [ ] `~/.rustcode/history` 文件存在且可读
- [ ] 连续重复的消息会被折叠

## 管道 / 纯文本模式

- [ ] `echo "say hi" | ./target/release/rustcode --tuix` → 输出中无 ANSI 字节
- [ ] `./target/release/rustcode --tuix > out.txt` → `out.txt` 是纯文本
- [ ] `NO_COLOR=1 ./target/release/rustcode --tuix` → 无彩色；TTY 下仍保留 spinner
- [ ] `TERM=dumb ./target/release/rustcode --tuix` → 无彩色、无 spinner

## 审批流程

- [ ] 触发一个需要审批的工具 → 行内出现确认提示
- [ ] Y → 批准；agent 继续
- [ ] N → 拒绝；agent 收到拒绝结果
- [ ] A → 批准并在本次会话内记住

## 窗口缩放

- [ ] 流式输出期间缩放终端 → 已有输出保持完整，新输出按新宽度排版
- [ ] 空闲提示符状态下缩放 → 下次按键时输入重绘，且不出现错乱
