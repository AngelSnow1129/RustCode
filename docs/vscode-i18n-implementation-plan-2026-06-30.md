# VS Code i18n 实施计划

## 目标

让 VS Code 扩展中面向 IDE 的字符串跟随 VS Code 的语言区域(locale),并让 RustCode webview 依据同一个 locale 信号一致地渲染中文或英文。

## 现状

- `extensions/vscode/package.json` 中硬编码了英文的命令标题、视图名称和配置项描述。
- 扩展宿主(extension host)代码在命令、code action、状态栏 tooltip、输入框、warning 消息和斜杠命令回复中使用硬编码英文字符串。
- `extensions/vscode/webview/index.html` 硬编码了 `<html lang="en">`。
- `extensions/vscode/webview-ui/src` 既没有翻译词表,也没有翻译 hook。面向用户的文案硬编码在 React 组件中。
- 根目录的浏览器版 webui 已经有一套可用的词表模式(`webui/src/i18n.ts` 与 `webui/src/settings.tsx`),但 VS Code webview 并未复用它。
- Rust core/TUI 的 i18n 是独立的,并且应保持独立。

## 架构

- VS Code manifest 字符串使用官方的 `package.nls.json` 与 `package.nls.zh-cn.json` 机制。
- 扩展宿主运行时字符串使用 `vscode.l10n.t`。
- Webview UI 字符串使用本地 TypeScript 词表和 `I18nProvider`。
- `ChatViewProvider` 把 `vscode.env.language` 传入 webview HTML 和 `init` 消息。
- Webview 目前把 `zh`、`zh-CN`、`zh-TW` 都映射为简体中文;其他 locale 一律回退到英文。
- 发送给模型的 prompt 保持英文,除非它本身就是纯用户可见的 UI 标签。这样可以在本地化界面的同时保持模型行为稳定。

## 文件

- 新建 `extensions/vscode/package.nls.json`。
- 新建 `extensions/vscode/package.nls.zh-cn.json`。
- 新建 `extensions/vscode/webview-ui/src/i18n.ts`。
- 新建 `extensions/vscode/webview-ui/test/i18n-regression.test.ts`。
- 新建 `extensions/vscode/webview-ui/test/run-tests.js`。
- 修改 `extensions/vscode/package.json`。
- 修改 `extensions/vscode/src/chat/provider.ts`。
- 修改 `extensions/vscode/src/editor/actions.ts`。
- 修改 `extensions/vscode/src/extension.ts`。
- 修改 `extensions/vscode/src/status.ts`。
- 修改 `extensions/vscode/webview/index.html`。
- 修改 `extensions/vscode/webview-ui/src/components` 下的 React webview 组件。
- 修改 `extensions/vscode/webview-ui/src/state/types.ts` 与 `ChatProvider.tsx`,使其携带 locale。
- 修改 `extensions/vscode/webview-ui/src/utils/format.ts`,使其接受已翻译的时间/token 标签。

## 阶段

### Phase 1:测试脚手架与词表基础

验收标准:
- `npm run test:webview` 能通过 esbuild 和 Node 运行 webview 回归测试。
- 在实现之前,测试会因为缺少 locale 归一化、缺少中文字符串、缺少 manifest 本地化文件而失败。
- `i18n.ts` 导出 `normalizeLocale`、`createTranslator`、`messages`、`Lang` 和 `MsgKey`。

### Phase 2:Webview locale 接线

验收标准:
- `index.html` 不再硬编码英文语言。
- `ChatViewProvider` 把 locale 注入 HTML 和 `init`。
- `ChatState` 保存 `locale`。
- `I18nProvider` 设置 `document.documentElement.lang`。
- Webview 默认使用 VS Code 的 locale,并回退到英文。

### Phase 3:Webview 文案覆盖

验收标准:
- 首页、快捷操作、设置流程、输入区、附加菜单、文件选择器、头部 tooltip、会话列表、搜索栏、模型选择器、权限请求、工具调用标签、助手复制按钮、provider 设置以及相对时间都改用 `t()`。
- 中文文案遵循 `docs/i18n-style.md`。
- 模型/provider 名称、文件路径、命令名、API key 以及 daemon/模型输出仍作为不翻译的数据。

### Phase 4:VS Code IDE 集成

验收标准:
- `package.json` 的 contribution 字符串使用 `%key%` 占位符。
- 中英文 `package.nls` 文件覆盖每一个占位符。
- 运行时扩展字符串使用 `vscode.l10n.t`。
- 快捷操作的展示文案被本地化,而发送给模型的 prompt 保持英文。

### Phase 5:验证

验收标准:
- `npm run test:webview` 通过。
- `npm run compile` 通过。
- 搜索已知的首页英文字符串,确认它们不再硬编码在 JSX 中。
- 搜索确认 `package.json` 的 contribution 标题/描述除稳定的产品名和枚举值外,不再有硬编码。

## 不在范围内

- 翻译远端 provider 返回的 daemon API 错误载荷。
- 翻译模型输出。
- 增加一个独立于 VS Code locale 的 webview 内语言切换器。
- 把 Rust 基于枚举的 i18n 直接共享给 TypeScript。

## 风险

- VS Code 静态 contribution 字符串与 webview 运行时字符串使用不同的本地化机制。两者要保持分开。
- 过度翻译 prompt 会改变模型行为。agent prompt 要保持稳定。
- 会话管理 UI 中已经存在一些硬编码的中文。应将其移入词表,而不是当成已经完成本地化。
