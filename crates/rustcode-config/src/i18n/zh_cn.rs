use super::messages::Msg;
use std::borrow::Cow;

pub(super) fn zh_cn(msg: Msg<'_>) -> Cow<'static, str> {
    match msg {
        Msg::WelcomeBannerLine1 =>
            "欢迎使用 {brand}，请选择一项开始：".into(),
        Msg::WelcomeBannerLine2 =>
            "（↑↓ 切换，Enter 确认，Esc 跳过）".into(),
        Msg::WelcomeOptionConfigureManually => "手动配置".into(),
        Msg::WelcomeOptionConfigureManuallyHint => "使用 API key".into(),
        Msg::WelcomeOptionSkip => "暂时跳过".into(),
        Msg::WelcomeOptionSkipHint => "稍后再说".into(),

        // ── /login（完整配置流程） ──

        Msg::ChatAuthExpired =>
            "认证已过期，请执行 /login 重新登录".into(),
        Msg::ProviderErrEntitlement403 =>
            "账号未开通该模型套餐或授权已失效（HTTP 403），请检查 API key 权限与账户状态。".into(),
        Msg::ProviderErrUnauthorized { code } =>
            format!("API key 未授权或已失效（HTTP {code}）").into(),
        Msg::ProviderErrInsufficientBalance { code } =>
            format!("账户余额不足（HTTP {code}）").into(),
        Msg::ProviderErrConnResetRetried { attempts } =>
            format!("网络连接中断：远端关闭或重置了连接，自动重连 {attempts} 次后仍失败，可重试。").into(),
        Msg::ProviderErrConnResetPartial =>
            "响应中断：为避免重复输出或工具执行，未自动重放；已保留可安全保存的部分回复，可继续。".into(),
        Msg::ProviderErrDetailLabel => "详情".into(),
        Msg::ProviderErrCorpProxyHint =>
            "此错误常见于公司网络或代理环境，请检查代理/VPN/防火墙设置后重试。".into(),
        Msg::ProviderErrProxyNamed { proxy } => format!("代理 {proxy}").into(),
        Msg::ProviderErrProxyConfigured => "配置的代理".into(),
        Msg::ProviderErrProxyUnreachable { who } => format!(
            "无法连接到{who}（代理可能未运行或地址不可达）。若你不需要代理，请运行 /proxy 并选择 no_proxy（不使用代理）后重试。"
        )
        .into(),
        Msg::ProviderErrTtfbTimeout { secs } =>
            format!("等待首字节超过 {secs}s（网关无响应）").into(),
        Msg::ProviderErrEffortUnsupported =>
            "当前模型/网关不支持「强度」（reasoning_effort）设置，已为本会话自动禁用，请重新发送。".into(),
        Msg::ToolProgressParallelEdit { count } =>
            format!("并行编辑 {count} 个文件（子代理）").into(),
        Msg::RoundCapHeader => "轮次上限".into(),
        Msg::RoundCapQuestion { cap } => format!("已运行 {cap} 轮，继续吗？").into(),
        Msg::RoundCapQuestionStats { cap, stats } =>
            format!("已运行 {cap} 轮（{stats}），继续吗？").into(),
        Msg::RoundCapContinue => "继续".into(),
        Msg::RoundCapContinueDesc { base } => format!("再跑 {base} 轮后重新确认").into(),
        Msg::RoundCapStop => "停止".into(),
        Msg::RoundCapStopDesc => "结束本回合".into(),
        Msg::GitRepoRootEmpty => "git 未返回仓库根目录".into(),
        Msg::GitCmdFailed { cmd, detail } => format!("git {cmd} 失败：{detail}").into(),
        Msg::GitOutputTooLarge { cmd, kib } =>
            format!("git {cmd} 输出超过 {kib} KiB，无法可靠展示").into(),
        Msg::GitSpawnFailed { error } => format!("无法启动 git：{error}").into(),
        Msg::GitStdoutUnavailable => "无法读取 git 输出".into(),
        Msg::GitStderrUnavailable => "无法读取 git 错误输出".into(),
        Msg::GitWaitFailed { error } => format!("等待 git 退出失败：{error}").into(),
        Msg::GitTimeout { secs } => format!("git 命令执行超过 {secs} 秒").into(),
        Msg::GitStatusPollFailed { error } => format!("查询 git 状态失败：{error}").into(),
        Msg::GitStdoutThreadPanicked => "读取 git 输出的线程异常退出".into(),
        Msg::GitStdoutReadFailed { error } => format!("读取 git 输出失败：{error}").into(),
        Msg::GitStderrThreadPanicked => "读取 git 错误输出的线程异常退出".into(),
        Msg::GitStderrReadFailed { error } => format!("读取 git 错误输出失败：{error}").into(),
        Msg::GitNumstatMissingPath => "git numstat 缺少文件路径".into(),
        Msg::GitNumstatMissingOldPath => "git numstat 缺少重命名前路径".into(),
        Msg::GitNumstatMissingNewPath => "git numstat 缺少重命名后路径".into(),
        Msg::GitNumstatMissingCount => "git numstat 缺少行数".into(),
        Msg::GitNumstatCountNotUtf8 => "git numstat 行数不是 UTF-8".into(),
        Msg::GitNumstatCountInvalid { text } => format!("git numstat 行数无效：{text}").into(),
        Msg::GoalCapRound { max } => format!("已达轮数预算（{max} 轮），继续对话即推进").into(),
        Msg::GoalCapRoundNoMax => "已达轮数预算，继续对话即推进".into(),
        Msg::GoalCapTime => "已达时间上限，继续对话即推进".into(),
        Msg::GoalCapStopped { other } => format!("已停止（{other}），继续对话即推进").into(),
        Msg::RetryReasonRateLimited => "请求过于频繁或额度已用尽".into(),
        Msg::RetryReasonUpstream => "上游服务暂时不可用".into(),
        Msg::RetryReasonTimeout => "模型响应超时".into(),
        Msg::RetryReasonNetwork => "网络连接失败".into(),
        Msg::TuixProviderRetry {
            reason,
            backoff_secs,
            attempt,
            max_attempts,
        } => format!(
            "API 错误 {reason}，{backoff_secs} 秒后重试（{attempt}/{max_attempts}）..."
        )
        .into(),
        Msg::TuixLoopUsage =>
            "用法：/loop <间隔> <prompt 或 /命令>，例 /loop 5m /diff".into(),
        Msg::TuixLoopSelfRef => "不能对 /loop 自身循环".into(),
        Msg::TuixLoopIntervalRange => "间隔需在 10s-24h 之间".into(),
        Msg::TuixRateLimitAutoResume { secs } =>
            format!("⏳ 限流，{secs}s 后自动继续...").into(),
        Msg::TuixRateLimit429 { reason, tail } => format!(
            "⏸ 限流（HTTP 429）{reason}{tail} · 已保留已完成内容 · 稍后重试或换模型"
        )
        .into(),
        Msg::TuixRateLimitRetryAfter { dur } => format!("（约 {dur} 后可重试）").into(),
        Msg::TuixRateLimitWindowNoTime { tail } => format!(
            "⏸ 5小时窗口已用尽，稍后恢复{tail} · 已保留已完成内容 · 可换模型或稍后重试"
        )
        .into(),
        Msg::TuixRateLimitWindowWithTime { reset_at, tail } => format!(
            "⏸ 5小时窗口已用尽，约 {reset_at} 恢复{tail} · 已保留已完成内容 · 可换模型或稍后重试"
        )
        .into(),
        Msg::TuixRateLimitWindowRemaining { dur } => format!("（还有 {dur}）").into(),
        Msg::TuixToolBatchSameParallel { count, tool } =>
            format!("并行运行 {count} 个 {tool} 调用").into(),
        Msg::TuixToolBatchSame { count, tool } => format!("运行 {count} 个 {tool} 调用").into(),
        Msg::TuixToolBatchParallel { count } => format!("并行运行 {count} 个工具").into(),
        Msg::TuixToolBatch { count } => format!("运行 {count} 个工具").into(),
        Msg::TuixRuntimeDeliveryFailed { operation } =>
            format!("运行时 {operation} 事件投递失败").into(),
        Msg::TuixShellContextQueueFailed =>
            "[将 shell 输出加入运行时上下文失败]".into(),
        Msg::TuixProviderReloadStartFailed { error } =>
            format!("无法启动 provider 重载：{error}").into(),
        Msg::TuixConfigRollbackFailed { error } =>
            format!("；配置回滚失败：{error}").into(),
        Msg::TuixProviderNoLongerAvailable { name } =>
            format!("provider「{name}」已不可用").into(),
        Msg::TuixFoldNoOutput => "（无输出）".into(),
        Msg::TuixFoldLinesSuffix { count } => format!("（{count} 行）").into(),
        Msg::KernelNoticeEmptyRetryMalformed {
            wait_secs,
            attempt,
            max,
        } => format!("响应格式异常，{wait_secs} 秒后重试（{attempt}/{max}）...").into(),
        Msg::KernelNoticeEmptyRetryEmpty {
            wait_secs,
            attempt,
            max,
        } => format!("模型返回空响应，{wait_secs} 秒后重试（{attempt}/{max}）...").into(),
        Msg::KernelNoticeReplyTruncated =>
            "模型这次回复达到了长度上限，内容可能没写完。可以让它「继续」，会接着把剩下的部分补完。".into(),
        Msg::KernelNoticeOverWindow { est_k, window_k } => format!(
            "请求约 {est_k}K tokens 接近当前模型可用上限（窗口约 {window_k}K，需为回复预留空间）：请精简输入或换用更大窗口的模型。"
        )
        .into(),
        Msg::KernelNoticeEmptyExhMalformed { max_retries } => format!(
            "模型连续 {max_retries} 次返回无法解析的响应（上游偶发）。可直接重试，或稍后再试。"
        )
        .into(),
        Msg::KernelNoticeEmptyExhOverWindowBrief { max_retries } => format!(
            "模型连续 {max_retries} 次返回空响应。如开头所述，本次请求已超过模型上下文窗口----请精简输入或 /compact 后重试。"
        )
        .into(),
        Msg::KernelNoticeEmptyExhOverWindowFull {
            max_retries,
            est_k,
            window_k,
        } => format!(
            "模型连续 {max_retries} 次返回空响应。当前请求约 {est_k}K tokens，已接近或超过模型上下文窗口（约 {window_k}K），很可能是请求过大所致。建议 /compact 或精简输入后重试。"
        )
        .into(),
        Msg::KernelNoticeEmptyExhTransient { max_retries } => format!(
            "模型连续 {max_retries} 次返回空响应（上游偶发，与上下文长度无关）。可直接重试，或稍后再试。"
        )
        .into(),
        Msg::NetworkConnectHint =>
            "网络连接失败。若浏览器能打开，可能是代理/防火墙差异：用 /proxy 配置代理或设置 HTTPS_PROXY，或在浏览器打开上面的登录链接完成扫码。可按 Esc 跳过，稍后 /login 重试。".into(),
        Msg::ErrUnsupportedLocale { input } =>
            format!("不支持的语言：{input}").into(),

        // ── 状态栏 ──
        Msg::StatusNoProvider =>
            "未配置 Provider . 使用 /provider 配置".into(),
        Msg::StatusRuntimeUnavailable =>
            "Runtime 不可用 . 请重启或查看上方错误".into(),
        Msg::StatusUpgradeHint { version } =>
            format!("↑ {version} 可用 . 使用 /upgrade 升级").into(),
        Msg::StatusUpgradeHintPm { version } =>
            format!("↑ {version} 可用 . 运行 brew upgrade rustcode 升级").into(),
        Msg::StatusModelNotConfigured =>
            "（未配置）".into(),
        Msg::StatusClipboardImageHint =>
            "剪贴板有图片 . ctrl+v / ctrl+alt+v 粘贴".into(),
        Msg::StatusClipboardImageHintSlash =>
            "剪贴板有图片 . /paste 粘贴".into(),
        Msg::StatusWebuiHint =>
            "提示：使用 /webui 在浏览器中打开 {brand}".into(),

        // ── /status 命令主体 ──
        Msg::StatusBody { model, dir, config } =>
            format!(
                "  模型：    {}\n  目录：    {}\n  配置文件：{}\n",
                model, dir, config,
            ).into(),


        Msg::StatusInstructionFilesHeader =>
            "  指令文件：\n".into(),
        Msg::StatusInstructionScopeGlobal => "用户全局".into(),
        Msg::StatusInstructionScopeProject => "项目共享".into(),
        Msg::StatusInstructionScopeUser => "用户项目覆盖".into(),
        Msg::StatusInstructionPresent { path, label, scope } =>
            format!("    [+] {scope}（{label}）：{path}\n").into(),
        Msg::StatusInstructionMissing { path, label, scope } =>
            format!("    [x] {scope}（{label}）：{path} -- 未找到\n").into(),
        Msg::StatusMemoryFilesHeader => "  记忆文件：\n".into(),
        Msg::StatusMemoryScopeGlobal => "用户全局".into(),
        Msg::StatusMemoryScopeProject => "项目记忆".into(),
        Msg::StatusMemoryScopeLocal => "本机记忆".into(),
        Msg::StatusMemoryPresent { path, scope } =>
            format!("    [+] {scope}：{path}\n").into(),
        Msg::StatusMemoryMissing { path, scope } =>
            format!("    [x] {scope}：{path} -- 未找到\n").into(),

        // ── 帮助 ──
        Msg::HelpAvailableCommands =>
            "  可用命令：\n".into(),
        Msg::KeybindingsHelp => r#"  键盘快捷键

  ── 输入 ──
    Enter                            发送消息
    \ 后接 Enter                     插入换行（所有终端通用）
    Shift / Alt / Ctrl+Enter         插入换行 *
    Ctrl+J                           插入换行 *
    /                                打开斜杠命令菜单
    Tab                              接受斜杠菜单、文件路径等补全
    Backspace / Ctrl+H               删除上一个字符
    Delete / Ctrl+?                  删除下一个字符
    Ctrl+W                           删除前一个单词
    Ctrl+U                           清空当前行
    Ctrl+K                           删除到行尾
    Ctrl+A / Home                    跳到行首
    Ctrl+E / End                     跳到行尾
    Left / Right                     光标左右移动

  ── 历史 ──
    Up / Down                        上一条 / 下一条输入
    Ctrl+R                           反向搜索；再次按下继续向前查找
    Right                            接受下一步建议（不会自动发送）

  ── 模式与模型 ──
    Shift+Tab                        无补全菜单时切换到下一个执行模式
    F2 / Shift+F2                    下一个 / 上一个模型（Mac：Fn+F2 / Fn+Shift+F2）
    Ctrl+T                           切换 reasoning_effort

  ── 翻看输出 ──
    Shift+Up / Shift+Down            向上 / 向下滚动一行
    PageUp / PageDown                向上 / 向下滚动 10 行
    Alt+Up / Alt+Down                跳到上一条 / 下一条消息
    Ctrl+Up / Ctrl+Down              跳到上一条 / 下一条用户消息
    Home / End                       空输入时跳到对话顶部 / 底部
    鼠标滚轮                         滚动聊天区
    鼠标拖选                         选择文本
    Shift+鼠标拖选                   使用终端原生文本选择
    Ctrl+Shift+C                     复制 {brand} 选区

  ── 流程控制 ──
    Esc                              清空输入 / 关闭弹层 / 取消当前操作
    Esc Esc                          空闲且输入为空时撤销上一轮
    Ctrl+C                           取消当前操作；空闲时再次按下退出
    Ctrl+O                           切换工具实时输出
    Ctrl+V / Ctrl+Alt+V              粘贴文本或图片 **

  ── 斜杠菜单 / 弹层导航 ──
    Up / Down                        移动选择
    Enter                            确认
    Esc                              取消 / 关闭弹层
    Tab                              在斜杠菜单中插入当前高亮命令
    1..9                             审批 / 交互提问中选择对应选项
    y / a / n                        审批时允许一次 / 始终允许 / 拒绝

  * 组合键需要终端能区分修饰键。目前已知支持的有：
     Kitty / WezTerm / iTerm2（启用 Report Modifiers）/
     Windows Terminal / Ghostty / Warp。其他终端（包括 macOS
     Apple Terminal、默认 xterm、GNOME Terminal、VS Code 集成
     终端）不区分 Shift+Enter 与 Enter，请用 \ + Enter。
  ** Ctrl+Alt+V 是终端拦截 Ctrl+V 时的备用键；Ctrl+Shift+V
     保留给终端原生纯文本粘贴。

  提示：输入 /help 查看完整斜杠命令列表。
"#.into(),

        // ── Provider 向导 ──
        Msg::ProviderWizardHeader =>
            "  管理 Provider：添加、编辑、删除或设置全局默认。按 Esc 取消。\n".into(),
        Msg::ProviderWizardCancelled =>
            "（已取消）".into(),
        Msg::ProviderMenuAdd => "添加".into(),
        Msg::ProviderMenuAddDesc => "新建 Provider 配置".into(),
        Msg::ProviderMenuEdit => "编辑".into(),
        Msg::ProviderMenuEditDesc => "修改已有 Provider 配置".into(),
        Msg::ProviderMenuDelete => "删除".into(),
        Msg::ProviderMenuDeleteDesc => "删除已有 Provider 配置".into(),
        Msg::ProviderMenuSetDefault => "设为全局默认".into(),
        Msg::ProviderMenuSetDefaultDesc => "设置默认 Provider，并切换当前会话".into(),
        Msg::ProviderImportPrompt =>
            "粘贴模板自动识别（curl / JSON / TOML），或直接回车手动填写：".into(),
        Msg::ProviderImportParsed { base_url, type_name, model } =>
            format!("已识别：{base_url} . {type_name} . {model}").into(),
        Msg::ProviderImportFailed =>
            "未能识别为模板，请重贴 curl / JSON / TOML，或留空回车手动填写。".into(),
        Msg::ProviderNoProviders =>
            "尚未配置任何 Provider。".into(),
        Msg::ProviderDeleteConfirm { name } =>
            format!("删除 \"{name}\"？[y/N]").into(),
        Msg::ProviderDeleted { name } =>
            format!("已删除 \"{name}\"。").into(),
        Msg::ProviderDeleteKept => "（已保留）".into(),
        Msg::ProviderDefaultSet { name } =>
            format!("默认已设为 {name}。").into(),
        Msg::ProviderAdded { name } =>
            format!("已添加账号 \"{name}\"。已进入其模型列表，按 Ctrl+A 添加模型。").into(),
        Msg::ProviderUpdated { name } =>
            format!("已更新 \"{name}\"。").into(),
        Msg::ProviderStepName => "Provider 名称？".into(),
        Msg::ProviderStepType => "类型？（openai / claude / ollama）".into(),
        Msg::ProviderStepTypeWithHint { current } =>
            format!("类型？[{current}]（openai / claude / ollama，留空保持不变）").into(),
        Msg::ProviderStepBaseUrl =>
            "Base URL？（例：https://api.example.com/v1 —— 你的第三方服务商地址）".into(),
        Msg::ProviderStepBaseUrlWithHint { current } =>
            format!("Base URL？[{current}]（留空保持不变）").into(),
        Msg::ProviderDefaultHint => "Provider 默认值".into(),
        Msg::ProviderStepApiKey =>
            "API 密钥？（留空不设置）".into(),
        Msg::ProviderStepApiKeyWithHint { hint } =>
            format!("API 密钥？[{hint}]").into(),
        Msg::ProviderStepApiKeySet => "已设置 -- 留空保持不变".into(),
        Msg::ProviderStepApiKeyUnset => "未设置".into(),
        Msg::ProviderStepModel => "模型？".into(),
        Msg::ProviderStepModelWithHint { current } =>
            format!("模型？[{current}]（留空保持不变）").into(),
        Msg::ProviderStepContextWindow { default } =>
            format!("上下文窗口？[{default}] tokens（留空使用默认值；如 128000 / 256000 / 512000 / 1000000，或 128k / 1m）").into(),
        Msg::ProviderStepContextWindowWithHint { current } =>
            format!("上下文窗口？[{current}] tokens（留空保持不变；如 128000 / 256000 / 512000 / 1000000，或 128k / 1m）").into(),
        Msg::ProviderContextWindowInvalid =>
            "上下文窗口必须是正整数 tokens，例如 128000 或 128k。".into(),
        Msg::ProviderNameEmpty => "名称不能为空。".into(),
        Msg::ProviderBaseUrlEmpty => "Base URL 不能为空。".into(),
        Msg::ProviderUnknownType =>
            "未知类型。请选择 openai / claude / ollama。".into(),
        Msg::ProviderUnknownTypeEdit =>
            "未知类型。请选择 openai / claude / ollama 或留空。".into(),
        Msg::ProviderModelEmpty => "模型不能为空。".into(),
        Msg::ProviderEditKeep => "（保持不变）".into(),
        Msg::ProviderTypeInferred { type_name } =>
            format!("已识别类型：{type_name}").into(),
        Msg::ProviderStepNameDefault { default } =>
            format!("Provider 名称？[{default}]（留空使用此名）").into(),
        Msg::ProviderStepProgress { current, total } =>
            format!("（{current}/{total}）").into(),

        // ── Provider 面板 ──
        Msg::ProviderPanelTabAccounts => "账号".into(),
        Msg::ProviderPanelTabModels => "模型".into(),
        Msg::ProviderPanelEmptyAccounts =>
            "（尚无 Provider 账号 -- 按 Ctrl+A 添加）".into(),
        Msg::ProviderPanelNoMatchingAccounts => "（无匹配的 Provider 账号）".into(),
        Msg::ProviderPanelEmptyModels =>
            "（尚无模型 -- 按 Ctrl+A 添加）".into(),
        Msg::ProviderPanelNoMatchingModels => "（无匹配的模型）".into(),
        Msg::ProviderPanelLegacyBadge => "旧".into(),
        Msg::ProviderPanelDefaultBadge => "默认".into(),
        Msg::ProviderPanelModelCount { count } => format!("{count} 个模型").into(),
        Msg::ProviderPanelAddModelRow => "＋ 添加模型".into(),
        Msg::ProviderPanelAccountsHint =>
            "筛选 . ↑↓选择 . ↵模型 . Ctrl+A添加 . Ctrl+E编辑 . Ctrl+Dx2 删除 . Tab切换 . Esc关闭".into(),
        Msg::ProviderPanelModelsHint =>
            "筛选 . ↑↓选择 . ↵默认/添加 . Ctrl+A添加 . Ctrl+E编辑 . Ctrl+Dx2 删除 . Tab切换 . Esc关闭".into(),
        Msg::ProviderPanelFilteredModelsHint { account } =>
            format!("〔{account}〕. ↑↓选择 . ↵默认/添加 . Ctrl+A加模型 . Ctrl+E编辑 . Ctrl+Dx2 删除 . Tab全部 . Esc关闭").into(),
        Msg::ProviderPanelModelSaved { model } => format!("已保存模型“{model}”。").into(),
        Msg::ProviderPanelAddTitle => "【添加 Provider 账号】".into(),
        Msg::ProviderPanelEditAccountTitle { account } =>
            format!("【编辑账号 {account}】").into(),
        Msg::ProviderPanelAddModelTitle => "【添加模型】".into(),
        Msg::ProviderPanelEditModelTitle => "【编辑模型】".into(),
        Msg::ProviderPanelFieldVendor => "厂商".into(),
        Msg::ProviderPanelFieldAccount => "账号".into(),
        Msg::ProviderPanelFieldBaseUrl => "Base URL".into(),
        Msg::ProviderPanelFieldApiKey => "API 密钥".into(),
        Msg::ProviderPanelFieldModel => "模型".into(),
        Msg::ProviderPanelFieldVision => "图片输入".into(),
        Msg::ProviderPanelVisionAuto => "自动".into(),
        Msg::ProviderPanelVisionEnabled => "启用".into(),
        Msg::ProviderPanelVisionDisabled => "禁用".into(),
        Msg::ProviderPanelFieldEffort => "默认思考强度".into(),
        Msg::ProviderPanelFieldEffortLevels => "支持档位".into(),
        Msg::ProviderPanelFieldWindow => "上下文窗口".into(),
        Msg::ProviderPanelFieldMakeDefault => "设为默认".into(),
        Msg::ProviderPanelSwitchHint => "←-> 切换".into(),
        Msg::ProviderPanelEnvHint { env } => format!("留空使用 ${env}").into(),
        Msg::ProviderPanelDefaultValue => "默认".into(),
        Msg::ProviderPanelKeepOriginal => "留空保留原值".into(),
        Msg::ProviderPanelProviderFormHint =>
            "Tab 下一项  ←-> 切厂商  空格 勾选  ↵ 保存  Esc 返回".into(),
        Msg::ProviderPanelAccountFormHint => "Tab 切换  ↵ 保存  Esc 返回".into(),
        Msg::ProviderPanelModelFormHint =>
            "Tab 下一项  ←-> 切选项  空格切换  ↵ 保存  Esc 返回".into(),
        // ── Model 选择器 ──
        Msg::ModelSwitched { provider, model } =>
            format!("  当前会话已切换到 {provider} . {model}\n").into(),
        Msg::ModelSwitchedAndDefault { provider, model } =>
            format!("  已切换到 {provider} . {model}；已设为新会话默认\n").into(),

        // ── 会话选择器 ──
        Msg::SessionLoadFailed { error } =>
            format!("加载会话失败：{error}").into(),
        Msg::SessionResumeInProgress => "另一个会话恢复仍在进行中".into(),
        Msg::SessionPrepJoinFailed { error } =>
            format!("会话准备任务失败：{error}").into(),
        Msg::SessionNotFoundById { session_id } =>
            format!("会话 {session_id} 不存在").into(),
        Msg::SessionPrepTimeout =>
            "会话准备耗时异常长（会话过大或磁盘较慢），请重试".into(),
        Msg::ProjectFallbackWord => "项目".into(),
        Msg::SessionResumedLabel { name } =>
            format!("已恢复：{name}").into(),
        Msg::SessionBusyForked { source_id, fork_id } =>
            format!(
                "最近会话（{source_id}）正在另一个窗口运行，已从其最后提交状态创建独立分支（{fork_id}）。"
            ).into(),

        // ── 待办面板 ──
        Msg::TodoPanelTitle => "待办".into(),
        Msg::TodoPanelCompleted { n } => format!("{n} 已完成").into(),
        Msg::TodoPanelMore { n } => format!("+{n} 更多...").into(),

        // ── 审批面板 ──
        Msg::ApprovalAllowOnce => "允许一次".into(),
        Msg::ApprovalAlwaysAllow { tool } => format!("本会话总是允许 {tool}").into(),
        Msg::ApprovalAlwaysAllowFolder => "本会话总是允许写入此目录".into(),
        Msg::ApprovalAlwaysAllowCommand => "本会话总是允许此命令".into(),
        Msg::ApprovalDeny => "拒绝".into(),
        Msg::ApprovalHint => "↑↓ 选择 . Enter 确认 . Esc 取消".into(),
        Msg::ApprovalHeader { tool, detail } => {
            if detail.is_empty() {
                format!("允许 {tool}？").into()
            } else {
                format!("允许 {tool}（{detail}）？").into()
            }
        }
        Msg::CredentialApprovalNote => "[!] 可能把凭据或敏感内容发送给模型 Provider".into(),
        Msg::ToolDenied => "已拒绝".into(),
        Msg::ToolBlockedBySecurityPolicy =>
            "安全策略已阻止工具调用：凭据不能通过通用 shell 参数、临时文件或环境变量传递".into(),
        Msg::PolicyRecoveryHeader => "需要安全决策".into(),
        Msg::PolicyRecoveryQuestion => "凭据操作已被拦截，{brand} 下一步应如何处理？".into(),
        Msg::PolicyRecoveryComplete => "我已在外部完成".into(),
        Msg::PolicyRecoveryCompleteDesc => "确认外部操作已完成，不自动调用模型".into(),
        Msg::PolicyRecoverySkip => "跳过该步骤".into(),
        Msg::PolicyRecoverySkipDesc => "跳过认证步骤并结束本次恢复，不自动调用模型".into(),
        Msg::PolicyRecoveryInstructions => "查看安全执行指引".into(),
        Msg::PolicyRecoveryInstructionsDesc => "仅展示本地固定指引和占位符".into(),
        Msg::PolicyRecoveryEnd => "结束任务".into(),
        Msg::PolicyRecoveryEndDesc => "关闭本次介入，不调用模型".into(),
        Msg::PolicyRecoverySafeInstructions => "安全手动路径：\n  1. 打开一个由你控制的独立终端。\n  2. 使用服务自身的登录流程或凭据感知的专用工具。\n  3. 在该终端完成认证操作；不要把密钥粘贴到 {brand}。\n  4. 返回这里并选择“我已在外部完成”。\n{brand} 不会重构或展示刚才被拒绝的命令。".into(),
        Msg::PolicyRecoveryCompletedLocally => "已确认认证步骤在外部完成。为避免重新生成敏感命令，本次恢复未调用模型；你可以继续提交不涉及凭据的任务。".into(),
        Msg::PolicyRecoverySkippedLocally => "已跳过需要访问凭据的步骤。为避免重新生成敏感命令，本次恢复未调用模型；你可以继续提交其他安全任务。".into(),
        Msg::PolicyRecoverySubmitError => "无法提交安全恢复选择，请重试。".into(),

        Msg::CmdSwitchedAutoMode => "  已切换到自动模式(所有工具自动批准)。\n".into(),
        Msg::CmdSwitchedAcceptEditsMode => "  已切换到自动接受编辑模式(文件编辑免审批;bash 仍会询问)。\n".into(),

        Msg::SessionTimeJustNow => "刚刚".into(),
        Msg::SessionTimeMinAgo { n } => format!("{n}分钟前").into(),
        Msg::SessionTimeHourAgo { n } => format!("{n}小时前").into(),
        Msg::SessionTimeDayAgo { n } => format!("{n}天前").into(),
        Msg::SessionMsgCount { count } =>
            format!("{count} 条消息").into(),
        Msg::SessionNameEmpty =>
            "会话名不能为空".into(),
        Msg::SessionNameTooLong { max } =>
            format!("会话名过长（最多 {max} 个字符）").into(),
        Msg::SessionNameControlChars =>
            "会话名不能包含控制字符".into(),
        Msg::SessionListFailed { error } =>
            format!("列出会话失败：{error}").into(),
        Msg::SessionRenamed { old, new } =>
            format!("  已重命名：'{old}' -> '{new}'").into(),
        Msg::SessionSaveFailed { error } =>
            format!("保存会话失败：{error}。未持久化新名称。").into(),
        Msg::SessionNoneSelected =>
            "未选中会话".into(),
        Msg::SessionPickerHint =>
            "↑↓ 移动 . Enter 打开 . Ctrl+D[x]2 删除 . 输入内容搜索 . Esc 取消".into(),
        Msg::SessionPickerTitle { n, total, project } =>
            format!("恢复会话（{n}/{total} . {project}）").into(),
        Msg::SessionPickerTitleBare =>
            "恢复会话".into(),
        Msg::SessionPickerEmptyProject =>
            "（此项目暂无会话）".into(),
        Msg::SessionPickerEmptyFilter =>
            "（无匹配会话）".into(),
        Msg::SessionPickerEmptyFilterQuery { query } =>
            format!("（无匹配 \"{query}\" -- Backspace 清除）").into(),
        Msg::SessionDeleted { name } =>
            format!("「{name}」已删除").into(),
        Msg::SessionDeleteConfirm { name } =>
            format!("再按 Ctrl+D 确认删除「{name}」").into(),
        Msg::SessionDeleteFailed { error } =>
            format!("删除会话失败：{error}").into(),


        // ── 目录选择器 ──
        Msg::DirPickerTitle { n, total } =>
            format!("切换工作目录（{n}/{total}）").into(),
        Msg::DirPickerHint =>
            "↑↓ 移动 . Tab 补全 . Enter 进入 . 输入内容搜索/路径 . Esc 取消".into(),
        Msg::DirPickerEmptyPath { query } =>
            format!("没有匹配的历史目录「{query}」. Enter 按路径进入").into(),
        Msg::DirCurrent => "当前".into(),
        Msg::DirNotExists { path } =>
            format!("目录已不存在：{path}").into(),
        Msg::DirChanged { path } =>
            format!("  已切换到：{path}\n").into(),
        Msg::DirNotADirectory { path } =>
            format!("不是目录：{path}").into(),
        Msg::CdHomeUnknown => "主目录未知".into(),
        Msg::CdNoPrevious => "没有上一个目录".into(),

        // ── 语言 ──
        Msg::LanguageSwitched { label, locale } =>
            format!("  [+] 已切换语言为 {label}（{locale}）。\n").into(),

        // ── 空闲/引导提示 ──
        Msg::IdleHintPrefix =>
            "输入内容，或按 ".into(),
        Msg::IdleHintSlash => "/".into(),
        Msg::IdleHintSuffix =>
            " 浏览命令".into(),
        Msg::IdleHintFull =>
            "输入内容，或按 / 浏览命令".into(),
        Msg::IdleHintProvider => "/provider".into(),
        Msg::IdleHintProviderSuffix =>
            "添加自定义模型".into(),
        Msg::IdleHintProviderFull =>
            "使用 /provider 添加自定义模型".into(),
        Msg::IdleHintWebui => "/webui".into(),
        Msg::IdleHintWebuiSuffix =>
            "在浏览器中同步会话".into(),
        Msg::IdleHintWebuiFull =>
            "使用 /webui 在浏览器中同步会话".into(),

        // ── 欢迎屏幕提示 ──
        Msg::WelcomeTipsHeading => "上手提示".into(),

        Msg::WelcomeTipProvider => "添加自定义模型".into(),
        Msg::WelcomeTipModel => "设置默认模型".into(),
        Msg::WelcomeTipResume => "恢复上次会话".into(),
        Msg::WelcomeTipSetup => "一键推荐配置".into(),
        Msg::WelcomeTipSkills => "浏览可用技能".into(),
        Msg::WelcomeTipPlugin => "安装技能/命令插件".into(),
        Msg::WelcomeTipWebui => "在浏览器打开同步会话".into(),
        Msg::WelcomeTipMcp => "接入 MCP 工具".into(),
        Msg::WelcomeTipPlan => "只读规划模式".into(),
        Msg::WelcomeTipSession => "管理与切换会话".into(),
        Msg::WelcomeTipLoop => "循环执行提示词".into(),
        Msg::WelcomeTipGoal => "为本次会话设定目标".into(),
        Msg::WelcomeTipInit => "扫描代码库生成 AGENTS.md".into(),
        Msg::WelcomeTipLanguage => "切换界面语言".into(),
        Msg::WelcomeTipUsage => "查看用量与额度".into(),

        // ── 斜杠命令 ──
        Msg::CmdSwitchedPlanMode =>
            "  已切换到 Plan 模式（只读探索）。\n".into(),
        Msg::CmdSwitchedBuildMode =>
            "  已切换到 Build 模式（完整执行）。\n".into(),
        Msg::CmdNewSession =>
            "  新会话已开始。\n".into(),
        Msg::CmdSessionTransitionPending =>
            "  Runtime 正在重配置；就绪前会保留当前输入。\n".into(),
        Msg::CmdSessionTransitionFailed { error } =>
            format!("会话切换失败，原会话仍可用：{error}").into(),
        Msg::CmdCapabilityReloadFailed { error } =>
            format!("Runtime 能力重载失败，原 Runtime 仍可用：{error}").into(),
        Msg::CmdNoProviders =>
            "  未配置任何 Provider。\n".into(),
        Msg::CmdSessionListLoading =>
            "  正在加载会话列表...\n".into(),
        Msg::CmdNoSessions =>
            "  未找到历史会话。请先开始一段对话。\n".into(),
        Msg::CmdUnknownCommand { name } =>
            format!("未知命令：/{name}").into(),
        Msg::CmdCustomArgRequired { name } =>
            format!("/{name} 需要提供参数。用法：/{name} <你的输入>").into(),

        Msg::CmdLogoutDone =>
            "  已退出登录。权限已刷新。\n".into(),
        Msg::CmdLogoutFailed { error } =>
            format!("退出登录失败：{error}").into(),
        Msg::CmdWhoamiNotSignedIn =>
            "  尚未登录。使用 /login 进行认证。\n".into(),
        Msg::CmdWhoamiNotSignedInNeutral =>
            "  尚未登录。此构建无托管账号 -- 用 /provider 配置第三方供应商（自带 API Key）。\n"
                .into(),
        Msg::CmdReloadDone { provider, model } =>
            format!("  配置已重载。当前：{provider} . {model}\n").into(),
        Msg::CmdReloadFailed { error } =>
            format!("重载失败：{error}（保留先前配置）").into(),
        Msg::CmdUndoNotSupported =>
            "  撤销功能暂不支持。\n".into(),
        Msg::CmdUndoDone { target, last } =>
            format!("  ↩ 已退回到第 {target} 轮之前（删除第 {target}~{last} 轮）。你的提示词已填回输入框。\n").into(),
        Msg::CmdUndoDiskWarning =>
            "  [!] 仅回滚了对话记忆，磁盘文件未恢复。如需还原代码，请手动处理或用 /diff 查看。\n".into(),
        Msg::CmdUndoNoTurns =>
            "  没有可撤销的轮次。\n".into(),
        Msg::CmdUndoOutOfRange { requested, available } =>
            format!("  无效的轮次 {requested}（当前共 {available} 轮）。\n").into(),
        Msg::CmdUndoBusy =>
            "  当前回合进行中，无法撤销----请先按 Esc 取消。\n".into(),
        Msg::CmdRewindBusy =>
            "  当前回合进行中，无法回退----请先按 Esc 取消。\n".into(),
        Msg::CmdRewindUnavailable => "暂时无法打开回退".into(),
        Msg::CmdUndoBadArg =>
            "  用法：/undo 或 /undo N（N 为轮次号）。\n".into(),
        Msg::CmdNoChanges =>
            "  （无变更）\n".into(),
        Msg::CmdDiffTruncated =>
            "  ... diff 输出已截断\n".into(),
        Msg::CmdDiffUntracked => "未跟踪".into(),
        Msg::CmdDiffBinary => "二进制".into(),
        Msg::CmdDiffSummary { files, additions, deletions } =>
            format!("{files} 个文件已修改，+{additions} -{deletions}").into(),
        Msg::CmdCheckingUpdate =>
            "  正在检查更新...\n".into(),
        Msg::CmdNoActiveProvider =>
            "未配置活跃的 Provider。使用 /provider 添加一个。".into(),
        Msg::CmdNoModelConfigured =>
            "未配置模型；请先运行 /provider 添加第三方 API 密钥".into(),
        Msg::CmdProviderUnavailable =>
            "Provider 当前不可用。请使用 /login 登录，或用 /provider 配置。".into(),
        Msg::CmdProviderUnavailableNeutral =>
            "Provider 当前不可用。请用 /provider 配置第三方 Provider（自带 API Key）。".into(),
        Msg::CmdProviderUnsupportedBuild =>
            "当前构建无法接入网关。请使用带网关支持的发行版构建，或用 /provider 配置第三方 Provider（自带 API Key）。".into(),
        Msg::CmdProviderReloading =>
            "正在切换 Provider/模型，请等待切换完成后再发送。".into(),
        Msg::SubmitHeldUntilProviderReady =>
            "  ↳ provider 尚未就绪，消息已排队，就绪后将自动发送\n".into(),


        // ── 审批提示 ──
        Msg::ApprovalPromptAlt { tool, detail } =>
            format!("允许 {}（{}）？[Y]是=回车 / [N]否 / [A]总是", tool, detail).into(),
        Msg::ApprovalWaitingLabel =>
            "> 等待审批：".into(),
        Msg::ApprovalAllow => " 允许  ".into(),
        Msg::ApprovalAlways => " 总是  ".into(),

        // ── 取消 / 错误前缀 ──
        Msg::Cancelled => "（已取消）".into(),
        Msg::ErrorPrefix { msg } =>
            format!("[错误：{msg}]").into(),

        // ── 升级 ──
        Msg::UpgradeSuccess { from, to } =>
            format!("  [+] 已升级 {} -> {}\n", from, to).into(),
        Msg::UpgradeManifestFetched { version } =>
            format!("  最新版本: {}\n", version).into(),
        Msg::UpgradeDownloading { pct, bytes, total } =>
            format!("  下载中 {}% ({} / {} bytes)\n", pct, bytes, total).into(),
        Msg::UpgradeVerifying =>
            "  正在校验 SHA256\n".into(),
        Msg::UpgradeReplacing =>
            "  正在替换二进制文件\n".into(),
        Msg::UpgradeDone { version, backup } =>
            format!("\n[+] 已升级到 {}（旧版本保留为 {}）\n  正在重启新版本...\n", version, backup).into(),
        Msg::UpgradeAlreadyLatest { current, latest } =>
            format!(
                "  [+] 已是最新版本，无需更新（当前 {}，远端最新 {}）。如需重装请加 --force。\n",
                current, latest
            ).into(),
        Msg::UpgradeFailed { error } =>
            format!("升级失败: {}", error).into(),
        Msg::UpgradeRolledBack { exe, backup } =>
            format!("\n[+] 已回滚。当前二进制: {}；另一版本保存在 {}\n  正在重启回滚版本...\n", exe, backup).into(),
        Msg::UpgradeReplaceRestored { error } =>
            format!("替换为新二进制失败（{error}）。已恢复为先前版本。").into(),
        Msg::UpgradeBackupPreserveFailed { error } =>
            format!("注意：无法将先前版本保留为备份（{error}）。在下次升级之前无法回滚。").into(),
        Msg::UpgradeBackupRemoveFailed { backup, rolling } =>
            format!("注意：无法移除旧备份 {backup}，回滚可能指向更旧的版本。\n  {rolling} 处的 .rolling 文件将在下次升级时清理。").into(),
        Msg::UpgradeNoRelease { os, arch } =>
            format!("当前平台没有已发布的 rustcode 版本（{os}/{arch}）").into(),
        Msg::UpgradeNoTarget { target } =>
            format!("发布清单中没有目标 {target} 的条目 —— 此版本可能不包含该平台").into(),
        Msg::UpgradeManifestHttp { status } =>
            format!("获取 latest.json 时返回 HTTP {status}").into(),
        Msg::UpgradeDownloadHttp { url, status } =>
            format!("下载 {url} 时返回 HTTP {status} —— 该平台可能没有对应的发布版本").into(),
        Msg::UpgradeShortDownload { got, expected } =>
            format!("下载不完整：实际 {got} 字节，预期 {expected} 字节").into(),
        Msg::UpgradeChecksumMismatch { expected, got } =>
            format!("校验和不匹配 —— 文件可能已损坏或被篡改。\n  预期: {expected}\n  实际: {got}").into(),
        Msg::UpgradeExeNoParent { exe } =>
            format!("可执行文件没有父目录：{exe}").into(),
        Msg::UpgradeDirNotWritable { dir, error } =>
            format!(
                "当前用户无权写入 {dir}（{error}）。\n\
                 可使用提升的权限重新运行：sudo rustcode upgrade\n\
                 或重新安装到用户可写的位置（例如 ~/.local/bin）。"
            )
            .into(),
        Msg::CliUpgradeAvailable { version } =>
            format!("[*] 发现新版本：{version}").into(),
        Msg::CliUpgradeDownloading { pct, mb, total_mb } =>
            format!("\r   下载中 {pct}% ({mb} / {total_mb} MB)      ").into(),
        Msg::CliUpgradeVerifying => "\n[+] 正在校验 sha256".into(),
        Msg::CliUpgradeApplying { version } =>
            format!("[+] 正在升级到 {version}...").into(),
        Msg::CliUpgradeReexecFailed { error } =>
            format!("升级已应用但重新执行失败（{error}）。新版本将在下次启动时生效。").into(),
        Msg::CliUpgradeCheckFailed =>
            "提示：启动时未能检查更新（将在后台重试）。".into(),
        Msg::CliUpgradeApplyFailed { error } =>
            format!("提示：待应用的升级未能执行（{error}）。继续使用当前版本。").into(),
        Msg::CliUpgradeDevDisabled => "[dev] 已禁用自动更新".into(),
        Msg::CliFatalError { error } => format!("\nRustCode 错误：{error}").into(),

        Msg::CliDaemonStarting { port } =>
            format!("正在端口 {port} 上启动 RustCode 守护进程...").into(),
        Msg::CliDaemonStopHint => "按 Ctrl+C 停止。".into(),
        Msg::CliDaemonFatal { error } => format!("致命错误：守护进程服务器错误：{error}").into(),

        Msg::CliLoggedOut => "  已退出登录。".into(),
        Msg::CliStatusLoggedIn { username, id } =>
            format!("\n  已登录：{username}（{id}）").into(),
        Msg::CliStatusName { name } => format!("  姓名：{name}").into(),
        Msg::CliStatusEmail { email } => format!("  邮箱：{email}").into(),



        Msg::CliStatusHintNeutral =>
            "\n  [*] 本构建不含托管登录 -- 使用自带密钥（BYO）的第三方服务商。\n\
请在 ~/.rustcode/config.toml 中配置第三方服务商，填入你自己的\n\
base_url 和 api_key，或使用 --provider <name> 运行 rustcode。\n"
                .into(),


        Msg::CliConfigSaveFailed { path, error } =>
            format!("  [!] 保存配置到 {path} 失败：{error}").into(),
        Msg::CliUpgradeLatest { version } => format!("==> 最新版本：{version}").into(),
        Msg::CliUpgradeDownloadProgress { pct, bytes, total } =>
            format!("\r    下载中 {pct}%（{bytes} / {total} 字节）   ").into(),
        Msg::CliUpgradeVerifyingSha => "\n==> 正在校验 SHA256".into(),
        Msg::CliUpgradeReplacingBinary => "==> 正在替换二进制文件".into(),
        Msg::CliUpgradeCmdDone { version, backup } =>
            format!("\n[+] 已升级到 {version}（旧版本保留在 {backup}）").into(),
        Msg::CliUpgradeStartNewHint => "  运行 `rustcode` 启动新版本。".into(),
        Msg::CliUpgradeCmdFailed { error } => format!("\n升级失败：{error}").into(),
        Msg::CliUpgradePanicked { error } => format!("升级任务异常中断：{error}").into(),
        Msg::CliRollbackCmdDone { exe, backup } =>
            format!("\n[+] 已回退。exe={exe}，备份={backup}").into(),
        Msg::CliRollbackCmdDoneTwo { current, saved } =>
            format!("[+] 已回退。当前二进制位于 {current}，另一版本保存在 {saved}").into(),
        Msg::CliRollbackStartHint => "  运行 `rustcode` 启动回退后的版本。".into(),
        Msg::CliPluginSpecEmpty =>
            "应为 <plugin> 或 <plugin>@<marketplace> 形式，但得到空字符串".into(),
        Msg::CliPluginSpecPartEmpty { spec } =>
            format!("`{spec}` 中的插件名或市场名不能为空").into(),

        // ── Headless（`-p`/`--print`）stderr 输出 ──
        Msg::CliHeadlessProviderRetry { reason, backoff_secs, attempt, max_attempts } =>
            format!("API 错误 {reason}，{backoff_secs} 秒后重试（{attempt}/{max_attempts}）...").into(),
        Msg::CliHeadlessRateAutoResume { secs } =>
            format!("将在 {secs} 秒后自动继续...").into(),
        Msg::CliHeadlessRateRetry { reason, secs } =>
            format!("HTTP 429{reason} —— 请稍后重试（{secs} 秒后）").into(),
        Msg::CliHeadlessRatePaused { reason } =>
            format!("HTTP 429{reason} —— 已暂停，请稍后重试").into(),
        Msg::CliHeadlessRateWindowResetAt { reset_at } =>
            format!("限流窗口已耗尽 —— 预计 {reset_at} 左右重置").into(),
        Msg::CliHeadlessRateWindowSecs { secs } =>
            format!("限流窗口已耗尽 —— {secs} 秒后重置，请稍后重试").into(),
        Msg::CliHeadlessRateWindowPaused =>
            "限流窗口已耗尽 —— 已暂停，请稍后重试".into(),
        Msg::CliHeadlessAutoApproved { tool } => format!("已自动批准 {tool}").into(),
        Msg::CliHeadlessDenied { tool } =>
            format!("{tool} 需要交互式批准，已拒绝").into(),
        Msg::CliSetupCwdError { error } =>
            format!("setup 错误：无法读取当前目录：{error}").into(),
        Msg::CliSetupFailed { error } => format!("setup 错误：{error}").into(),
        Msg::CliSeedInitialized { path, source } =>
            format!("已从 {source} 初始化 {path}").into(),
        Msg::CliSeedInvalid { error } =>
            format!("警告：--seed-config 已忽略（不是有效的配置文件）：{error}").into(),
        Msg::CliSeedIoError { error } =>
            format!("警告：--seed-config 无法应用：{error}").into(),
        Msg::CliConfigLoadWarnings { path, warnings } =>
            format!("警告：{path} 中的部分 provider 配置段无法加载：\n{warnings}").into(),
        Msg::CliConfigLoadFailed { path, error } =>
            format!("警告：加载 {path} 失败（{error}）；改用默认配置。").into(),
        Msg::CliResumeHint { cmd } => format!("继续此会话，运行：{cmd}").into(),
        // 同 en：不使用 `\` 续行（Rust 会吞掉下一行前导空格，破坏缩进）。
        Msg::CliWindowsCodePage { input, output } =>
            format!(
                "\n[!]  控制台代码页 —— 输入：{input}（应为 65001/UTF-8），输出：{output}。\n                       中文/日文/韩文输入法的输入/输出可能显示为乱码。\n                       -> 请使用 Windows Terminal 以获得原生 UTF-8 支持。\n                       -> 或在区域设置中启用“Beta：使用 Unicode UTF-8 提供全球语言支持”。\n"
            ).into(),
        Msg::CliPromptFileReadFailed { path, error } =>
            format!("错误：读取 --prompt-file {path} 失败：{error}").into(),
        Msg::CliResumeNoMatch { selector } =>
            format!(
                "本项目中没有匹配 id 或名称 {selector} 的会话 —— 运行 `rustcode resume` 列出会话，或检查工作目录（-C）"
            ).into(),
        Msg::CliHeadlessNoProviderNamed { name, path } =>
            format!(
                "未找到 Provider：'{name}' 不匹配任何已配置的 provider，且未设置默认 provider。请在 {path} 中配置第三方 provider（base_url、api_key、model），或不带参数运行 `rustcode` 进入交互式设置。"
            ).into(),
        Msg::CliHeadlessNoProvider { path } =>
            format!(
                "未配置 provider。请在 {path} 中添加第三方 provider（base_url、api_key、model），或不带参数运行 `rustcode` 进入交互式设置。"
            ).into(),

        // ── `rustcode mcp` ──
        Msg::CliMcpAdded { name, path, program, args } =>
            format!("  已添加 MCP 服务器 {name} -> {path}（stdio：{program} + {args} 个参数）").into(),
        Msg::CliMcpAddedOauth { name, path, url } =>
            format!("  已添加 OAuth MCP 服务器 {name} -> {path}（{url}）").into(),
        Msg::CliMcpAddedGithub { name, path } =>
            format!("  已添加 GitHub OAuth MCP 服务器 {name} -> {path}").into(),
        Msg::CliMcpLoginSaved { provider, name, scopes } =>
            format!("  已为 MCP 服务器 {name} 保存 {provider} OAuth 令牌（{scopes} 个 scope）").into(),
        Msg::CliMcpLogoutRemoved { name } =>
            format!("  已移除 MCP 服务器 {name} 保存的 OAuth 令牌").into(),
        Msg::CliMcpLogoutNotFound { name } =>
            format!("  未找到 MCP 服务器 {name} 已保存的 OAuth 令牌").into(),
        Msg::CliMcpServerNotFound { name } =>
            format!("配置中未找到 MCP 服务器 {name}").into(),

        // ── `rustcode hooks` ──
        Msg::CliHooksLoadedHeader => "\n已加载的 Hooks：".into(),
        Msg::CliHooksNone => "  （未加载任何 hook）".into(),
        Msg::CliHooksTableEvent => "事件".into(),
        Msg::CliHooksTableCount => "数量".into(),
        Msg::CliHooksTableTotal => "总计".into(),
        Msg::CliHooksConfigFiles => "\nHook 配置文件：".into(),
        Msg::CliHooksPathGlobal { path } => format!("全局：   {path}").into(),
        Msg::CliHooksPathProject { path } => format!("项目：  {path}").into(),
        Msg::CliHooksPathNoHome => "全局：   （无 home 目录）".into(),
        Msg::CliHooksUntrustedHeader => "未受信任的插件 hook（不会加载）：".into(),
        Msg::CliHooksUntrustedRow { plugin, count, events } =>
            format!("  {plugin} —— {count} 个 hook [{events}]。运行：rustcode plugin trust {plugin}").into(),
        Msg::CliHooksTestNotFound { name } =>
            format!("[x] 未找到匹配 '{name}' 的 hook。").into(),
        Msg::CliHooksTestNoneLoaded =>
            "\n  （未加载任何 hook，请检查 hooks.json / .hooks.json。）".into(),
        Msg::CliHooksTestAvailable =>
            "\n可用的 hook（按事件名或命令子串测试）：".into(),
        Msg::CliHooksTesting { event } => format!("\n[*] 正在测试 Hook（{event}）").into(),
        Msg::CliHooksFieldCommand { command } => format!("  命令：    {command}").into(),
        Msg::CliHooksFieldTimeout { ms } => format!("  超时：    {ms} ms").into(),
        Msg::CliHooksFieldMatcher { matcher } => format!("  匹配器：  {matcher}").into(),
        Msg::CliHooksResultHeader => "[+] 结果：".into(),
        Msg::CliHooksDuration { duration } => format!("  耗时：    {duration}").into(),
        Msg::CliHooksFieldStatus { label, detail } =>
            format!("  状态：    {label}（{detail}）").into(),
        Msg::CliHooksStatusSuccess => "退出码 0".into(),
        Msg::CliHooksStatusBlock => "退出码 2 —— hook 请求阻止（CC 协议约定）".into(),
        Msg::CliHooksStatusExitCode { code } => format!("退出码 {code}").into(),
        Msg::CliHooksStatusSignal => "被信号终止".into(),
        Msg::CliHooksDidNotComplete { ms } =>
            format!("  [x] Hook 未完成：超时（>{ms} ms）或启动失败。").into(),
        Msg::CliHooksPathsHeader => "\nHook 配置文件：".into(),
        Msg::CliHooksDocsHeader => "\n文档：".into(),
        Msg::CliHooksDocsEntry => "  docs/hooks.md - Hook 使用指南".into(),

        // ── `rustcode plugin` / `marketplace` ──
        Msg::CliPluginMpAdded { name, commit, plugins } =>
            format!("  已添加 marketplace `{name}`（{commit}，{plugins} 个插件）").into(),
        Msg::CliPluginMpRemoved { name } =>
            format!("  已移除 marketplace `{name}`").into(),
        Msg::CliPluginMpUpdated { name, commit } =>
            format!("  marketplace `{name}` 已更新到 {commit}").into(),
        Msg::CliPluginMpNone => "  未注册任何 marketplace".into(),
        Msg::CliPluginMpRow { name, source, commit, plugins } =>
            format!("  {name}  {source}  {commit}（{plugins} 个插件）").into(),
        Msg::CliPluginInstalled { plugin, marketplace } =>
            format!("  已安装 `{plugin}@{marketplace}`").into(),
        Msg::CliPluginInstallAmbiguous { plugin, list } =>
            format!("插件 `{plugin}` 存在于多个 marketplace，请指定：\n{list}").into(),
        Msg::CliPluginNotFound { plugin } =>
            format!("在任何 marketplace 中都找不到插件 `{plugin}`").into(),
        Msg::CliPluginUntrustedNotice { plugin, count, events } =>
            format!(
                "插件 `{plugin}` 附带 {count} 个 hook（[{events}]）。在受信任之前它们不会运行：\n  rustcode plugin trust {plugin}"
            ).into(),
        Msg::CliPluginUninstalled { plugin, marketplace } =>
            format!("  已卸载 `{plugin}@{marketplace}`").into(),
        Msg::CliPluginNotInstalled { plugin } =>
            format!("插件 `{plugin}` 未安装").into(),
        Msg::CliPluginUninstallAmbiguous { plugin, list } =>
            format!(
                "插件 `{plugin}` 从多个 marketplace 安装，请指定：\n{list}请使用 /plugin 选择要移除的安装范围。"
            ).into(),
        Msg::CliPluginNoHooks { name } =>
            format!("插件 `{name}` 没有 hook（或未安装）").into(),
        Msg::CliPluginTrusted { count, name, events } =>
            format!("已信任来自 `{name}` 的 {count} 个 hook（[{events}]）。").into(),
        Msg::CliPluginTrustAmbiguous { name, list } =>
            format!(
                "插件 `{name}` 的 hook 存在于多个安装中：\n{list}请使用 /plugin 查看安装范围。"
            ).into(),
        Msg::CliPluginUntrusted { name } =>
            format!("已取消信任来自 `{name}` 的 hook。").into(),
        Msg::CliPluginNone => "  没有已安装的插件".into(),
        Msg::CliPluginErrAddMp => "添加 marketplace".into(),
        Msg::CliPluginErrRemoveMp => "移除 marketplace".into(),
        Msg::CliPluginErrUpdateMp => "更新 marketplace".into(),
        Msg::CliPluginErrInstall => "安装".into(),
        Msg::CliPluginErrResolve => "解析".into(),
        Msg::CliPluginErrUninstall => "卸载".into(),

        // ── `rustcode schedule` ──
        Msg::CliSchedDailyBad { value } =>
            format!("--daily 需要 HH:MM 格式，实际为 {value}").into(),
        Msg::CliSchedWeeklyBad { value } =>
            format!("--weekly 需要 N@HH:MM 格式，实际为 {value}").into(),
        Msg::CliSchedWeekdayBad { value } =>
            format!("--weekly 星期必须为 1..7，实际为 {value}").into(),
        Msg::CliSchedWeeklyTimeBad { value } =>
            format!("--weekly 时间必须为 HH:MM，实际为 {value}").into(),
        Msg::CliSchedEveryBad { value } =>
            format!("--every 需要类似 '30m' 的格式，实际为 {value}").into(),
        Msg::CliSchedEveryIntBad { value } =>
            format!("--every 分钟数必须为正整数，实际为 {value}").into(),
        Msg::CliSchedEveryZero => "--every 分钟数必须大于 0".into(),
        Msg::CliSchedFrequencyRequired =>
            "必须指定一个频率参数：--daily HH:MM | --weekly N@HH:MM | --every Nm | --hourly | --cron EXPR".into(),
        Msg::CliSchedRemoveFailed { id } => format!("移除任务 {id} 失败").into(),
        Msg::CliSchedTaskNotFound { id } => format!("未找到任务 {id}").into(),
        Msg::CliSchedSaveFailed { id } => format!("保存任务 {id} 失败").into(),
        Msg::CliSchedRegFailed { id, error } =>
            format!("[schedule] 警告：任务 {id} 注册到系统调度器失败：{error}\n可运行 `rustcode schedule sync` 重试。").into(),
        Msg::CliSchedSyncInstalled { id } => format!("  sync：已安装 {id}").into(),
        Msg::CliSchedSyncInstallFailed { id, error } =>
            format!("  sync：安装 {id} 失败：{error}").into(),
        Msg::CliSchedSyncUninstalled { id } => format!("  sync：已卸载 {id}").into(),
        Msg::CliSchedSyncUninstallFailed { id, error } =>
            format!("  sync：卸载 {id} 失败：{error}").into(),
        Msg::CliSchedSyncDone { installed, uninstalled, errors } =>
            format!("  sync 完成：已安装 {installed} 个，已卸载 {uninstalled} 个，{errors} 个错误").into(),
        Msg::CliSchedAdded { id, title } => format!("  已添加任务 {id}（{title}）").into(),
        Msg::CliSchedNone =>
            "  暂无计划任务。使用 `rustcode schedule add` 创建一个。".into(),
        Msg::CliSchedRemoved { id } => format!("  已移除任务 {id}").into(),
        Msg::CliSchedEnabled { id } => format!("  已启用任务 {id}").into(),
        Msg::CliSchedDisabled { id } => format!("  已禁用任务 {id}").into(),
        Msg::CliSchedRunSkipped { id } =>
            format!("  schedule run：任务 {id} 已禁用，跳过").into(),
        Msg::CliSchedBadCwd { cwd, id } =>
            format!("[schedule] 任务 {id} 的工作目录 {cwd} 不存在").into(),
        Msg::CliSchedListRow { id, title, next, last, state, reg } =>
            format!("  {id} | {title} | 下次：{next} | 上次：{last} | {state} | {reg}").into(),
        Msg::CliSchedStateOn => "开".into(),
        Msg::CliSchedStateOff => "关".into(),
        Msg::CliSchedRegRegistered => "已注册".into(),
        Msg::CliSchedRegMissing => "缺失".into(),
        Msg::CliSchedRegUnknown => "未知".into(),

        // ── `rustcode uninstall` ──
        Msg::CliUninstallPurgeConflict =>
            "rustcode uninstall：--purge 与 --keep-data 冲突".into(),
        Msg::CliUninstallNoTty =>
            "rustcode uninstall：没有 TTY，拒绝交互式运行。\n请指定以下参数之一：--yes（使用默认项）、--purge（删除全部）、--keep-data（仅二进制）、--dry-run。".into(),
        Msg::CliUninstallBinaryRequired =>
            "rustcode uninstall：不删除二进制就无法卸载，已中止。".into(),
        Msg::CliUninstallProcsAborted => "已中止：仍在运行的进程未被终止。".into(),
        Msg::CliUninstallProcsFound { count } =>
            format!("\n发现 {count} 个正在运行的 rustcode 进程：").into(),
        Msg::CliUninstallKillPrompt => "终止这些进程并继续？[y/N]：".into(),
        Msg::CliUninstallKillFailed { pid, error } =>
            format!("无法终止进程 pid {pid}：{error}").into(),
        Msg::CliUninstallKillWarn { pid, error } =>
            format!("警告：无法终止进程 pid {pid}：{error}（继续 —— Unix 下 unlink 不需要终止进程）").into(),
        Msg::CliUninstallDryRun => "试运行（DRY RUN）—— 不会做任何更改。\n".into(),
        Msg::CliUninstallGroup1Plan => "[组 1] 二进制文件 + PATH 修改".into(),
        Msg::CliUninstallGroup2Plan => "[组 2] 凭据与全局配置".into(),
        Msg::CliUninstallGroup3Plan => "[组 3] 本地状态与扩展".into(),
        Msg::CliUninstallGroup1Prompt => "[组 1] 删除二进制文件并撤销 PATH 修改？".into(),
        Msg::CliUninstallGroup2Prompt => "[组 2] 删除凭据与全局配置？".into(),
        Msg::CliUninstallGroup3Prompt => "[组 3] 删除本地状态与扩展？".into(),
        Msg::CliUninstallTagWillRemove => "将删除".into(),
        Msg::CliUninstallTagKeep => "保留".into(),
        Msg::CliUninstallIntro => "这将从系统中卸载 RustCode。\n".into(),
        Msg::CliUninstallGroup1Declined =>
            "组 1 被拒绝；已中止（删除数据时不能保留二进制）。".into(),
        Msg::CliUninstallSummaryHeader => "\n汇总：".into(),
        Msg::CliUninstallContinuePrompt => "\n是否继续？[y/N]：".into(),
        Msg::CliUninstallProceedPrompt { suffix } => format!("是否继续？{suffix}：").into(),
        Msg::CliUninstallActionRemove => "删除".into(),
        Msg::CliUninstallActionKeep => "保留".into(),
        Msg::CliUninstallLabelBinary => "二进制 + PATH".into(),
        Msg::CliUninstallLabelCredentials => "凭据".into(),
        Msg::CliUninstallLabelState => "本地状态".into(),
        Msg::CliUninstallSummaryRow { action, count, label } =>
            format!("  {action}：{count} 项（{label}）").into(),
        Msg::CliUninstallResultRemoved => "已删除：".into(),
        Msg::CliUninstallResultKept => "已保留（以后可用 --purge 删除）：".into(),
        Msg::CliUninstallResultFailed => "失败：".into(),
        Msg::CliUninstallResultBackups => "备份：".into(),
        Msg::CliWebuiNotBuilt =>
            "本二进制未内嵌 webui 资源。\n请先构建前端，再重新构建：\n\n   ./scripts/build-webui.sh\n   cargo clean -p rustcode-daemon\n   cargo build -p rustcode\n\n或手工执行等价步骤：\n   cd webui && npm ci && npm run build\n   cargo clean -p rustcode-daemon\n   cargo build -p rustcode\n".into(),

        // ── /config ──
        Msg::ConfigProviderLabel { provider, path } =>
            format!("  Provider：{}\n  配置文件：{}\n\n", provider, path).into(),

        // ── /cost ──
        Msg::CostTokenReport { prompt, completion, cached, cache_rate, total } =>
            format!(
                "  提示 Token：       {}\n  补全 Token：       {}\n  缓存 Token：       {}（{}% 命中率）\n  Token 总计：       {}\n",
                prompt, completion, cached, cache_rate, total
            ).into(),
        Msg::CostUnattributed { tokens } =>
            format!("历史未归属用量\n  Token 总计：       {}", tokens).into(),

        // ── /think ──
        Msg::ThinkStatus { enabled, budget, provider } =>
            format!(
                "  深度思考：{}\n  预算：{} Token\n  Provider：{}\n\n  用法：/think on | off | budget <N>\n",
                if enabled { "已启用" } else { "已禁用" },
                budget, provider
            ).into(),
        Msg::ThinkEnabled { budget } =>
            format!("  深度思考已启用（预算：{} Token）。\n", budget).into(),
        Msg::ThinkDisabled =>
            "  深度思考已禁用。\n".into(),
        Msg::ThinkBudgetSet { n } =>
            format!("  思考预算已设为 {} Token。\n", n).into(),
        Msg::ThinkBudgetTooSmall { n } =>
            format!("预算必须 >= 1024（当前 {}）", n).into(),
        Msg::ThinkBudgetUsage =>
            "用法：/think budget <数字>".into(),
        Msg::ThinkUsage =>
            "  用法：/think [on | off | budget <N>]\n".into(),

        // ── /remember, /forget ──
        Msg::RememberUsage =>
            "用法：/remember <要记住的内容>（--global 为全局范围）".into(),
        Msg::ForgetUsage =>
            "用法：/forget <关键词>".into(),
        Msg::MemoryScopeGlobal => "全局".into(),
        Msg::MemoryScopeProject => "项目".into(),
        Msg::Remembered { scope, content } =>
            format!("已记住（{scope}）：{content}").into(),
        Msg::AlreadyRemembered { scope, content } =>
            format!("此前已记住（{scope}）：{content}").into(),
        Msg::RememberFailed { error } =>
            format!("记忆写入失败：{error}").into(),
        Msg::ForgetNoMatch { keyword } =>
            format!("没有匹配的记忆条目：“{keyword}”。").into(),
        Msg::ForgotOne => "已忘记 1 条记忆。".into(),
        Msg::ForgotMany { count } => format!("已忘记 {count} 条记忆。").into(),
        Msg::MemoryEmpty => "（记忆为空）".into(),

        // ── /team ──
        Msg::TeamNoRuns => "暂无团队运行。".into(),
        Msg::TeamSummary { runs, completed, running, failed, stopped } => format!(
            "团队：{runs} 个运行 · 已完成 {completed} · 运行中 {running} · 失败 {failed} · 已停止 {stopped}"
        ).into(),
        Msg::TeamNoticeDispatched { run_id } =>
            format!("  ○ 团队已派发 · {run_id}\n").into(),
        Msg::TeamNoticeStopped { run_id } =>
            format!("  ○ 团队已停止 · {run_id}\n").into(),
        Msg::TeamNoticeResultsHeader { run_id } =>
            format!("  团队结果 · {run_id}").into(),
        Msg::TeamMemberFallbackId => "子代理".into(),
        Msg::TeamStatusUnknown => "未知".into(),
        Msg::TeamResultNone => "无报告".into(),
        Msg::TeamSuffixDispatched { run_id } =>
            format!("已派发 · {run_id}").into(),
        Msg::TeamSuffixStopped { run_id } =>
            format!("已停止 · {run_id}").into(),
        Msg::TeamSuffixUpdated => "已更新".into(),

        // ── 通用开关词 ──
        Msg::WordOn => "开".into(),
        Msg::WordOff => "关".into(),

        // ── /schedule list ──
        Msg::ScheduleListEmpty =>
            "  暂无定时任务。使用 `rustcode schedule add` 创建一个。\n".into(),
        Msg::ScheduleListHeader => "  定时任务：\n\n".into(),
        Msg::ScheduleRow { id, title, next, last, state } => format!(
            "  {id} | {title} | 下次：{next} | 上次：{last} | {state}\n"
        ).into(),

        // ── /background ──
        Msg::BackgroundUsage =>
            "  用法：/background <任务描述>\n".into(),

        // ── /init ──
        Msg::InitKickoff =>
            "  正在分析项目并生成 AGENTS.md...\n".into(),

        // ── /cd ──
        Msg::CdWorkingDir { cwd } =>
            format!("  工作目录：{}\n  无最近项目。使用 `/cd <路径>` 切换。\n", cwd).into(),

        // ── /diff ──
        Msg::DiffFailed { error } =>
            format!("git diff 失败：{}", error).into(),

        // ── /upgrade ──
        Msg::UpgradePackageManaged =>
            "本版本由 HarmonyBrew 管理，请运行 `brew upgrade rustcode` 升级".into(),
        Msg::UpgradeUnknownArg { arg } =>
            format!("未知的 /upgrade 参数：{}\n  用法：/upgrade [rollback|--force]", arg).into(),
        Msg::UpgradeNoEndpoint =>
            "  当前构建不支持自升级：未配置更新清单端点。\n  此开源构建请通过包管理器或发布压缩包安装更新。\n".into(),

        // ── /skills ──
        Msg::SkillsNone =>
            "  没有可调用的技能。\n".into(),
        Msg::SkillsAvailable =>
            "  可用技能：\n".into(),
        Msg::CmdSkillsEmptyHint =>
            "尚未安装可供调用的技能。\n    \u{2022} 将 SKILL.md 放入 ~/.rustcode/skills/<name>/ \n      （Windows：%USERPROFILE%\\.rustcode\\skills\\<name>\\）\n    \u{2022} 或通过 /plugin install <git-url> 安装自带技能的插件\n\n".into(),
        Msg::SkillUnknown { name } =>
            format!("未知技能：{}（输入 /skills 查看列表）", name).into(),
        Msg::SkillsLoaded { names } =>
            format!("  已加载 skills：{}\n", names).into(),

        // ── /mcp ──
        Msg::McpReloading { count } =>
            format!("  正在重载 MCP 服务器...（{} 个已配置）\n", count).into(),
        Msg::McpConnecting =>
            "  正在连接：\n".into(),
        Msg::McpConnectingServer { name } =>
            format!("    - {}  连接中...\n", name).into(),
        Msg::McpNoServersConfigured =>
            "  未配置 MCP 服务器。\n".into(),
        Msg::McpClearedReconnecting =>
            "  已请求重载 MCP；旧 MCP 工具会先撤下，再在后台重新连接。\n".into(),
        Msg::McpClearedNoServers =>
            "  已请求重载 MCP；旧 MCP 工具会先撤下，当前没有已配置的服务器。\n".into(),
        Msg::McpToolsUsage =>
            "  用法：/mcp tools <服务器名>\n  示例：/mcp tools filesystem\n".into(),
        Msg::McpServersHeader =>
            "  MCP 服务器：\n".into(),
        Msg::McpUnknownServer { name, available } =>
            format!("  未找到名为 '{name}' 的 MCP 服务器 —— 可用：{available}\n").into(),
        Msg::McpHelp =>
            "  /mcp 用法：\n    \
             /mcp                    列出已配置的 MCP 服务器及状态\n    \
             /mcp tools <服务器名>    列出某个服务器的工具\n    \
             /mcp reload             重新加载 MCP 配置\n    \
             /mcp trust              信任本项目的 MCP 服务器\n    \
             /mcp untrust            取消信任本项目的 MCP 服务器\n    \
             /mcp login <服务器名>    对远程服务器进行 OAuth 登录\n    \
             /mcp logout <服务器名>   注销 OAuth 登录\n    \
             /mcp help               显示本帮助\n".into(),
        Msg::McpBlockedTrustHint { count } =>
            format!(
                "  有 {count} 个服务器因本项目未被信任而被拦截。\n  运行 /mcp trust 可加载本项目的 MCP 服务器。\n"
            ).into(),
        Msg::McpReloadFailed { error } =>
            format!("MCP 重载失败：无法加载 .mcp.json / $RUSTCODE_HOME/mcp.json：{:#}", error).into(),
        // /mcp login / logout
        Msg::McpOAuthLoginUsage =>
            "  用法：/mcp login <服务名>\n  示例：/mcp login github\n".into(),
        Msg::McpOAuthLogoutUsage =>
            "  用法：/mcp logout <服务名>\n  示例：/mcp logout github\n".into(),
        Msg::McpOAuthLoadConfigFailed { error } =>
            format!("  MCP OAuth 登录失败：无法加载配置：{error}\n").into(),
        Msg::McpOAuthServerNotFound { server } =>
            format!("  MCP OAuth 登录失败：配置中未找到服务 '{server}'。\n").into(),
        Msg::McpOAuthStarting { server } =>
            format!("  正在浏览器中启动 '{server}' 的 MCP OAuth 流程...\n").into(),
        Msg::McpOAuthSaved { provider, server } =>
            format!("  已保存 MCP 服务 '{server}' 的 {provider} OAuth Token。正在重载 MCP 能力。\n").into(),
        Msg::McpOAuthFailed { error } =>
            format!("  MCP OAuth 失败：{error}\n").into(),
        Msg::McpOAuthTokenRemoved { server } =>
            format!("  已移除 MCP 服务 '{server}' 保存的 OAuth Token。\n").into(),
        Msg::McpOAuthNoToken { server } =>
            format!("  未找到 MCP 服务 '{server}' 保存的 OAuth Token。\n").into(),
        Msg::McpOAuthLogoutFailed { error } =>
            format!("  MCP OAuth 登出失败：{error}\n").into(),
        Msg::McpProjectTrusted =>
            "  已信任本项目 -- 正在重连 MCP。\n".into(),
        Msg::McpProjectUntrusted =>
            "  已撤销本项目信任。\n".into(),
        Msg::McpProjectNotTrusted =>
            "  本项目未被信任。\n".into(),
        Msg::LspServerStarted { name, ext } =>
            format!("[+] LSP 服务 '{name}' 已为 .{ext} 启动").into(),
        Msg::LspServerFailed { name, ext, error } =>
            format!("[x] LSP 服务 '{name}'（.{ext}）失败：{error}").into(),

        // ── /worktree ──
        Msg::WorktreeUsage =>
            "  用法：\n    /worktree create <分支> [基准]   创建工作树并切换\n    /worktree list                    列出所有工作树\n    /worktree done                    切回原始目录\n    /worktree cleanup <分支>          清理工作树\n".into(),
        Msg::WorktreeCreateUsage =>
            "  用法：/worktree create <分支> [基准]\n  示例：/worktree create fix-bug main\n".into(),
        Msg::WorktreeCreated { branch, base, path } =>
            format!("  [+] 工作树已创建\n    分支：{}（基于 {}）\n    路径：{}\n    工作目录已切换\n", branch, base, path).into(),
        Msg::WorktreeCreateFailed { error } =>
            format!("工作树创建失败：{}", error).into(),
        Msg::WorktreeNoActive =>
            "  没有活跃的工作树。\n".into(),
        Msg::WorktreeListFailed { error } =>
            format!("工作树列表失败：{}", error).into(),
        Msg::WorktreeActiveHeader =>
            "  活跃工作树：\n".into(),
        Msg::WorktreeHasChanges => "（有变更）".into(),
        Msg::WorktreeClean => "（无变更）".into(),
        Msg::WorktreeCurrent => " ← 当前".into(),
        Msg::WorktreeDoneBack { path } =>
            format!("  [+] 工作目录已切回：{}\n", path).into(),
        Msg::WorktreeDoneMergeHint { branch } =>
            format!("  提示：使用 'git merge {}' 或创建 PR 合入主分支\n", branch).into(),
        Msg::WorktreeNoSession =>
            "  没有活跃的工作树会话。先使用 /worktree create 创建一个。\n".into(),
        Msg::WorktreeCleanupUsage =>
            "  用法：/worktree cleanup <分支> [--force]\n".into(),
        Msg::WorktreeCleaned { branch } =>
            format!("  [+] 工作树 '{}' 已清理\n", branch).into(),
        Msg::WorktreeCleanedSwitched { path } =>
            format!("  工作目录已切回：{}\n", path).into(),
        Msg::WorktreeCleanupUncommitted { branch } =>
            format!("  [!] 工作树 '{}' 有未提交的变更。\n  使用 /worktree cleanup {} --force 强制清理\n", branch, branch).into(),
        Msg::WorktreeCleanupFailed { error } =>
            format!("工作树清理失败：{}", error).into(),

        // ── /help commands（自定义命令） ──
        Msg::HelpCustomCommandsHeader =>
            "  自定义命令：\n".into(),
        Msg::HelpCustomNone =>
            "    （无）\n\n".into(),
        Msg::HelpCustomCreateHint =>
            "  创建方式：~/.rustcode/commands/<名称>.md 或 .rustcode/commands/<名称>.md\n".into(),
        Msg::HelpSourceGlobal => "全局".into(),
        Msg::HelpSourceProject => "项目".into(),

        // ── /setup ──
        Msg::SetupHeader { installed, skipped, failed, duration_ms } =>
            format!("\n[+] Setup 完成 -- {} 已安装, {} 已跳过, {} 失败  . 耗时 {}ms\n\n", installed, skipped, failed, duration_ms).into(),
        Msg::SetupInstalledLabel =>
            "已安装:\n".into(),
        Msg::SetupSkippedLabel =>
            "\n跳过:\n".into(),
        Msg::SetupFailedLabel =>
            "\n失败:\n".into(),
        Msg::SetupInstalledRow { kind, slug, path } =>
            format!("  [+] {}:{} -> {}\n", kind, slug, path).into(),
        Msg::SetupSkippedRow { kind, slug, reason } =>
            format!("  - {}:{} ({:?})\n", kind, slug, reason).into(),
        Msg::SetupFailedRow { kind, slug, error } =>
            format!("  [x] {}:{} -- {}\n", kind, slug, error).into(),
        Msg::CmdSetupTip =>
            // No leading emoji -- U+1F4A1 has ambiguous terminal display
            // width and desynced the line's cell layout on some terminals.
            // CJK chars below have stable width-2 so they're fine.
            "提示：运行 \x1b[1;96m/setup\x1b[0m 可自动为该项目配置 hooks、skills 和 MCP。".into(),
        Msg::CmdSetupRunning =>
            "正在运行 rustcode setup...".into(),
        Msg::CmdSetupSkillsReloaded { count } =>
            format!("  [*] Skills 已重载 -- {} 个可用", count).into(),
        Msg::CmdSetupError { error } =>
            format!("setup 错误：{error}").into(),
        Msg::CmdSetupRunningSkill =>
            "  [*] 正在运行 setup skill -- 分析项目并生成推荐...".into(),
        Msg::CmdSetupSkillMissing =>
            "setup skill 未找到 -- 请重新运行 /setup 以重新安装".into(),

        // ── /plugin ──
        Msg::PluginUsage =>
            "用法：/plugin [marketplace add|remove|update|list | install <p>@<m> | uninstall <p>@<m> | reload | list]".into(),
        Msg::PluginMarketplaceUsage =>
            "用法：/plugin marketplace [add|remove|update|list] <参数>".into(),
        Msg::PluginInstallUsage =>
            "用法：/plugin install <插件名> 或 <插件>@<市场>".into(),
        Msg::PluginInstallNotFound { plugin } =>
            format!("未在任何市场中找到插件 `{plugin}`。使用 /plugin marketplace list 查看已注册的市场。").into(),
        Msg::PluginInstallAmbiguous { plugin } =>
            format!("插件 `{plugin}` 存在于多个市场中，请指定：").into(),
        Msg::PluginUninstallUsage =>
            "用法：/plugin uninstall <插件名> 或 <插件>@<市场>".into(),
        Msg::PluginUninstallNotFound { plugin } =>
            format!("插件 `{plugin}` 未安装。使用 /plugin list 查看已安装插件。").into(),
        Msg::PluginUninstallAmbiguous { plugin } =>
            format!("插件 `{plugin}` 从多个市场安装，请指定卸载哪一个：\n").into(),
        Msg::PluginNoMarketplaces =>
            "未注册任何市场".into(),
        Msg::PluginMarketplacesHeader =>
            "已注册的市场：".into(),
        Msg::PluginNoInstalled =>
            "未安装任何插件".into(),
        Msg::PluginInstalledHeader =>
            "已安装的插件：".into(),
        Msg::PluginMarketplaceCloning { url } =>
            format!("正在从 {url} 克隆 marketplace...").into(),
        Msg::PluginMarketplaceRemoved { name } =>
            format!("已移除 marketplace `{name}`").into(),
        Msg::PluginMarketplaceRemoveFailed { error } =>
            format!("移除 marketplace 失败：{error}").into(),
        Msg::PluginMarketplaceUpdating { name } =>
            format!("正在更新 marketplace `{name}`...").into(),
        Msg::PluginMarketplaceListFailed { error } =>
            format!("列出 marketplace 失败：{error}").into(),
        Msg::PluginAutoUpdateSkipped { detail } =>
            format!("插件市场同步已跳过（不影响对话）：{detail}").into(),
        Msg::OfflineModeActive =>
            "离线模式：已停用联网工具、遥测与自动更新。".into(),
        Msg::PluginHooksUntrusted { count, names } => format!(
            "{count} 个插件带未信任的 hook（{names}）---- 不会运行。运行 rustcode plugin trust <name> 授权。"
        ).into(),
        Msg::PluginInstalling { plugin, marketplace } =>
            format!("正在安装 `{plugin}@{marketplace}`...").into(),
        Msg::PluginInstallingByName { plugin } =>
            format!("正在安装 `{plugin}`...").into(),
        Msg::PluginAlreadyInstalled { id } =>
            format!("  插件 `{id}` 已安装。\n  PS: 如需重新安装，请先执行 `/plugin uninstall {id}`，然后再执行 `/plugin install {id}`\n").into(),
        Msg::PluginMgrBrowse => "浏览并安装".into(),
        Msg::PluginMgrAdd => "添加市场...".into(),
        Msg::PluginMgrRemove => "移除市场...".into(),
        Msg::PluginMgrInstalled { count } => format!("已安装 ({count})").into(),
        Msg::PluginMgrInstalledMark => "[+] 已安装".into(),
        Msg::PluginMgrInstalledStatus => "已安装".into(),
        Msg::PluginMgrInstallableStatus => "可以安装".into(),
        Msg::PluginMgrInstallingStatus => "安装中".into(),
        Msg::PluginMgrUpdatingStatus => "更新中".into(),
        Msg::PluginMgrHintNav => "↑/↓ 选择 . ⏎ 进入 . esc 返回".into(),
        Msg::PluginMgrHintToggle => "⏎ 安装/卸载 . esc 返回".into(),
        Msg::PluginMgrHintRemove => "⏎ 移除 . esc 返回".into(),
        Msg::PluginMgrHintUninstall => "⏎ 卸载 . esc 返回".into(),
        Msg::PluginMgrHintUrl => "⏎ 确认添加 . esc 取消".into(),
Msg::PluginMgrHintPending => "安装中，请稍候... . esc 返回".into(),
Msg::PluginMgrHintUpdating => "更新中，请稍候... . esc 返回".into(),
Msg::PluginMgrInstallingLabel => "安装中...".into(),
        Msg::PluginMgrEmptyMarketplaces => "暂无市场，请选「添加市场...」 . esc 返回".into(),
        Msg::PluginMgrEmptyPlugins => "该市场暂无插件 . esc 返回".into(),
        Msg::PluginMgrEmptyInstalled => "暂无已安装插件 . esc 返回".into(),
        Msg::PluginMgrCloning => "正在克隆市场...".into(),
        Msg::PluginMgrInstalling { plugin } => format!("正在安装 {plugin}...").into(),
        Msg::PluginMgrUpdating { plugin } => format!("正在更新 {plugin}...").into(),
 Msg::PluginMgrEscToCancel => "Esc 取消".into(),
        Msg::PluginMgrRemoveMarketplaceTitle => "  * 移除市场".into(),
        Msg::PluginMgrRemoveMarketplacePrompt { name } => format!("  \x1b[33m您确定要移除插件市场 '{name}' 吗？\x1b[39m").into(),
        Msg::PluginMgrRemoveMarketplaceYes => "是，移除".into(),
        Msg::PluginMgrRemoveMarketplaceNo => "否，保留".into(),
        Msg::PluginMgrRemoveMarketplaceHint => "↑/↓ 选择 . Enter 确认 . Esc 取消".into(),
 Msg::PluginScopeUser => "为你安装（用户级）".into(),
Msg::PluginScopeUserDesc => "~/.rustcode/plugins -- 所有项目可见".into(),
Msg::PluginScopeProject => "为所有协作者安装（项目级）".into(),
Msg::PluginScopeProjectDesc => ".rustcode/plugins -- 通过 git 共享".into(),
Msg::PluginScopeLocal => "仅在本仓库为你安装（本地级）".into(),
Msg::PluginScopeLocalDesc => ".rustcode/plugins/local -- 不提交到 git".into(),
Msg::PluginScopeHint => "↑↓ 选择范围 . Enter 确认 . Esc 返回".into(),
Msg::PluginScopeUserShort => "用户级".into(),
Msg::PluginScopeProjectShort => "项目级".into(),
Msg::PluginScopeLocalShort => "本地级".into(),
Msg::PluginActionUninstall => "卸载".into(),
Msg::PluginActionUninstallDesc => "卸载该插件所有的组件与配置".into(),
Msg::PluginActionUpdate => "更新".into(),
Msg::PluginActionUpdateDesc => "重新拉取并安装最新版本".into(),
Msg::PluginActionDisable => "禁用".into(),
Msg::PluginActionDisableDesc => "临时禁用该插件".into(),
Msg::PluginActionBack => "返回到上一级".into(),
Msg::PluginActionBackDesc => "返回已安装插件列表".into(),
        Msg::PluginUninstalled { plugin, marketplace } =>
            format!("已卸载 `{plugin}@{marketplace}`").into(),
        Msg::PluginUninstallFailed { error } =>
            format!("卸载失败：{error}").into(),
        Msg::PluginListFailed { error } =>
            format!("列出插件失败：{error}").into(),
        Msg::PluginReloadDone { skills, warnings } =>
            format!("插件已重新加载：{skills} 个 skill，{warnings} 个警告").into(),
        Msg::PluginGitNotFound =>
            "[!] 当前环境未安装 git 或 git 不在 PATH 中，插件市场自动安装和自动更新已禁用。请安装 git（macOS 可执行 `xcode-select --install`，Ubuntu 可执行 `sudo apt install git`）后重启 {brand}。".into(),
        Msg::PluginMarketplaceAdded { name, commit, count, plugins } =>
            format!(
                "[+] 已添加 marketplace `{name}`（commit {commit}，共 {count} 个插件）\n  \
                 插件：{plugins} ---- 运行 /plugin install <插件名>@{name} 安装后才能使用其命令"
            ).into(),
        Msg::PluginMarketplaceUpdated { name, commit } =>
            format!("[+] marketplace `{name}` 已更新至 {commit}").into(),
        Msg::PluginInstallDone { plugin, marketplace: _, loaded, skipped, show_details_hint } => {
            format!("  `  [+] 已安装 {plugin} ---- {}", plugin_reload_summary(loaded, skipped, show_details_hint)).into()
        }
        Msg::PluginUpdateDone { plugin, marketplace: _, loaded, skipped, show_details_hint } => {
            format!("  `  [+] 已更新 {plugin} ---- {}", plugin_reload_summary(loaded, skipped, show_details_hint)).into()
        }
        Msg::SetupAutoReloaded { skills, warnings } =>
            format!("[+] Setup 完成，已自动刷新：{skills} 个 skill，{warnings} 个警告").into(),

        // ── 插件管理模态框 ──
        Msg::PluginTabAll => "全部插件".into(),
        Msg::PluginTabInstalled { count } => format!("已安装（{count}）").into(),
        Msg::PluginTabMarketplaces => "插件市场".into(),
        Msg::PluginAutoUninstallFailed { name, error } =>
            format!("自动卸载插件 '{name}' 失败：{error}").into(),
        Msg::PluginNoPluginsMatch { query } =>
            format!("没有匹配的插件：'{query}'").into(),
        Msg::PluginNoInstalledMatch { query } =>
            format!("已安装插件中没有匹配项：'{query}'").into(),
        Msg::PluginAddMarketplaceRow => "添加插件市场".into(),
        Msg::PluginAddMarketplacePlus => "+ 添加插件市场".into(),
        Msg::PluginEnterSourceRow => "输入市场来源：".into(),
        Msg::PluginExamplesRow => "示例：".into(),
        Msg::PluginBrowseRow { count } => format!("浏览插件（{count}）").into(),
        Msg::PluginUpdateRow { date } =>
            format!("更新市场（上次更新 {date}）").into(),
        Msg::PluginRemoveMarketplaceRow => "移除市场".into(),
        Msg::PluginInfoHeader => "  * 插件信息".into(),
        Msg::PluginNameLabel => "  名称：       ".into(),
        Msg::PluginMarketplaceLabel => "  市场：       ".into(),
        Msg::PluginVersionLabel => "  版本：       ".into(),
        Msg::PluginScopeLabel => "  范围：       ".into(),
        Msg::PluginDescriptionLabel => "  描述：       ".into(),
        Msg::PluginSelectScopeHeader => "  选择安装范围：".into(),
        Msg::PluginManageHeader => "  管理插件：".into(),
        Msg::PluginStatusInstalling { label } =>
            format!("  状态：       {label}…").into(),
        Msg::PluginAvailableCount { count } =>
            format!("  {count} 个可用插件").into(),
        Msg::PluginModalInstalledHeader { count } =>
            format!("  \x1b[1m已安装插件（{count}）：\x1b[22m").into(),
        Msg::PluginNoInstalledFromMarketplace =>
            "尚未从该市场安装插件。".into(),
        Msg::PluginVersionUnknown => "未知".into(),
        Msg::PluginCategoryGit => "Git".into(),
        Msg::PluginCategoryLinter => "代码检查".into(),
        Msg::PluginCategoryFormatter => "格式化".into(),
        Msg::PluginCategoryLanguage => "语言".into(),
        Msg::PluginCategorySecurity => "安全".into(),
        Msg::PluginCategoryAi => "AI".into(),
        Msg::PluginCategoryUtility => "实用工具".into(),
        Msg::PluginCategoryTool => "工具".into(),
        Msg::PluginCategoryCompletion => "补全".into(),

        // ── 命令描述 ──
        Msg::CmdDescWebui => "启动浏览器 webui（子命令：stop / lan / --host <地址>）".into(),
        Msg::CmdDescTunnel => "通过中继将当前会话暴露为远程访问（frp 风格反向隧道；子命令：lan / stop）".into(),
Msg::CmdDescSetup =>
"扫描项目、安装种子文件并运行 setup skill [hooks|mcp|skills|all]".into(),
        Msg::CmdDescResume => "恢复上次会话".into(),
        Msg::CmdDescRename => "重命名当前会话".into(),

        Msg::CmdDescWhoami => "显示本地运行时与身份（provider / model / base_url / 凭据 / RUSTCODE_HOME / 会话 / turns）".into(),
        Msg::CmdDescModel => "设置默认 Provider / 模型，并切换当前会话".into(),
        Msg::CmdDescProvider => "管理 Provider（添加、编辑、删除、设为全局默认）".into(),
        Msg::CmdDescStatus => "显示会话状态".into(),
        Msg::CmdDescConfig => "显示配置文件路径".into(),
        Msg::CmdDescReload => "从磁盘重新加载 $RUSTCODE_HOME/config.toml".into(),
        Msg::CmdDescCd => "切换工作目录并开启新建对话".into(),
Msg::CmdDescInit => "分析项目并生成 AGENTS.md".into(),
Msg::CmdDescBg => "后台会话：/bg、/bg list、/bg <N>、/bg drop <N>".into(),
Msg::CmdDescBackground => "在隔离的后台上下文中运行一次性任务（只读工具子集）".into(),
        Msg::CmdDescDiff => "显示 git diff".into(),
        Msg::CmdDescClear => "清屏".into(),
        Msg::CmdDescSession => "开始新会话（清除对话）".into(),
        Msg::CmdDescCost => "显示本会话 Token 用量".into(),
        Msg::CmdDescUsageNeutral =>
            "显示令牌用量（标签：当前窗口 / 总览 / 模型）".into(),
        Msg::CmdDescContext => "显示上下文预算明细".into(),
        Msg::CmdDescCompact => "压缩对话历史".into(),
        Msg::CmdDescRemember => "保存记忆（/remember --global 为全局）".into(),
        Msg::CmdDescForget => "删除匹配的记忆".into(),
        Msg::CmdDescMemory => "显示所有已保存的记忆".into(),
        Msg::CmdDescMcp => "显示 MCP 服务器状态（子命令：reload）".into(),
        Msg::CmdDescUndo => "撤销：把对话记忆回退一轮（/undo 或 /undo N）".into(),
        Msg::CmdDescRewind => "回退：把对话恢复到更早的检查点".into(),
        Msg::CmdDescWorktree => "Git 工作树隔离（create/list/done/cleanup）".into(),
        Msg::CmdDescWorklog => "跨所有项目的每日工作复盘（/worklog [today|yesterday|月/日]）".into(),
        Msg::CmdDescOpenrouter =>
            "接入 OpenRouter 免费模型（/openrouter 走 OAuth，/openrouter <key> 直传已有密钥）".into(),
        Msg::OpenrouterConnecting => "正在连接 OpenRouter...".into(),
        Msg::OpenrouterAwaitingBrowser { auth_url } =>
            format!("浏览器未自动打开?手动访问完成授权:{auth_url}").into(),
        Msg::OpenrouterReady { count } =>
            format!("已接入 OpenRouter,新增 {count} 个免费模型。/model 可切换。").into(),
        Msg::OpenrouterFailed { reason } =>
            format!("OpenRouter 接入失败: {reason}。可重试 /openrouter,或 /openrouter <你的key> 直接接入。").into(),
        Msg::OpenrouterConfigSaveFailed { error } =>
            format!("OpenRouter 配置保存失败: {error}").into(),
        Msg::CmdDescUpgrade => "升级到最新版本（子命令：rollback）".into(),
        Msg::CmdDescPlan => "切换到 Plan 模式（只读探索）".into(),
        Msg::CmdDescBuild => "切换到 Build 模式（完整执行）".into(),
        Msg::CmdDescAuto => "切换到 Auto 模式（所有工具自动批准）".into(),
        Msg::CmdDescThink => "深度思考控制（on/off/budget N）".into(),
        Msg::CmdDescEffort => "模型推理强度控制（low / medium / high / xhigh / max / auto）".into(),
        Msg::CmdDescHelp => "显示帮助".into(),
        Msg::CmdDescKeys => "显示键盘快捷键".into(),
        Msg::CmdDescLanguage => "切换显示语言".into(),
        Msg::CmdDescQuit => "退出 {brand}".into(),
        Msg::CmdDescSkills => "浏览已加载的技能".into(),
        Msg::CmdDescPlugin => "插件市场（子命令：marketplace, install, uninstall, reload, list）".into(),
        Msg::CmdDescPaste => "从剪贴板粘贴图片（Windows 下 Ctrl+V 被终端拦截时的备用入口）".into(),
        Msg::CmdDescCopy => "复制上一条回复里的代码块，或用 /copy msg 复制整条回复（/copy、/copy N、/copy all、/copy msg）".into(),
        Msg::CopyOk { lines, chars } => format!("已复制代码块到剪贴板（{lines} 行，{chars} 字符）").into(),
        Msg::CopyOkMsg { lines, chars } => format!("已复制回复到剪贴板（{lines} 行，{chars} 字符）").into(),
        Msg::CopyNoCodeBlock => "上一条回复里没有可复制的代码块".into(),
        Msg::CopyMsgEmpty => "上一条回复为空，没有可复制的内容".into(),
        Msg::CopyBadIndex { count } => format!("没有这个代码块----上一条回复共 {count} 个（用 /copy N，范围 1..={count}）").into(),
        Msg::CopyFailed => "剪贴板不可用----复制失败".into(),
        Msg::CmdDescSave => "把当前对话导出为 markdown 文件（/save、/save [文件名]）".into(),
        Msg::SaveOk { path } => format!("对话已保存到 {path}").into(),
        Msg::SaveEmpty => "当前没有对话内容可导出".into(),
        Msg::SaveIoError { error } => format!("保存对话失败：{error}").into(),
        Msg::SaveInvalidPath { path } => format!("路径无效----目录不存在：{path}").into(),
        Msg::SaveRefuseOverwrite { path } => format!("目标已存在且非 markdown 文件，已拒绝覆盖（避免误删源码/配置）：{path}。请换个 .md 文件名或新路径。").into(),
        Msg::CodeBlockCopied => "[+] 代码块已复制到剪贴板".into(),
        Msg::CmdDescGuide => "向 rustcode-guide 提问使用方法".into(),
        Msg::CmdDescView => "在浮层窗口中查看文件内容".into(),
        Msg::CmdDescSync => "接入实时 webui 会话（/sync off 断开）".into(),
        Msg::CmdDescReview => "审查当前代码改动（/review . /review staged . /review <基准>）".into(),
        Msg::CmdDescWiki => "分析项目并生成 wiki（架构图 + 模块文档）".into(),
        Msg::CmdDescGoal => "设定完成目标（自主循环直到达成）".into(),
        Msg::CmdDescProxy => "切换出站代理模式".into(),
        Msg::CmdDescTodo => "显示当前任务清单；`/todo add <任务>` 追加一条，`/todo clear` 清空".into(),
        Msg::CmdDescTeam => "显示或控制 Team Agent 进度面板".into(),
        Msg::CmdDescSchedule => "查看定时任务列表和下次运行时间".into(),
        // ── /proxy 选择器 ──
        Msg::ProxyTitleFollowSystem => "跟随系统".into(),
        Msg::ProxyTitleDefaultProxy => "默认代理".into(),
        Msg::ProxyTitleNoProxy => "无代理".into(),
        Msg::ProxyDescFollowSystem =>
            "跟随当前启动环境 / 系统代理状态".into(),
        Msg::ProxyDescDefaultProxy =>
            "固定当前代理环境变量，后续启动继续复用".into(),
        Msg::ProxyDescNoProxy =>
            "出站 HTTP 客户端禁用代理解析".into(),
        Msg::ProxyModeLine { mode } => format!("  代理模式：{mode}\n").into(),
        Msg::ProxyDefaultPinned { count } =>
            format!("默认代理（已固定 {count} 个环境变量）").into(),
        Msg::ProxyDefaultEmpty => "默认代理（未捕获到环境变量）".into(),
        Msg::DesktopOpening { name, path } =>
            format!("正在打开 {}...\n  {}\n", name, path).into(),
        Msg::DesktopNotInstalled { url } =>
            format!("未检测到 {{brand}} 桌面端。下载安装：\n  {}\n", url).into(),
        Msg::DesktopNotInstalledNoUrl =>
            "未检测到 {brand} 桌面端，且当前构建未提供桌面端下载地址。\n  请使用终端界面，或在 config.toml 中配置你自己的供应商。\n".into(),
        Msg::DesktopLaunchFailed { path, err } =>
            format!("找到了应用但启动失败：{}\n  {}\n", err, path).into(),
        Msg::TodoNoList => "当前无任务清单（模型尚未创建 todo）。".into(),
        Msg::TodoListHeader => "当前任务清单:".into(),
        Msg::TodoAddUsage => "用法：/todo add <任务描述>".into(),
        Msg::GuideMenuHeader => "[*] {brand} 使用指南 -- 输入 /guide <问题> 提问".into(),
        Msg::GuideMenuTopics => "常用话题：".into(),
        Msg::GuideMenuGettingStarted => "怎么开始使用          首次安装、登录、配置".into(),
        Msg::GuideMenuSwitchModel => "怎么设置默认模型       /model /provider 操作".into(),
        Msg::GuideMenuMcp => "怎么用 MCP            MCP 服务器配置与管理".into(),
        Msg::GuideMenuSkills => "怎么用技能和插件       /skills /plugin 使用".into(),
        Msg::GuideMenuMemory => "怎么用记忆功能         /remember /forget /memory".into(),
        Msg::GuideMenuBackground => "怎么用后台任务         /bg 后台执行".into(),
        Msg::GuideMenuContext => "怎么管理上下文         /compact /context /cost".into(),
        Msg::GuideMenuKeybindings => "快捷键有哪些           键盘快捷键参考".into(),
        Msg::GuideMenuConfig => "怎么配置               config.toml 配置说明".into(),
        Msg::GuideMenuTip => "
  提示：输入 /guide <你的问题> 获取具体回答。
  例如：/guide 怎么设置默认模型
".into(),
        Msg::GuideMenuDocUrl => "  完整文档：https://docs.rustcode.dev/zh/".into(),
        Msg::CmdGuideInstalling => "正在安装 ask skill，请稍候...".into(),
        Msg::CmdGuideAutoInstall => "ask skill 未安装，正在自动安装 rustcode@rustcode-skills...".into(),
        Msg::CmdGuideAutoInvoke { topic } =>
            format!("ask skill 安装完成，正在回答: {}", topic).into(),
        Msg::CmdGuideSkillNotFound =>
            "安装完成但未找到 ask skill，请运行 /plugin reload 后重试".into(),
        Msg::CmdGuideInstallFailed { error } =>
            format!("安装 ask skill 失败: {}. 请手动运行 /plugin install rustcode@rustcode-skills", error).into(),
        Msg::CmdPasteNoImage => "剪贴板中没有图片。".into(),
        Msg::CmdPasteNoImageOhos => {
            "鸿蒙暂不支持读取系统剪贴板图片。请把图片存成文件，然后粘贴/输入它的绝对路径（如 /storage/.../pic.png）来添加图片。".into()
        }

        // ── reasoning effort ──
        Msg::ReasoningEffortNoEffect => "当前模型未配置 reasoning_effort 支持，请在 /provider 中启用".into(),
        Msg::EffortCleared => "  reasoning_effort 已清除（API 默认值）\n".into(),

        // ── 配置保存失败 ──
        Msg::ConfigSaveFailed { error } =>
            format!("配置保存失败：{}", error).into(),
        Msg::CfgLegacyProviderNotFound { name } =>
            format!("未找到旧版 provider `{name}`").into(),
        Msg::CfgLegacyProviderExists { name } =>
            format!("无法升级 `{name}`：新结构的账号或模型已占用该 id").into(),
        Msg::CfgResolveNoModel =>
            "未选择模型（请设置 `default_model` 或 `default_provider`）".into(),
        Msg::CfgResolveModelNotFound { id } => format!("未找到模型 `{id}`").into(),
        Msg::CfgResolveModelUnknownAccount { id, account } =>
            format!("模型 `{id}` 引用了未知账号 `{account}`").into(),
        Msg::CfgDiagAccountMissingProvider { id } =>
            format!("Provider 账号 `{id}` 缺少 `provider` 字段").into(),
        Msg::CfgDiagAccountNoEndpoint { id, provider } =>
            format!(
                "Provider 账号 `{id}` 使用的 `{provider}` 没有默认端点；请设置 `base_url`"
            )
            .into(),
        Msg::CfgDiagModelMissingModel { id } =>
            format!("模型 `{id}` 缺少 `model` 字段").into(),
        Msg::CfgDiagModelMissingAccount { id } =>
            format!("模型 `{id}` 缺少 `account` 字段").into(),
        Msg::CfgDiagModelUnknownAccount { id, account } =>
            format!("模型 `{id}` 引用了未知账号 `{account}`").into(),
        Msg::CfgDiagModelContextWindow { id } =>
            format!("模型 `{id}` 的 context_window = 0").into(),
        Msg::CfgDiagModelMaxTokens { id } => format!("模型 `{id}` 的 max_tokens = 0").into(),
        Msg::CfgDiagDefaultModelMismatch { sel } =>
            format!("default_model `{sel}` 不匹配任何模型配置").into(),
        Msg::CfgDiagAccountCollision { id } =>
            format!(
                "Provider 账号 `{id}` 与同名旧版 provider 冲突；以新结构账号为准"
            )
            .into(),
        Msg::CfgDiagModelCollision { id } =>
            format!("模型 `{id}` 与同名旧版 provider 冲突；以新结构模型为准").into(),

        // ── OnboardingWizard ──
        Msg::OnboardingStepHeaderWelcome => "第 1/3 步 . 欢迎".into(),
        Msg::OnboardingStepHeaderLanguage => "第 2/3 步 . 语言".into(),
        Msg::OnboardingStepHeaderSetup => "第 3/3 步 . 配置".into(),
        Msg::OnboardingPanelTitle => "{brand}".into(),
        Msg::OnboardingStepIndicator { current, total } =>
            format!("第 {current}/{total} 步").into(),
        Msg::OnboardingIntroVersionLine { v } =>
            format!("版本 {v}  .  在终端里运行的 AI 编程代理").into(),
        Msg::OnboardingIntroBullet1 =>
            "* 多步骤 agent loop . 内置代码图工具".into(),
        Msg::OnboardingIntroBullet2 =>
            "* 兼容所有 OpenAI 风格 API".into(),
        Msg::OnboardingIntroBullet3 =>
            "* 通过托管套餐获取免费额度".into(),
        Msg::OnboardingIntroBullet3Neutral =>
            "* 自带 API Key，无需注册账号".into(),
        Msg::OnboardingIntroPressEnter => "按 Enter 继续。".into(),
        Msg::OnboardingIntroCtrlC => "Ctrl+C 可随时退出。".into(),
        Msg::OnboardingIntroCompactTagline =>
            "在终端里运行的 AI 编程代理。".into(),
        Msg::OnboardingLanguageTitleBilingual =>
            "Choose your language / 选择语言".into(),
        Msg::OnboardingLanguagePrompt =>
            "选择界面语言。任何时候都可以用 `/language` 修改。".into(),
        Msg::OnboardingLanguageOptionAuto =>
            "自动检测 (LC_ALL / LANG)".into(),
        Msg::OnboardingLanguageOptionEn => "English".into(),
        Msg::OnboardingLanguageOptionZhCn => "简体中文 (Simplified Chinese)".into(),
        Msg::OnboardingSetupTitle => "想怎么开始？".into(),
        Msg::OnboardingNavHint =>
            "1-3 选择 . Enter 确认 . ← 返回 . Esc 跳过".into(),
        Msg::OnboardingSetupNavHint =>
            "数字键选择 . Enter 确认 . ← 返回 . Esc 跳过".into(),
        Msg::OnboardingConfirmClear =>
            "/welcome 会清屏。是否继续？[y/N]".into(),
        Msg::CmdWelcomeDescription => "重新运行 onboarding 向导".into(),
        Msg::VisionPreprocessSuccess { char_count } =>
            format!("[+] VL 识别图片成功，返回 {char_count} 个字符").into(),
        Msg::VisionPreprocessFailed { reason } =>
            format!("VL 预处理失败：{reason} . 本轮以纯文字继续，图片已恢复可重试").into(),
        Msg::TurnSummary { done, turn_count, tool_call_count, duration, total_tokens, cached_pct } =>
            format!(
                "[+] {done} . {turn_count} 轮 . {tool_call_count} 工具 . {duration} . {} tokens{}",
                super::fmt_tokens(total_tokens),
                cached_pct.map(|p| format!(" · 缓存命中 {p}%")).unwrap_or_default(),
            ).into(),
        Msg::TurnStatsFragment { tool_call_count, duration, total_tokens, cached_pct } =>
            format!(
                "{tool_call_count} 工具 . {duration} . {} tokens{}",
                super::fmt_tokens(total_tokens),
                cached_pct.map(|p| format!(" · 缓存命中 {p}%")).unwrap_or_default(),
            ).into(),
        Msg::GoalRound { round, stats } =>
            format!("\u{21bb} goal 第 {round} 轮 . {stats}").into(),
        Msg::TurnSummaryError { turn_count, tool_call_count, duration, total_tokens, reason } => {
            let cause = reason.map(|r| format!("：{r}")).unwrap_or_default();
            format!("[x] 已中断{cause} . {turn_count} 轮 . {tool_call_count} 工具 . {duration} . {} tokens", super::fmt_tokens(total_tokens)).into()
        }
        Msg::TurnSummaryPolicyDenied { turn_count, tool_call_count, duration, total_tokens, reason } => {
            let cause = reason.map(|r| format!("：{r}")).unwrap_or_default();
            format!("[x] 安全策略已终止本回合{cause} . {turn_count} 轮 . {tool_call_count} 工具 . {duration} . {} tokens", super::fmt_tokens(total_tokens)).into()
        }
        Msg::SpinnerEffortSuffix { effort } => format!(" · {effort}强度思考").into(),
        Msg::SpinnerQueuedSuffix { count } => format!(" · {count} 条排队").into(),
        Msg::SpinnerElapsedTokens { elapsed, tokens } =>
            format!(" ({elapsed} · ↑ {tokens} tokens)").into(),
        Msg::SpinnerElapsedOnly { elapsed } => format!(" ({elapsed})").into(),
        Msg::SpinnerSubAgents { done, total } => format!("子代理 {done}/{total}").into(),
        Msg::SpinnerWaitingApproval => "等待审批".into(),
        Msg::SpinnerRunningLabel => "运行中".into(),

        // ── Live hub / 手机远程同步错误 ──
        Msg::LiveSyncEventFailed { error } =>
            format!("实时事件同步失败：{error}").into(),
        Msg::LiveSyncProviderFailed { error } =>
            format!("实时 Provider 同步失败：{error}").into(),
        Msg::LiveSyncGoalFailed { error } =>
            format!("实时 Goal 同步失败：{error}").into(),
        Msg::LiveSyncRemoteOutputFailed { error } =>
            format!("远程命令输出同步失败：{error}").into(),
        Msg::LiveSyncRemoteRejectFailed { error } =>
            format!("远程命令拒绝通知同步失败：{error}").into(),
        Msg::LiveRemoteCommandEcho { display } =>
            format!("（手机端执行 {display}）").into(),
        Msg::LiveRemoteCommandRejected =>
            "  该命令需要在桌面端执行（手机端仅支持 /status /cost /whoami /diff）".into(),
        Msg::LiveProjectionNoSessionIdentity =>
            "会话切换已完成但缺少会话标识".into(),
        Msg::LiveProjectionUnexpectedIdentity =>
            "能力重载返回了意外的会话标识".into(),
        Msg::LiveCapabilitySnapshotFailed { error } =>
            format!("更新实时能力快照失败：{error}").into(),
        Msg::LiveSessionDecodeFailed { session_id, error } =>
            format!("解码会话 {session_id} 失败：{error}").into(),
        Msg::LiveSessionDisappeared { session_id } =>
            format!("会话 {session_id} 在运行时切换后消失").into(),
        Msg::LiveSessionResolveFailed { session_id, error } =>
            format!("解析会话 {session_id} 失败：{error}").into(),
        Msg::LiveSessionSnapshotFailed { error } =>
            format!("更新实时会话快照失败：{error}").into(),

        // ── 渲染：状态徽标 / agent 分组标题 ──
        Msg::BadgeSearchPrefix => " 搜索：'".into(),
        Msg::BadgeSearchCount { current, total } =>
            format!(" {current}/{total} ").into(),
        Msg::BadgeHistory { current, total } =>
            format!(" 历史 {current}/{total} ").into(),
        Msg::AgentGroupTeamKind => "团队代理".into(),
        Msg::AgentGroupSubKind => "子代理".into(),
        Msg::AgentGroupFinished { marker, kind, terminal, total, failed } => format!(
            "{marker} {kind} · 已完成 {terminal}/{total} · 失败 {failed}"
        ).into(),
        Msg::AgentGroupRunning { marker, kind, running, total } =>
            format!("{marker} 运行中 {running}/{total} {kind}…").into(),
        Msg::SubtaskCounts { finished, total, running, pending } => format!(
            " · 已完成 {finished}/{total} · 运行中 {running} · 排队 {pending}"
        ).into(),
        Msg::SubtaskPanelTeamTitle => " 团队".into(),
        Msg::SubtaskPanelSubTitle => " 子任务".into(),
        Msg::SubtaskActivityAnalyzing => "正在分析任务".into(),
        Msg::SubtaskSummaryRunning { count } => format!("{count} 个运行中").into(),
        Msg::SubtaskSummaryPending { count } => format!("{count} 个排队中").into(),
        Msg::SubtaskSummaryFailed { count } => format!("{count} 个失败").into(),
        Msg::SubtaskSummaryStopped { count } => format!("{count} 个已停止").into(),
        Msg::SubtaskPendingSuffix => " · 排队中".into(),
        Msg::SubtaskStatePending => "排队中".into(),
        Msg::SubtaskStateRunning => "运行中".into(),
        Msg::SubtaskStateQueued => "已排队".into(),
        Msg::SubtaskStateDone => "完成".into(),
        Msg::SubtaskStateStopped => "已停止".into(),
        Msg::SubtaskStateFailed => "失败".into(),
        Msg::TodoHeaderTitle => "任务 ".into(),
        Msg::TodoHeaderCounts { completed, in_progress, open } =>
            format!("（已完成 {completed} · 进行中 {in_progress} · 待办 {open}）").into(),
        Msg::TodoMoreFold { hidden, ellipsis } =>
            format!("  +{hidden} 项更多{ellipsis}").into(),
        Msg::MoreLinesHint { count } => format!(" +{count} 行已折叠 ").into(),
        Msg::ScrollHiddenLines { count } =>
            format!("{count} 行被隐藏 · PgUp/PgDn").into(),
        Msg::BodyMoreLines { ellipsis, count } =>
            format!("  {ellipsis} 还有 {count} 行").into(),
        Msg::RoundMeta { round, elapsed } => format!(" · 第 {round} 轮 · {elapsed}").into(),
        Msg::RoundBare { round, elapsed } => format!("第 {round} 轮 · {elapsed}").into(),
        Msg::GoalRowPausedBody => "goal 已暂停".into(),
        Msg::GoalRowPausedMeta => " · 继续对话即恢复 · /goal stop 结束".into(),
        Msg::GoalRowPausedAtCapBody => "goal 暂停".into(),
        Msg::GoalRowPausedAtCapMeta { round } =>
            format!(" · 已达 {round} 轮 · 继续对话即推进").into(),
        Msg::GoalRowSatisfiedBody => "goal 已达成".into(),
        Msg::GoalRowSatisfiedMeta => " · /goal clear 结束".into(),
        Msg::SlashOutputSyncFailed { error } =>
            format!("斜杠命令输出同步失败：{error}").into(),
        Msg::RefreshContextStartFailed { error } =>
            format!("无法启动上下文统计刷新：{error}").into(),
        Msg::RefreshContextFailed { error } =>
            format!("刷新上下文统计失败：{error}").into(),
        Msg::SyncStoppedSharing => "已停止共享当前会话".into(),
        Msg::SyncNotActive => "当前未处于同步模式".into(),
        Msg::WebuiOpenedBrowser { url } => format!("已在浏览器打开 webui：{url}").into(),
        Msg::WebuiOpenManually { url } => format!("请手动在浏览器打开：{url}").into(),
        Msg::WebuiBindFailed { host, port, error } =>
            format!("webui 启动失败：{host}:{port} 起的端口绑定失败（{error}）").into(),
        Msg::WebuiRebindHint { bound_host, host } => format!(
            "\n（webui 已在运行，绑定 {bound_host}；如需改绑 {host}，请先 /webui stop 再重试）"
        )
        .into(),
        Msg::WebuiLanWarning => "\n[!] 主地址为局域网 IP，仅同一网络内的设备可访问；公网访问请用隧道（如 cloudflared / Tailscale）。无 TLS，凡能访问者凭 token 即可进入。".into(),
        Msg::WebuiNonLoopbackWarning => "\n[!] 已绑定非回环地址：凡能访问该地址者凭此 token 即可进入，请仅在可信网络使用（无 TLS）。".into(),
        Msg::WebuiStopped => "已停止 webui server".into(),
        Msg::WebuiNotRunning => "webui server 未在运行".into(),
        Msg::AppServerBindFailed { host, port, error } =>
            format!("绑定 {host}:{port} 失败（{error}）").into(),
        Msg::BgSessionLoadFailed { error } =>
            format!("无法加载后台会话：{error}").into(),
        Msg::McpToolsHeader => "工具列表：\n".into(),
        Msg::McpToolsEmpty { status } => format!("  （无 -- {status}）\n").into(),
        Msg::McpToolsNoServer => "  （无 -- 未配置该服务器）\n".into(),
        Msg::TeamPanelShown => "团队面板已显示。".into(),
        Msg::TeamPanelHidden => "团队面板已隐藏。".into(),
        Msg::TeamPanelCleared => "团队面板已清空。".into(),
        Msg::TeamPanelUsage => "用法：/team [show|hide|status|clear]".into(),
        Msg::InternalError { error } => format!("内部错误：{error}").into(),

        Msg::SteerQueuedLine { prompt } => format!("  ↳ 已排队：{prompt}\n").into(),
        Msg::EmptyCompletionReasoningOnly =>
            "本轮模型只输出了推理、未给出正文。按 Ctrl+O 可查看推理内容；可直接重试或换个问法。".into(),
        Msg::EmptyCompletionNoOutput =>
            "本轮模型未输出任何正文内容。可直接重试或换个问法。".into(),
        Msg::TaskWordFinished => "已结束".into(),
        Msg::TaskWordCompleted => "已完成".into(),
        Msg::UndoFailed { error } => format!("回退失败：{error}").into(),
        Msg::CompactFailed { error } => format!("压缩失败：{error}").into(),
        Msg::WebSourcesPrefix { sources } => format!("来源：{sources}").into(),
        Msg::GoalExecFailed { error } => format!("Goal 执行失败：{error}").into(),
        Msg::TurnDoneDispatched => "已派发".into(),
        Msg::BgProjectionSessionless =>
            "后台运行时已切换到无会话状态".into(),
        Msg::BgProjectionLoadFailed { bucket, session_id, error } =>
            format!("加载后台会话 {bucket}/{session_id} 失败：{error}").into(),
        Msg::BgProjectionDecodeFailed { session_id, error } =>
            format!("解码后台会话 {session_id} 失败：{error}").into(),
        Msg::BgProjectionIdentityMismatchRuntime { session_id, catalog } =>
            format!("后台会话标识不匹配：运行时={session_id:?}，目录={catalog:?}").into(),
        Msg::BgProjectionIdentityMismatchLoaded { expected, loaded } =>
            format!("后台会话标识不匹配：运行时={expected:?}，已加载={loaded:?}").into(),
        Msg::ReviewCompleteClean { changed_files } =>
            format!("代码评审完成 —— {changed_files} 个变更文件中未发现问题。").into(),
        Msg::ReviewHeader { findings, changed_files } =>
            format!("代码评审：{changed_files} 个变更文件中发现 {findings} 个问题。\n").into(),
        Msg::ReviewFindingEntry { index, priority, confidence, location, title } =>
            format!("\n{index}. [{priority} · 置信度 {confidence}] {location}\n   {title}\n").into(),
        Msg::ReviewFixSuggestion { suggestion } =>
            format!("   ↳ 修复建议：{suggestion}\n").into(),
        Msg::ReviewMoreFindings { hidden, shown } =>
            format!("\n… 另有 {hidden} 个问题（按优先级仅显示前 {shown} 个）。\n").into(),
        Msg::ReviewIncompleteHeader { stop, findings, changed_files } =>
            format!("代码评审未完成（{stop}）—— 覆盖不完整，并非一次干净的评审。{changed_files} 个变更文件中有 {findings} 个已确认问题。").into(),
        Msg::ReviewIncompleteReason { reason } => format!("\n原因：{reason}").into(),
        Msg::ReviewDeepIncomplete { total, note } =>
            format!("深度评审未完成 -- 所有维度均失败（0/{total}），覆盖结果不可靠。{note}\n").into(),
        Msg::ReviewDeepClean { changed_files, completed, total, note } =>
            format!("深度评审完成 -- {changed_files} 个变更文件中未发现问题（{completed}/{total} 个维度已完成）{note}。\n").into(),
        Msg::ReviewDeepHeader { findings, changed_files, completed, total } =>
            format!("深度评审：{changed_files} 个变更文件中发现 {findings} 个问题 . {completed}/{total} 个维度已完成").into(),
        Msg::ReviewDeepDeduped { count } => format!(" . 已去重 {count} 项").into(),
        Msg::ReviewVerifyDropped { count } => format!(" . 验证阶段剔除 {count} 项").into(),
        Msg::ReviewFailedDimensions { list } => format!("失败的维度：{list}\n").into(),
        Msg::ReviewDeepFindingEntry { index, priority, confidence, location, dims, title } =>
            format!("\n{index}. [{priority} . 置信度 {confidence}] {location} . 维度：{dims}\n   {title}\n").into(),
        Msg::ReviewActivityHead => "评审".into(),
        Msg::ReviewActivityHeadLabeled { label } => format!("评审 [{label}]").into(),
        Msg::ReviewActivityFindingOne { count } => format!("{count} 个问题").into(),
        Msg::ReviewActivityFindingMany { count } => format!("{count} 个问题").into(),
        Msg::ReviewActivityThinking => "思考中".into(),
        Msg::ReviewActivityReporting => "正在上报问题".into(),
        Msg::ReviewActivityPreparing => "评审 · 正在准备差异".into(),
        Msg::ReviewActivityAnalyzing { files } =>
            format!("评审 · 正在分析 {files} 个文件").into(),
        Msg::ReviewStageVerify => "验证".into(),
        Msg::ModeWordPlan => "计划".into(),
        Msg::ModeWordAcceptEdits => "自动编辑".into(),
        Msg::ModeWordBuild => "构建".into(),
        Msg::ModeWordAuto => "自动".into(),
        Msg::ModeSwitchedLine { mode } => format!("  已切换到{mode}模式。\n").into(),
        Msg::GoalMetBanner { reason } => format!("  [+] 目标已达成：{reason}\n").into(),
        Msg::GoalPausedBanner { reason } => format!("  ⏸ 目标已暂停：{reason}\n").into(),
        Msg::GoalPausedByUserBanner =>
            "  ⏸ 目标已暂停；继续发消息即可恢复，或使用 /goal stop 结束。\n".into(),
        Msg::GoalStoppedBanner { reason } => format!("  [!] 目标已停止：{reason}\n").into(),
        Msg::ParallelDispatchStart { count } => format!("正在并行派发 {count} 个子代理…").into(),
        Msg::WordFailed => "失败".into(),
        Msg::ParallelSummaryOk { ok, total, elapsed } =>
            format!("● 并行编辑 . {ok}/{total} 成功 . 耗时 {elapsed}").into(),
        Msg::ParallelSummaryFail { ok, failed, elapsed } =>
            format!("● 并行编辑 . {ok} 成功 . {failed} 失败 . 耗时 {elapsed}").into(),
        Msg::BashInflightCtrlOHint => "按 Ctrl+o 查看运行中的实时输出".into(),
        Msg::VerboseOnLine { mute, reset } =>
            format!("{mute}  o 详细模式已开启（显示工具输出 + 推理）（Ctrl+o 关闭）{reset}\n").into(),
        Msg::VerboseOffLine { mute, reset } =>
            format!("{mute}  o 详细模式已关闭（Ctrl+o 显示工具输出 + 推理）{reset}\n").into(),
        Msg::EffortLevelLow => "最低推理强度".into(),
        Msg::EffortLevelMedium => "中等推理强度".into(),
        Msg::EffortLevelHigh => "更深度推理".into(),
        Msg::EffortLevelXhigh => "超高推理强度".into(),
        Msg::EffortLevelMax => "最大推理深度".into(),
        Msg::EffortLevelDefault => "恢复 API 默认（保留能力）".into(),
        Msg::EffortUsage { levels } =>
            format!("  用法：/effort {levels} | default\n  快捷键：Ctrl+T\n").into(),
        Msg::EffortCurrent { current, usage } =>
            format!("  当前推理强度：{current}\n{usage}").into(),
        Msg::EffortStatusUnsupported => "不支持".into(),
        Msg::EffortStatusDefault => "默认（API 默认）".into(),
        Msg::EffortSet { level } => format!("  o 推理强度已设为：{level}\n").into(),
        Msg::EffortSetDefault => "  o 推理强度：默认（API 选择；保留能力）\n".into(),

        Msg::McpCfgCommentsWouldDelete { path } => format!(
            "{path} 中包含注释，重写该文件会删除这些注释。\
             请手动编辑该文件，或先移除注释后重试。"
        )
        .into(),
        Msg::McpServerNeedsCommandOrUrl { name } => format!(
            "MCP 服务器“{name}”必须提供 command（stdio）或 url（http）之一"
        )
        .into(),
        Msg::McpAuthTypeUnsupported { name, ty } => {
            format!("MCP 服务器“{name}”使用了不支持的 auth.type“{ty}”").into()
        }
        Msg::McpCfgNameEmpty => "MCP 服务器名称不能为空".into(),
        Msg::McpCfgCommandEmpty => "command 不能为空".into(),
        Msg::McpCfgUrlEmpty => "url 不能为空".into(),
        Msg::McpCfgProviderEmpty => "provider 不能为空".into(),
        Msg::McpCfgRootNotObject => "MCP 配置文件的根节点必须是 JSON 对象".into(),
        Msg::McpOAuthBrowserHintServer { name } => format!(
            "  浏览器没有自动打开？请在浏览器中打开下方网址，为 MCP 服务器“{name}”授权："
        )
        .into(),
        Msg::McpOAuthBrowserHintGithub =>
            "  浏览器没有自动打开？请在浏览器中打开下方网址，为 GitHub MCP 授权：".into(),
        Msg::McpOAuthStateMismatch => "OAuth state 不匹配".into(),
        Msg::McpOAuthRefreshNoRefreshToken { server } => format!(
            "MCP 服务器 {server} 的 OAuth 令牌已过期，且没有保存刷新令牌"
        )
        .into(),
        Msg::McpOAuthRefreshNoTokenEndpoint { server } => format!(
            "MCP 服务器 {server} 的 OAuth 令牌已过期，且没有保存令牌端点"
        )
        .into(),
        Msg::McpOAuthRefreshNoClientId { server } => format!(
            "MCP 服务器 {server} 的 OAuth 令牌已过期，且没有保存 client id"
        )
        .into(),
        Msg::McpOAuthRefreshFailed { status } => {
            format!("MCP OAuth 刷新失败：HTTP {status}").into()
        }
        Msg::McpOAuthHttpNotOAuth { name } => format!(
            "MCP 服务器“{name}”是 HTTP 类型，但未使用 OAuth 认证"
        )
        .into(),
        Msg::McpOAuthStdioUnsupported { name } => format!(
            "MCP 服务器“{name}”使用 stdio；OAuth 登录仅适用于 HTTP 类型的 MCP 服务器"
        )
        .into(),
        Msg::McpOAuthExchangeFailed { status } => {
            format!("MCP OAuth 令牌交换失败：HTTP {status}").into()
        }
        Msg::McpGithubClientIdRequired => "GitHub OAuth 需要提供 client id".into(),
        Msg::McpGithubSecretEnvRequired =>
            "GitHub MCP OAuth 需要 --client-secret-env 参数，或在 mcp.json 中配置 \
             auth.client_secret_env"
                .into(),
        Msg::McpGithubExchangeFailed { status } => {
            format!("GitHub OAuth 令牌交换失败：HTTP {status}").into()
        }
        Msg::McpOAuthRegistrationRequired =>
            "授权服务器不支持动态客户端注册（RFC 7591），MCP OAuth 需要预先注册的 \
             client_id。请在 .mcp.json 的 auth.client_id 中填入预先注册的 client_id 后重试。"
                .into(),
        Msg::McpOAuthRegisterRejected { status, body } => format!(
            "MCP OAuth 动态客户端注册失败：HTTP {status} -- 授权服务器拒绝了该请求。\
             请在 .mcp.json 的 auth.client_id 中填入预先注册的 client_id 后重试。\n\
             响应内容：{body}"
        )
        .into(),
        Msg::McpOAuthRegisterFailed { status, body } => format!(
            "MCP OAuth 动态客户端注册失败：HTTP {status}\n响应内容：{body}"
        )
        .into(),
        Msg::McpOAuthRequiredHint { name } => format!(
            "MCP 服务器 {name} 需要 OAuth；请运行 `rustcode mcp login {name}` 或 \
             `/mcp login {name}`"
        )
        .into(),
        Msg::PluginGitRequired =>
            "未安装 git 或 git 不在 PATH 中。RustCode 需要 git 来管理插件市场。\
             请安装 git（例如 macOS 上运行 `xcode-select --install`，\
             Ubuntu 上运行 `sudo apt install git`）后重启 RustCode。"
                .into(),
        Msg::PluginVerbClone => "克隆".into(),
        Msg::PluginVerbUpdate => "更新".into(),
        Msg::PluginMpNameEmpty { name } => {
            format!("市场名称 `{name}` 净化后为空字符串").into()
        }
        Msg::PluginMpExists { name } => {
            format!("市场 `{name}` 已存在，请先移除").into()
        }
        Msg::PluginMpDirExists { path } => format!(
            "目录 {path} 已存在但未注册，请手动移除该目录"
        )
        .into(),
        Msg::PluginMpNotFound { name } => format!("未找到市场 `{name}`").into(),
        Msg::PluginMpHasPlugins { name } => {
            format!("市场 `{name}` 中仍有已安装的插件，请先卸载这些插件").into()
        }
        Msg::PluginGitCloneFailed { stderr } => format!("git clone 失败：{stderr}").into(),
        Msg::PluginGitPullFailed { stderr } => format!("git pull 失败：{stderr}").into(),
        Msg::PluginGitRevParseFailed { stderr } => {
            format!("git rev-parse 失败：{stderr}").into()
        }

        Msg::PluginReloginHintNeutral =>
            "本构建无托管登录服务；请改用 SSH 地址或更新本地 git 凭证后重试".into(),
        Msg::PluginGitAuthUntrusted { verb, stderr } => format!(
            "{verb}失败：该仓库需要认证（私有仓库）。请改用 SSH 地址（git@...）\
             或先用 git 配置好凭证后重试。\n原始错误：{stderr}"
        )
        .into(),



        Msg::PluginUrlMalformed { url } => format!("git 网址格式不正确：{url}").into(),
        Msg::PluginUrlUnsupported { url } => {
            format!("不支持或格式不正确的 git 网址：{url}").into()
        }
        Msg::PluginUrlMissingHost { url } => format!("git 网址缺少主机名：{url}").into(),
        Msg::PluginUrlMissingPath { url } => {
            format!("git 网址缺少仓库路径：{url}").into()
        }
        Msg::PluginUrlBadScheme { scheme } => {
            format!("不支持的 git 网址协议：{scheme}").into()
        }
        Msg::PluginInstallDirRegistered { path } => {
            format!("插件安装目录已存在且已注册：{path}").into()
        }
        Msg::PluginAlreadyInstalledError { id } => format!(
            "插件 `{id}` 已安装。\n提示：如需重新安装，请先运行 `/plugin uninstall {id}`，\
             再运行 `/plugin install {id}`"
        )
        .into(),
        Msg::PluginAlreadyInProject { path } => {
            format!("插件已安装到项目目录：{path}").into()
        }
        Msg::PluginAlreadyInProjectScope { id, scope } => {
            format!("插件 `{id}` 已安装到项目范围 {scope}").into()
        }
        Msg::PluginSubdirEmpty => "git-subdir 来源的子目录路径为空".into(),
        Msg::PluginSparseCheckoutFailed { stderr } => {
            format!("git sparse-checkout 失败：{stderr}").into()
        }
        Msg::PluginCheckoutFailed { stderr } => format!("git checkout 失败：{stderr}").into(),
        Msg::PluginSubdirNotFound { sub, url } => {
            format!("在仓库 {url} 中未找到 git-subdir 路径 `{sub}`").into()
        }
        Msg::PluginGithubForm { repo } => {
            format!("GitHub 仓库必须是 `owner/name` 形式，收到 `{repo}`").into()
        }
        Msg::PluginGithubChars { repo } => {
            format!("GitHub 仓库 `{repo}` 包含不允许的字符").into()
        }
        Msg::PluginGithubDash { repo } => {
            format!("GitHub 仓库 `{repo}` 的各段不能以 '-' 开头").into()
        }
        Msg::PluginLocalMissing { path } => {
            format!("本地插件来源路径不存在：{path}").into()
        }
        Msg::PluginPinCheckoutFailed { rev, stderr } => {
            format!("git checkout {rev} 失败：{stderr}").into()
        }
        Msg::PluginSourceBadComponents { source } => {
            format!("插件来源路径“{source}”包含不允许的路径分量").into()
        }
        Msg::PluginMpNotRegistered { name } => {
            format!("市场 `{name}` 未注册").into()
        }
        Msg::PluginNotInMarketplace { plugin, marketplace } => {
            format!("在市场 `{marketplace}` 中未找到插件 `{plugin}`").into()
        }
        Msg::CtxUsageHeader => "上下文用量".into(),
        Msg::CtxUsageNoTurns => "（请至少完成一轮对话 -- 统计在每轮结束时记录）".into(),
        Msg::CtxUsageWaiting => "（等待首轮完成 -- 当前仅为部分统计）".into(),
        Msg::CtxProvider => "Provider".into(),
        Msg::CtxCtxName => "ctx".into(),
        Msg::CtxLabelSystemPrompt => "系统提示".into(),
        Msg::CtxLabelToolDefs => "工具定义".into(),
        Msg::CtxLabelColdZone => "冷区".into(),
        Msg::CtxLabelMessages => "消息".into(),
        Msg::CtxLabelFree => "空闲".into(),
        Msg::CtxMessagesInWindow { n } => format!("窗口内消息数：{n}").into(),
        Msg::CtxSystemPromptHeader => "=== 系统提示 ===".into(),
        Msg::CtxSystemPromptEmpty => "（为空 -- 完成一轮对话后捕获）".into(),
        Msg::CtxTokensSuffix => "tokens".into(),
        Msg::CompactNothingShort => "（无需压缩 -- 当前对话较短）\n".into(),
        Msg::CompactStarting => "（正在使用 LLM 摘要进行压缩...）\n".into(),
        Msg::CompactInterrupted => "（压缩已中断 -- coding runtime 已变更或停止）\n".into(),
        Msg::CompactUnavailableDuringSync =>
            "同步模式下 /compact 暂不可用；请先执行 /sync off".into(),
        Msg::CompactUnavailableDuringResync =>
            "本地 runtime 尚未恢复最新同步会话，暂不能执行 /compact".into(),
        Msg::LocalRuntimeRestorePending =>
            "本地 runtime 正在恢复同步会话，请稍候".into(),
        Msg::LocalRuntimeRestoreTimedOut =>
            "本地 runtime 恢复超时，已重新接回 Live 同步".into(),
        Msg::CompactNothingNoSavings { before, after } =>
            format!("（无需压缩 -- 压缩后不会节省 token：{} -> {}）\n", before, after).into(),
        Msg::CompactDropped { messages, before, after } =>
            format!("（已压缩 -- 丢弃 {} 条消息，{} -> {} tokens）\n", messages, before, after).into(),
        Msg::Compacting => "正在压缩...".into(),
        Msg::CompactingSlow => "正在压缩...（较慢）".into(),
        Msg::CompactMarkDrain { messages, before, after } =>
            format!("已压缩 . 摘要 {} 条 . ~{}->~{} tok", messages, before, after).into(),
        Msg::CompactMarkStub { saved } =>
            format!("已折叠工具输出 . 节省 ~{} tok", saved).into(),
        Msg::CompactNegligibleSavings => "（当前会话无需压缩）\n".into(),
        Msg::GoalHelp =>
            "  /goal -- 朝着设定的条件自主进行多轮工作。\n  \
             用法：\n  \
             \u{20}\u{20}/goal <条件>          设定新目标；智能体循环执行直到评估器判定达成\n  \
             \u{20}\u{20}/goal                 显示当前目标状态\n  \
             \u{20}\u{20}/goal status          同上\n  \
             \u{20}\u{20}/goal clear           停止当前目标（别名：stop、off、reset、none、cancel）\n  \
             \u{20}\u{20}/goal help            显示本帮助\n  \
             说明：\n  \
             \u{20}\u{20}- 每轮由一个快速模型评估；通过 ~/.rustcode/config.toml 中的 [providers] +\n  \
             \u{20}\u{20}\u{20}\u{20}evaluator_provider 配置。\n  \
             \u{20}\u{20}- 没有内置的轮次 / 时间上限----请在条件文本中自行表达预算\n  \
             \u{20}\u{20}\u{20}\u{20}（例如 \"或在 20 轮后停止\"）。Claude Code 的 /goal 也是这样工作的。\n  \
             \u{20}\u{20}- 随时可用 Esc / Ctrl+C 停止目标。\n".into(),
        Msg::GoalStatus { condition, round, mins, secs } =>
            format!("  * 目标：{}\n  轮次：{}\n  已用时：{}分 {}秒\n", condition, round, mins, secs).into(),
        Msg::GoalNoActive =>
            "  当前没有进行中的目标。\n  用法：/goal <条件>   |   /goal help\n".into(),
        Msg::GoalCleared => "  已清除目标。\n".into(),

        // ── /loop ──
        Msg::LoopStatus { label, round, mins, secs } =>
            format!("  ↻ loop：{} . 第 {} 轮 . {}分 {}秒\n", label, round, mins, secs).into(),
        Msg::LoopNoActive =>
            "  当前没有进行中的 /loop。\n  用法：/loop <间隔> <命令>  或  /loop <任务>\n".into(),
        Msg::LoopCleared => "  已停止 /loop。\n".into(),
        Msg::LoopRound { round, stats } =>
            format!("[*] loop 第 {} 轮 . {}", round, stats).into(),
        Msg::LoopStopped => "[!] loop 已停止（达到次数上限）\n".into(),
        Msg::LoopEnded { reason } =>
            format!("  ↻ Loop 已结束：{reason}\n").into(),
        Msg::LoopNoPersistHint =>
            "  （提示：重启 / 恢复会话后该 loop 不会保留）".into(),
        Msg::CmdDescLoop =>
            "按固定间隔重复执行提示/命令，或让模型自主决定节奏".into(),
        Msg::ModelNoImageSupport { model } => format!(
            "当前模型 \"{}\" 不支持图片输入，且未配置 vision_preprocessor_provider。\
             请用 /model 切换到支持视觉的模型，或在配置中设置 vision_preprocessor_provider。",
            model
        )
        .into(),
        Msg::VisionPreprocessorUnresolvable { model, provider } => format!(
            "当前模型 \"{}\" 不支持图片输入；已配置的 vision_preprocessor_provider \"{}\" \
             无法解析（请检查名称是否与配置中的 provider/model 一致）。\
             请修正该名称，或用 /model 切换到支持视觉的模型。",
            model, provider
        )
        .into(),
        // ── --dangerously-skip-permissions / -y ──
        Msg::BypassWarningBanner =>
            "\u{26a0} --dangerously-skip-permissions 已启用：所有工具调用将自动批准（无权限提示）\n".into(),
        Msg::BypassWarningHeadless =>
            "[headless] --dangerously-skip-permissions：所有工具调用将自动批准".into(),

        Msg::AdminWarningBanner =>
            "\x1b[33m\u{26a0} 警告：正在以管理员权限运行。\n   模型可能可以访问系统文件。\n   建议改用普通权限、并在受限的工作目录中运行 {brand}。\x1b[39m\n".into(),
        Msg::AdminWarningHeadless =>
            "[warning] 正在以管理员权限运行 -- 模型可能可以访问系统文件。".into(),

        Msg::CtrlCAgainToExit => "  （再次按 Ctrl+C 退出）\n".into(),
        Msg::EscAgainToUndo => "  （再次按 Esc 打开回退选择）\n".into(),
        Msg::BashInputHint => "回车执行 bash 命令".into(),
        Msg::ShellModeHint => "! 进入 shell 模式".into(),
        Msg::PendingMessagesTitle =>
            "将在下一次工具调用后提交的消息（按 Esc 中断并立即发送）".into(),
        Msg::PendingMessagesNotSent { count } =>
            format!("运行时已停止，{count} 条待处理消息未发送").into(),
        Msg::HintMultiLineInput =>
            "  \u{24d8} 多行输入：在行尾加 `\\` 再按 Enter。\n    \
            所有终端均可用。（Shift / Alt / Ctrl + Enter 在部分终端也支持，\n    \
            取决于该终端的键盘协议 -- 可以试试看。）\n\n"
                .into(),

        // ── /bg（后台会话）──
        Msg::BgHelp =>
            "  /bg                 将当前会话放到后台，打开新的前台会话\n  /bg list            列出后台会话\n  /bg <N>             恢复第 N 号后台会话\n  /bg drop <N>        丢弃第 N 号后台会话\n  /bg help            显示此帮助\n".into(),
        Msg::BgListEmpty => "  没有后台会话。\n".into(),
        Msg::BgListHeader => "  #   ID        状态       创建时间   摘要\n".into(),
        Msg::BgListRow { slot, short_id, state, age, summary } =>
            format!("  {:<3} {:<8}  {:<9}  {:<8}  {}\n", slot, short_id, state, age, summary).into(),
        Msg::BgStateRunning => "运行中".into(),
        Msg::BgStateIdle => "空闲".into(),
        Msg::BgStateDone => "已完成".into(),
        Msg::BgStateCancelled => "已取消".into(),
        Msg::BgStateError => "错误".into(),
        Msg::BgAgeNow => "刚刚".into(),
        Msg::BgAgeMinutes { n } => format!("{n} 分钟").into(),
        Msg::BgAgeHours { n } => format!("{n} 小时").into(),
        Msg::BgAgeDays { n } => format!("{n} 天").into(),
        Msg::BgSlotLimitReached { max } =>
            format!("后台槽位已达上限（{max}）").into(),
        Msg::BgBackgroundCurrent { new_id, slot, old_id, state } =>
            format!("  新前台会话 [{new_id}]\n  后台：[#{slot}] {old_id}（状态：{state}）\n").into(),
        Msg::BgInvalidSlot { slot, available } =>
            format!("无效的后台槽位 {slot}（可用：{available}）").into(),
        Msg::BgNoRuntimeClient => "后台槽位没有运行时客户端".into(),
        Msg::BgSwitchProviderTransition =>
            "/bg 无法切换前台：提供商切换正在进行中".into(),
        Msg::BgSwitchRuntimePending =>
            "/bg 无法切换前台：交互式运行时请求尚未完成".into(),
        Msg::BgSwitchLiveSync =>
            "/bg 无法切换前台：实时同步已连接，请先执行 /sync off".into(),
        Msg::BgTaskFallbackName => "后台任务".into(),
        Msg::BgStartFailed { error } =>
            format!("后台任务无法启动：{error}").into(),
        Msg::BgResumed { slot, short_id } =>
            format!("  已恢复后台 [#{slot}] {short_id}\n").into(),
        Msg::BgPreviousForegroundMoved { slot } =>
            format!("  原前台会话已移至 [#{slot}]\n").into(),
        Msg::BgDropped { slot, short_id } =>
            format!("  已丢弃后台 [#{slot}] {short_id}\n").into(),
        Msg::BgTaskStarted { slot, short_id } =>
            format!("  后台：[#{slot}] {short_id}（状态：运行中）\n").into(),
        Msg::BgTaskTimedOut { secs } =>
            format!("后台任务超时（{secs} 秒）。").into(),
        Msg::BgTaskError { error } =>
            format!("错误：{error}").into(),
        Msg::BgTaskCancelled => "已取消。".into(),
        Msg::BgTaskNoSummary => "任务完成（无摘要文本）。".into(),
        // ── CLI rustcode --help i18n ──
        Msg::CliAbout => "终端中的 AI 编程助手".into(),

        Msg::CliAboutStatus => "查看当前供应商与登录状态".into(),
        Msg::CliAboutUpgrade => "就地升级 rustcode 到最新发布版本".into(),
        Msg::CliAboutRollback => "回退到上一个版本（与 .bak 交换）".into(),
        Msg::CliAboutMcp => "管理 .mcp.json 中的 MCP 服务器配置".into(),
        Msg::CliAboutDaemon => "启动用于 IDE 集成的 HTTP 守护进程".into(),
        Msg::CliAboutWebui => "启动本地浏览器 webui".into(),
        Msg::CliAboutPlugin => "管理技能/命令插件".into(),
        Msg::CliAboutUninstall => "卸载 {brand}：移除二进制文件、PATH 编辑和数据".into(),
        Msg::CliAboutSetup => "安装种子文件（技能/命令/钩子/MCP）到 ~/.rustcode/".into(),
        Msg::CliAboutHooks => "管理钩子（列表、测试、启用/禁用）".into(),
        Msg::CliAboutHooksList => "列出所有已加载钩子及其状态".into(),
        Msg::CliAboutHooksTest => "按名称测试指定钩子".into(),
        Msg::CliAboutHooksPaths => "显示钩子配置路径".into(),
        Msg::CliAboutPluginMarketplace => "市场注册操作".into(),
        Msg::CliAboutPluginInstall => "从已注册的市场安装插件".into(),
        Msg::CliAboutPluginUninstall => "卸载已安装的插件".into(),
        Msg::CliAboutPluginList => "列出已安装的插件".into(),
        Msg::CliAboutMarketplaceAdd => "克隆市场 git 仓库并在本地注册".into(),
        Msg::CliAboutMarketplaceRemove => "删除已注册的市场".into(),
        Msg::CliAboutMarketplaceUpdate => "重新拉取已注册的市场并刷新插件索引".into(),
        Msg::CliAboutMarketplaceList => "列出已注册的市场".into(),
        Msg::CliAboutWiki => "生成项目 wiki：架构图与模块文档".into(),
        Msg::WikiGenerating => "正在生成项目 wiki...".into(),
        Msg::WikiSummary { modules, files, path } => {
            format!("已生成 wiki：{modules} 个模块、{files} 个文件，写入 {path}").into()
        }
        Msg::WikiAutoSynced { modules, files, path } => {
            format!("wiki 已自动同步：{modules} 个模块、{files} 个文件更新于 {path}").into()
        }
        Msg::WikiForeignConflict { path } => {
            format!("检测到目录 {path} 已存在且不是由 rustcode-wiki 自动生成；为安全起见已忽略，未改动你的内容。如需重新生成，请改用其它 --out-dir 或先手动删除该目录。").into()
        }
        Msg::WikiFilePreserved { path } => {
            format!("已保留你对 {path} 的手动修改（使用 --force 可覆盖）").into()
        }
        Msg::WikiFileRemoved { path } => {
            format!("已移除 {path}（对应模块已不存在）").into()
        }
        Msg::WikiSyncUpToDate => "wiki 已是最新（未检测到源码变更）".into(),
        Msg::WikiEnriching => "正在用已配置的 LLM 充实模块文档...".into(),
        Msg::WikiEnriched { count } => {
            format!("LLM 充实完成，共 {count} 个模块").into()
        }
        Msg::WikiEnrichSkipped { reason } => format!("已跳过 LLM 充实：{reason}").into(),
        Msg::CliAboutMcpAdd => "添加或替换 stdio MCP 服务器".into(),
        Msg::CliAboutMcpAddOauth => "按 URL 添加远程 OAuth MCP 服务器（任意服务商）".into(),
        Msg::CliAboutMcpAddGithubOauth => "使用 OAuth 添加 GitHub 远程 MCP 服务器".into(),
        Msg::CliAboutMcpLogin => "完成远程 MCP 服务器的 OAuth 登录".into(),
        Msg::CliAboutMcpLogout => "删除远程 MCP 服务器的已保存 OAuth 凭证".into(),
        Msg::CliHelpContinue => "继续上一次会话而不是启动新会话".into(),
        Msg::CliHelpProvider => "指定使用的 Provider（覆盖配置默认值）".into(),
        Msg::CliHelpModel => "指定使用的模型（覆盖配置中的 Provider 模型）".into(),
        Msg::CliHelpLang => "设置界面语言（如 en、zh-CN、zh）".into(),
        Msg::CliHelpConfig => "配置文件路径".into(),
        Msg::CliHelpDir => "工作目录（默认为当前目录）".into(),
        Msg::CliHelpPrompt => "在无头（非交互）模式下运行的提示".into(),
        Msg::CliHelpPromptFile => "从文件读取提示".into(),
        Msg::CliHelpVerbose => "在 stderr 上显示工具调用、token 用量和回合摘要".into(),
        Msg::CliHelpDev => "禁用本次启动的自动更新".into(),
        Msg::CliHelpDangerouslySkipPermissions => "跳过所有权限提示 -- 自动批准每个工具调用".into(),
        Msg::CliHelpPermissionMode => "权限/沙箱模式：default、accept-edits、auto、plan、bypass-permissions（auto / bypass-permissions 等价于 --dangerously-skip-permissions / --yolo）".into(),
        Msg::CliHelpForce => "即使已是最新版本也重新安装".into(),
        Msg::CliHelpPortDaemon => "监听端口（默认：13456）".into(),
        Msg::CliHelpIdleTimeout => "空闲关闭超时（秒）；0 禁用".into(),
        Msg::CliHelpPortWebui => "端口（默认：13457）".into(),
        Msg::CliHelpHost => "绑定地址（默认：0.0.0.0；改 127.0.0.1 则仅本机可访问）".into(),
        Msg::CliHelpUninstallYes => "跳过提示；使用每组的默认决定".into(),
        Msg::CliHelpUninstallPurge => "完全清除 ~/.rustcode/".into(),
        Msg::CliHelpUninstallKeepData => "完全保留 ~/.rustcode/".into(),
        Msg::CliHelpUninstallDryRun => "仅打印计划；不执行操作".into(),
        Msg::CliHelpMcpGlobal => "写入 ~/.rustcode/mcp.json 而非 <dir>/.mcp.json".into(),
        Msg::CliHelpMcpDir => "项目 .mcp.json 的目录".into(),
        Msg::CliHelpMcpName => "服务器键名".into(),
        Msg::CliHelpMcpUrl => "服务器 HTTP URL（将从中发现 OAuth 元数据）".into(),
        Msg::CliHelpMcpProvider => "可选的 OAuth 服务商提示（通常从服务器配置读取）".into(),
        Msg::CliHelpMcpClientId => "OAuth 客户端 ID".into(),
        Msg::CliHelpHooksTestName => "要测试的钩子名称".into(),
        Msg::CliHelpPluginSpec => "如 plugin@marketplace".into(),
        Msg::CliHelpMarketplaceUrl => "市场仓库的 Git URL".into(),
        Msg::CliHelpMarketplaceName => "市场名称".into(),
        Msg::CliAboutHelp => "打印帮助信息".into(),
        Msg::CliHelpMcpCommand => "可执行文件及参数".into(),
        Msg::CliAboutCompletion => "在标准输出生成 shell 补全脚本".into(),
        Msg::CliHelpCompletionShell => "要生成补全的 shell".into(),
        Msg::CliAboutResume => "按 id 或名称恢复会话（在其上启动 TUI）".into(),
        Msg::CliHelpResumeSession => "要恢复的会话 id 或名称（默认：最近一次会话）".into(),
        Msg::CliAboutSchedule => "管理定时任务（添加/列表/移除/启用/禁用/同步）".into(),
        Msg::CliAboutScheduleAdd => "添加新的定时任务".into(),
        Msg::CliAboutScheduleList => "列出所有定时任务".into(),
        Msg::CliAboutScheduleRemove => "按 id 移除定时任务".into(),
        Msg::CliAboutScheduleEnable => "启用定时任务".into(),
        Msg::CliAboutScheduleDisable => "禁用定时任务（不再触发）".into(),
        Msg::CliAboutScheduleRun => "立即运行一次定时任务".into(),
        Msg::CliAboutScheduleSync => "将系统调度器注册状态与已存储的任务列表同步".into(),
        Msg::CliHelpSchedId => "任务 id".into(),
        Msg::CliHelpSchedTitle => "任务的可读名称".into(),
        Msg::CliHelpSchedPrompt => "任务触发时发送给代理的提示文本".into(),
        Msg::CliHelpSchedCwd => "代理会话的工作目录（默认为当前目录）".into(),
        Msg::CliHelpSchedDaily => "每天 HH:MM 运行（如 \"09:00\"）".into(),
        Msg::CliHelpSchedWeekly => "每周运行，格式 N@HH:MM（N=1..7，1=周一）".into(),
        Msg::CliHelpSchedEvery => "每 N 分钟运行一次（如 \"30m\"）".into(),
        Msg::CliHelpSchedHourly => "每小时运行一次".into(),
        Msg::CliHelpSchedCron => "cron 表达式（如 \"0 9 * * 1-5\"）".into(),
        Msg::CliHelpSchedMode => "权限模式：plan | accept_edits | auto".into(),
        Msg::CliHelpSchedNotify => "通知级别：off | important | all".into(),

        // ── rustcode ide ──
        Msg::CliAboutIde => "检测已安装的 IDE 并安装 RustCode 扩展".into(),
        Msg::CliAboutIdeList => "列出已检测到的 IDE 及其扩展状态".into(),
        Msg::CliAboutIdeInstall => "将 RustCode 扩展安装到 IDE 中".into(),
        Msg::CliHelpIdeInstallIde => "目标 IDE：vscode | cursor | vscodium | jetbrains".into(),
        Msg::CliHelpIdeInstallAll => "安装到所有检测到的 IDE（忽略位置参数）".into(),
        Msg::CliIdeHeader => "已检测到的 IDE：".into(),
        Msg::CliIdeDetected { ide, path, ext } => format!(
            "  {ide:<14} {path}  [{ext}]"
        ).into(),
        Msg::CliIdeNoneDetected => "PATH 上未检测到受支持的 IDE。".into(),
        Msg::CliIdeInstalling { ide } => format!("[*] 正在为 {ide} 安装 RustCode 扩展...").into(),
        Msg::CliIdeInstallOk { ide } => format!("[+] {ide}：扩展已安装。").into(),
        Msg::CliIdeInstallFailed { ide, error } => format!("[!] {ide}：安装失败：{error}").into(),
        Msg::CliIdeNotFound { ide } => format!("[!] {ide}：PATH 上未找到可执行文件。").into(),
        Msg::CliIdeManualRequired { ide, marketplace_url } => format!(
            "[*] {ide}：不支持自动安装。请手动从以下地址安装：\n    {marketplace_url}"
        ).into(),
        Msg::CliIdeUnknown { ide } => format!(
            "[!] 未知的 IDE \"{ide}\"。支持：vscode、cursor、vscodium、jetbrains。"
        ).into(),

        // ── rustcodex 独立 CLI（rustcode-clix）──
        Msg::ClixAbout => "RustCode 独立命令行（新栈）".into(),
        Msg::ClixAboutCode => "交互式编码代理（完整装配：工具 + 代码索引 + Web + 技能 + MCP + 会话 + 记忆）。".into(),
        Msg::ClixAboutSessions => "列出当前项目可恢复的会话。".into(),
        Msg::ClixAboutReview => "审查本地 git 差异并输出结构化发现。".into(),
        Msg::ClixSessionsNone { dir, bucket } => {
            format!("{dir} 没有可恢复的会话（存储位置：{bucket}）").into()
        }
        Msg::ClixSessionsRow { id, name, turns, ts } => {
            format!("{id}  {name:<28}  {turns:<4} 轮  {ts}").into()
        }
        Msg::ClixProjectDirNotFound => "项目目录不存在".into(),
        Msg::ClixWorkingDirNotFound => "工作目录不存在".into(),
        Msg::ClixRepoNotFound { path } => format!("仓库目录不存在：{path}").into(),
        Msg::ClixMissingBaseUrl => "缺少 base URL：请传入 --base-url、设置 $RUSTCODE_BASE_URL，或在配置中提供供应商".into(),
        Msg::ClixMissingBaseUrlReview => "缺少 base URL：请传入 --base-url、设置 $RUSTCODE_BASE_URL，或在配置的供应商条目中添加 base_url".into(),
        Msg::ClixMissingModel => "缺少模型：请传入 --model、设置 $RUSTCODE_MODEL，或在配置中提供供应商".into(),
        Msg::ClixMissingModelReview => "缺少模型：请传入 --model、设置 $RUSTCODE_MODEL，或在配置的供应商条目中添加 model".into(),
        Msg::ClixNoSessionToContinue => {
            "本项目没有可继续的会话 -- 不加 --continue 直接启动一个新会话".into()
        }
        Msg::ClixConfigLoadFailed { path } => format!("加载配置失败：{path}").into(),
        Msg::ClixConfigParseFailed => "解析 config.toml 失败".into(),
        Msg::ClixConfigReadFailed { path } => format!("无法读取配置文件：{path}").into(),
        Msg::ClixConfigMalformed { path } => format!("配置文件格式错误：{path}").into(),
        Msg::ClixPreparing { model } => format!("准备中（{model}）...").into(),
        Msg::ClixRuntimeStartFailed => "运行时启动失败".into(),
        Msg::ClixSessionNew { id } => format!("会话 {id}（新建）").into(),
        Msg::ClixSessionResumed { id } => format!("会话 {id}（已恢复）").into(),
        Msg::ClixTurnAbnormal { reason } => format!("回合未正常结束：{reason}").into(),
        Msg::ClixTurnSnapshotUnavailable { reason, error } => {
            format!("{reason} 之后回合快照不可用：{error}").into()
        }
        Msg::ClixAgentTerminatedUnexpectedly => "代理意外终止".into(),
        Msg::ClixInteractiveHint => "交互模式 -- 输入 /help 查看命令，/quit 退出".into(),
        Msg::ClixStdinError { error } => format!("{error} -- 正在退出").into(),
        Msg::ClixSigintExit => "（正在退出 -- 会话已保存；下次可用 /quit，或再按一次 Ctrl-C）".into(),
        Msg::ClixAgentTerminatedNote => "正在退出".into(),
        Msg::ClixSessionSaved { id } => {
            format!("会话已保存 -- 恢复命令：rustcodex code --resume {id}").into()
        }
        Msg::ClixCancelling => "正在取消 ...".into(),
        Msg::ClixRetry {
            reason,
            backoff_secs,
            attempt,
            max_attempts,
        } => format!("API 错误 {reason}；{backoff_secs} 秒后重试（{attempt}/{max_attempts}）").into(),
        Msg::ClixStreamRecovered => "已从中断的流恢复".into(),
        Msg::ClixStreamContinuing {
            attempt,
            max_attempts,
        } => format!("正从已保存的进度安全继续（{attempt}/{max_attempts}）").into(),
        Msg::ClixCompacting => "正在压缩 ...".into(),
        Msg::ClixCompacted => "已压缩".into(),
        Msg::ClixCompactedNoGain => "已压缩 -- 无收益，未写入".into(),
        Msg::ClixCompactFailed { error } => format!("压缩失败：{error}").into(),
        Msg::ClixTurnEnded { reason } => format!("回合结束：{reason}").into(),
        Msg::ClixToolResultChars { count } => format!("（{count} 字符）").into(),
        Msg::ClixToolResultCharsNamed { name, count } => {
            format!("{name}（{count} 字符）").into()
        }
        Msg::ClixYoloAutoAllow { tool, args } => format!("自动允许 {tool} {args}").into(),
        Msg::ClixDiscardedTypedAhead { count } => {
            format!("（已丢弃 {count} 行提前输入的内容 -- 批准提示需要当场给出回答）").into()
        }
        Msg::ClixApprovalNeeded { tool, args } => format!("需要批准：{tool} {args}").into(),
        Msg::ClixApprovalPrompt => "允许？[y = 一次 / always = 记住 / N = 拒绝]".into(),
        // 单行 + 显式 \n：Rust 字符串的 `\` 续行会吞掉下一行前导空格。
        Msg::ClixSlashHelp => "  /remember [-g] <事实>    追加到项目（-g：全局）memory.md\n  /forget [-g] <关键词>  删除匹配的记忆条目\n  /memory                 查看模型拿到的合并记忆\n  /compact [焦点]         压缩对话（回合边界）\n  /sessions               列出本项目的会话\n  /quit                   退出（会话会保留）".into(),
        Msg::ClixRememberUsage => "用法：/remember [-g] <事实>".into(),
        Msg::ClixForgetUsage => "用法：/forget [-g] <关键词>".into(),
        Msg::ClixRemembered { scope } => {
            format!("已记住（{scope}）-- 下次启动会话时注入").into()
        }
        Msg::ClixMemoryWriteFailed { error } => format!("写入记忆失败：{error}").into(),
        Msg::ClixMemoryUpdateFailed { error } => format!("更新记忆失败：{error}").into(),
        Msg::ClixForgetNoMatch { keyword } => format!("没有匹配 '{keyword}' 的条目").into(),
        Msg::ClixForgotEntry { entry } => format!("已忘记：{entry}").into(),
        Msg::ClixMemoryEmptyHint => "（记忆为空 -- 用 /remember <事实> 添加）".into(),
        Msg::ClixCompactionRequested => "已请求压缩".into(),
        Msg::ClixUnknownSlash { name } => {
            format!("未知命令 /{name} -- 输入 /help 查看（以 / 开头的是命令）").into()
        }
        Msg::ClixStdinConflict { flags } => {
            format!("{flags} 都从标准输入读取；请只保留一个用 -，其余给文件路径").into()
        }
        Msg::ClixNoChanges => "没有需要审查的更改。".into(),
        Msg::ClixRulesInjected { files, chars } => {
            format!("已为 {files} 个变更文件注入规则（{chars} 字符）").into()
        }
        Msg::ClixRulesNone => "没有语言规则匹配变更文件".into(),
        Msg::ClixTraceCustomTask { chars } => format!("自定义任务（{chars} 字符）").into(),
        Msg::ClixTraceChangedLines { lines } => format!("{lines} 个变更行").into(),
        Msg::ClixRunning { label, model } => {
            format!("正在运行 {label}（模型 {model}）...").into()
        }
        Msg::ClixTraceTools { count, profile } => {
            format!("-- 跟踪 -- {count} 次工具调用：{profile}").into()
        }
        Msg::ClixTraceTokens {
            prompt,
            completion,
            cached,
        } => format!("-- token -- 提示 {prompt} / 补全 {completion} / 缓存 {cached}").into(),
        Msg::ClixPassInitial => "首轮".into(),
        Msg::ClixPassCoverage => "覆盖率复审".into(),
        Msg::ClixScopeDropped { dropped, files } => {
            format!("丢弃了 {dropped} 条锚定在 {files} 个变更文件之外的发现").into()
        }
        Msg::ClixCoverageSkippedFlag => "已跳过 -- --no-coverage".into(),
        Msg::ClixCoverageRereview { count, files } => {
            format!("{count} 个变更文件没有发现；正在复审：{files}").into()
        }
        Msg::ClixCoverageTrace { count, profile } => {
            format!("-- 覆盖率跟踪 -- {count} 次工具调用：{profile}").into()
        }
        Msg::ClixCoverageRecovered { added } => format!("复审补回 {added} 条发现").into(),
        Msg::ClixCoverageSkippedNoSignal { findings } => {
            format!("已跳过 -- 过滤后没有高信号的未覆盖文件（首轮发现数={findings}）").into()
        }
        Msg::ClixCoverageSkippedIncomplete { reasons } => {
            format!("已跳过 -- 首轮未完成（{reasons}）").into()
        }
        Msg::ClixCoverageCapped { cap, dropped } => {
            format!("复审上限为 {cap} 个最高优先级文件；丢弃了 {dropped} 个较低优先级的未覆盖文件").into()
        }
        Msg::ClixReviewIncompleteNoFindings => "审查未完成 -- 没有收集到发现。".into(),
        Msg::ClixReviewClean => "没有发现 -- 此 diff 看起来没有问题。\n".into(),
        Msg::ClixFindingsHeader {
            total,
            p0,
            p1,
            p2,
            p3,
        } => format!("发现 {total} 条：{p0} 个 P0，{p1} 个 P1，{p2} 个 P2，{p3} 个 P3\n\n").into(),
        Msg::ClixReviewerSummary { text } => format!("\n-- 审查员总结 --\n{text}").into(),
        Msg::ClixReviewBailIncomplete { why } => {
            format!("审查未完成（{why}）：没有收集到发现").into()
        }
        Msg::ClixReviewEndedEarly { why, count } => {
            format!("警告：审查提前结束（{why}）；停止前已收集 {count} 条发现").into()
        }
        Msg::ClixTaskStdinFailed => "从标准输入读取任务失败".into(),
        Msg::ClixTaskFileFailed { path } => format!("读取任务文件失败：{path}").into(),
        Msg::ClixTaskFileEmpty { path } => format!("任务文件为空：{path}").into(),
        Msg::ClixPromptStdinFailed => "从标准输入读取系统提示失败".into(),
        Msg::ClixPromptFileFailed { path } => format!("读取系统提示文件失败：{path}").into(),
        Msg::ClixDiffStdinFailed => "从标准输入读取 diff 失败".into(),
        Msg::ClixDiffFileFailed { path } => format!("读取 diff 文件失败：{path}").into(),
        Msg::ClixGhFailed => "运行 `gh` 失败 -- 请安装 GitHub CLI，或通过 `--diff-file -` 管道传入 diff（如 GitLab 等其他平台）".into(),
        Msg::ClixGhPrFailed { pr, error } => format!("`gh pr diff {pr}` 失败：{error}").into(),
        Msg::ClixGitFailed => "运行 `git` 失败 -- 它是否已安装并在 PATH 上？".into(),
        Msg::ClixGitDiffFailed { error } => format!("git diff 失败：{error}").into(),
        Msg::ClixSkillDirNotFound { path } => format!("--skill-dir 目录不存在：{path}").into(),
        Msg::ClixHelpCodePrompt => "一次性提示：运行单个回合、打印回答后退出（会话仍会保存且可恢复）。".into(),
        Msg::ClixHelpCodeDir => "代理工具所限定的工作目录。".into(),
        Msg::ClixHelpCodeResume => "按 id 恢复会话（参见 `rustcodex sessions`）。".into(),
        Msg::ClixHelpCodeContinue => "恢复本项目最近更新的会话。".into(),
        Msg::ClixHelpCodeYolo => "自动批准所有有风险的工具调用（CI / 受信任运行）。".into(),
        Msg::ClixHelpCodeNoMcp => "跳过 MCP 服务器连接。".into(),
        Msg::ClixHelpCodeNoMemory => "跳过 memory.md 注入。".into(),
        Msg::ClixHelpCodeNoWeb => "跳过 web_fetch / web_search 工具。".into(),
        Msg::ClixHelpCodeModel => "模型 id（覆盖 $RUSTCODE_MODEL）。".into(),
        Msg::ClixHelpCodeApiKey => "第三方 Provider 的 API 密钥（覆盖 $RUSTCODE_API_KEY）。".into(),
        Msg::ClixHelpCodeBaseUrl => "第三方 Provider 的基础 URL（覆盖 $RUSTCODE_BASE_URL）。".into(),
        Msg::ClixHelpCodeProvider => "使用配置文件中指定的 `[providers.<name>]` 条目（覆盖 `default_provider`）。".into(),
        Msg::ClixHelpCodeConfig => "配置文件路径（默认：~/.rustcode/config.toml）。".into(),
        Msg::ClixHelpCodeStreamTimeout => "等待每个流事件的最长秒数（存活守卫）。".into(),
        Msg::ClixHelpSessionsDir => "项目目录（默认：当前目录）。".into(),
        Msg::ClixHelpReviewRepo => "仓库根目录（默认：当前目录）。".into(),
        Msg::ClixHelpReviewJson => "以 JSON 输出审查结果，而不是人类可读的报告。".into(),

        // ── /usage 命令 ──
        Msg::UsageUnavailableNeutral =>
            "当前构建不支持托管账号用量查询。运行 /cost 可查看本会话的本地 Token 用量。".into(),

        Msg::NotifyTitleDone => "RustCode 已完成".into(),
        Msg::NotifyTitleCancelled => "RustCode 已取消".into(),
        Msg::NotifyTitleFailed => "RustCode 执行失败".into(),
        Msg::NotifyTitleStopped => "RustCode 已停止".into(),
        Msg::NotifyStatusDone => "已完成".into(),
        Msg::NotifyStatusCancelled => "已取消".into(),
        Msg::NotifyStatusFailed => "失败".into(),
        Msg::NotifyStatusStopped => "已停止".into(),
        Msg::NotifyRounds { n } => format!("{n} 轮").into(),
        Msg::NotifyTools { n } => format!("{n} 次工具调用").into(),
        Msg::NotifyApprovalTitle => "RustCode 需要批准".into(),
        Msg::NotifyApprovalBody { tool } =>
            format!("{tool} 等待批准（Y/A/N）").into(),

        // ── TUI 用户输入面板 ──
        Msg::UserInputTextPlaceholder => "输入答案...".into(),
        Msg::UserInputOwnAnswer => "输入自己的答案\u{2026}".into(),
        Msg::UserInputSubmitRow => "\u{2714} 提交".into(),
        Msg::UserInputSubmitLabel => "提交".into(),
        Msg::UserInputHintSingle { n } => format!(
            "\u{2191}\u{2193} 移动 \u{00b7} 1-{n} 选择 \u{00b7} Enter 确认 \u{00b7} Esc 取消"
        ).into(),
        Msg::UserInputHintMultiple =>
            "\u{2191}\u{2193} 移动 \u{00b7} Space 切换 \u{00b7} Enter 在提交行确认 \u{00b7} Esc 取消".into(),
        Msg::UserInputHintText =>
            "输入答案 \u{00b7} Enter 确认 \u{00b7} Esc 取消".into(),
        Msg::UserInputBatchNav { index, total } =>
            format!("问题 {index}/{total}").into(),
        Msg::UserInputReviewTitle => "提交前确认".into(),
        Msg::UserInputAnswer { answer } => format!("回答：{answer}").into(),
        Msg::UserInputUnanswered => "未回答".into(),
        Msg::UserInputSubmitAll { answered, total } =>
            format!("提交全部 ({answered}/{total} 已答)").into(),
        Msg::UserInputHintSubmit =>
            "Enter 提交 \u{00b7} PgUp/PgDn 查看 \u{00b7} Shift+Tab 返回 \u{00b7} Esc 放弃".into(),
        Msg::UserInputHintBatch =>
            "作答 \u{00b7} Tab/Shift+Tab 切换问题 \u{00b7} 到提交行 Enter 交全部 \u{00b7} Esc 放弃".into(),
        Msg::PlainAgentsStatus { finished, total, failed } =>
            format!("子代理：{finished}/{total} 完成 \u{00b7} {failed} 失败").into(),
        Msg::ModelPickerEmptyNoProviders =>
            "（未配置模型 -- 使用 /provider add）".into(),
        Msg::ModelPickerEmptyNoMatch => "（无匹配模型）".into(),
        Msg::ModelPickerEmptyQuery { query } =>
            format!("（无匹配模型 “{query}” -- 按 Backspace 清除）").into(),
        Msg::CliCrashHeader { info } => format!("\nRustCode 崩溃：{info}").into(),
        Msg::CliCrashReport =>
            "\n请将此崩溃信息（连同上方堆栈）反馈给 RustCode 安装渠道的问题追踪器。".into(),

        // ── TUI 会话恢复 / 回退 ──
        Msg::SessionResumeCancelled => "已取消加载会话".into(),
        Msg::SessionResumeCancelling => "正在取消加载会话...".into(),
        Msg::RewindNoPoints => "当前会话还没有可回退的回合。".into(),
        Msg::RewindCatalogLoadFailed { error } =>
            format!("加载回退点失败：{error}").into(),
        Msg::RewindFailed { error } => format!("回退失败：{error}").into(),
        Msg::RewindScopeConversation => "对话".into(),
        Msg::RewindScopeCode => "代码".into(),
        Msg::RewindScopeConversationAndCode => "对话和代码".into(),
        Msg::RewindSuccessMain { scope, prompt } =>
            format!("\u{21a9} 已将{scope}回退到\u{201c}{prompt}\u{201d}之前").into(),
        Msg::RewindSuccessFiles { n } => format!("（恢复 {n} 个文件）").into(),
        Msg::RewindSuccessEnd => "。".into(),
        Msg::ImageCacheDropped { n } =>
            format!("[Image #{n}] 缓存已丢失，已从消息中移除").into(),
        Msg::MoreFilesHint { hidden } =>
            format!("还有 {hidden} 个文件 (\u{2191}/\u{2193} 滚动)").into(),
        Msg::DiffPanelFilesChanged { count } => format!("{count} 个文件有变更").into(),
        Msg::DiffPanelRenamedFrom { path } => format!("重命名前：{path}").into(),
        Msg::DiffPanelNoChanges => "暂无变更".into(),
        Msg::DiffPanelTruncated => "... 差异已截断".into(),
        Msg::DiffPanelTitle => "差异".into(),
        Msg::DiffPanelEscToClose => "Esc 关闭".into(),
        Msg::FileViewerSelectFile => "选择文件".into(),
        Msg::DiffPanelBoundedSnapshot => "快照超过展示上限，部分文件或行已截断".into(),
        Msg::DiffPanelUntracked => "未跟踪文件；加入暂存区后可查看补丁".into(),
        Msg::DiffPanelPatchLimit => "补丁超过展示上限".into(),
        Msg::DiffScopeStaged => "已暂存".into(),
        Msg::DiffScopeUnstaged => "未暂存".into(),
        Msg::DiffPanelLoading => "正在读取仓库变更...".into(),
        Msg::DiffPanelFooterSelect => "↑/↓ 选择 . Enter 查看 . Esc 关闭".into(),
        Msg::DiffPanelFooterScroll => "↑/↓ 滚动 . ← 返回 . Esc 返回".into(),
        Msg::DiffPanelWorkerStopped => "差异读取线程意外停止".into(),
        Msg::DiffPanelBinary => "二进制文件，无法展示内容差异".into(),
        Msg::DiffPanelMetadataNoHunks => "文件元数据已变更，没有文本块".into(),
        Msg::DiffPanelInitialChanges => "初始变更  (仓库还没有 HEAD)".into(),
        Msg::DiffPanelUncommittedChanges => "未提交变更  (git diff HEAD)".into(),
        Msg::FileViewerOpenExternal => "打开外部文件:".into(),
        Msg::FileViewerTypeToSearch => "输入以搜索文件".into(),
        Msg::FileViewerNoMatches => "无匹配文件".into(),
        Msg::FileViewerFooter => "↑↓ 选择 . Enter 打开 . 输入 /~ 路径打开外部文件 . Esc 取消".into(),
        Msg::FileViewerReadFailed => "读取失败".into(),
        Msg::FileViewerNotRegular => "不是常规文件".into(),
        Msg::FileViewerBinary => "文件似乎是二进制（包含 NUL 字节）".into(),
        Msg::FileViewerNotUtf8 => "文件不是有效的 UTF-8 编码".into(),
        Msg::FileViewerTruncatedMarker => "已截断".into(),
        Msg::FileViewerFooterBack => "↑↓/PgUp 滚动 . Esc 返回".into(),
        Msg::FileViewerFooterClose => "↑↓/PgUp 滚动 . Esc 关闭".into(),
        Msg::RewindTargetHeader => "将对话恢复到以下提示之前...".into(),
        Msg::RewindMoreAbove { count } => format!("↑ 上方还有 {count} 个回退点").into(),
        Msg::RewindCheckpoint => "  对话检查点".into(),
        Msg::RewindNoCodeChanges => "  无代码变更".into(),
        Msg::RewindFilesChanged { count } => format!("{count} 个文件有变更").into(),
        Msg::RewindCurrent => "(当前)".into(),
        Msg::RewindMoreBelow { count } => format!("↓ 下方还有 {count} 个回退点").into(),
        Msg::RewindScopeTitle => "回退到此提示之前：".into(),
        Msg::RewindScopeMenuConversation => "仅回退对话".into(),
        Msg::RewindScopeMenuCode => "仅回退代码".into(),
        Msg::RewindScopeMenuBoth => "回退对话和代码".into(),
        Msg::RewindUnavailable => "  (不可用)".into(),
        Msg::RewindFooterTarget => "↑/↓ 选择 . Enter 继续 . Esc 取消".into(),
        Msg::RewindFooterScope => "↑/↓ 选择 . Enter 回退 . ← 返回 . Esc 取消".into(),
        Msg::RewindStartFailed => "无法开始回退".into(),
        Msg::SessionPreviewLoading => "正在加载预览...".into(),
        Msg::SessionPreviewUnavailable => "预览不可用".into(),
        Msg::PluginUninstallMarketplaceWarning { count } =>
            format!("  此操作将同时卸载该市场下的 {count} 个插件：").into(),
        Msg::ConfigPanelTitle { shown, total } => format!("配置 ({shown} / {total})").into(),
        Msg::ConfigPanelResetHint { id } => format!("再次按 Delete 恢复 {id} 的默认值").into(),
        Msg::ConfigPanelFooter => "↑↓ 选择 . Enter 修改 . Delete 恢复默认 . Esc 返回".into(),
        Msg::ConfigPanelRetryAttempts { model } =>
            format!("最大重试次数（当前模型：{model}）").into(),
        Msg::ConfigPanelPolicyImmediate => "立即".into(),
        Msg::ConfigPanelPolicyNextTurn => "下一轮".into(),
        Msg::ConfigPanelPolicyReload => "重新加载".into(),
        Msg::ConfigPanelPolicyReprepare => "重建能力".into(),
        Msg::ConfigPanelPolicyRestart => "重启后".into(),
        Msg::ProviderPanelAddAccountRow => "＋ 添加自定义 provider".into(),
        Msg::ProviderPanelRequiredMark => "(必填)".into(),
        Msg::ProviderPanelFieldName => "名称".into(),
        Msg::ProviderPanelFieldProtocol => "协议".into(),
        Msg::ProviderPanelAddAccountFormHint =>
            "Tab 下一项  ←-> 切协议  ↵ 保存  Esc 返回  （名称必填；模型到模型页加）".into(),
        Msg::ProviderPanelProtocolLocked { protocol } =>
            format!("  协议: {protocol} (锁定)").into(),
        Msg::ProviderPanelEditFormProtocolLockedHint =>
            "Tab 下一项  ↵ 保存  Esc 返回  （厂商协议已锁定）".into(),
        Msg::ProviderPanelEditAccountFormHint =>
            "Tab 下一项  ←-> 切协议  ↵ 保存  Esc 返回".into(),
        Msg::ProviderPanelProviderNotConfigured => "该 provider 尚未配置".into(),
        Msg::ProviderPanelDiscoveryTitle => "发现的模型：".into(),
        Msg::ProviderPanelDiscoveryHint =>
            "Space 切换  Enter 添加  Ctrl+A 全选  Ctrl+N 取消全选  Esc 取消".into(),
        Msg::MenuPlaceholderSearchSessions => "搜索会话...".into(),
        Msg::MenuPlaceholderSearchDirs => "搜索历史目录或输入路径...".into(),
        Msg::MenuPlaceholderFilter => "输入以筛选...".into(),

        // ── TUI Provider 重载失败 ──
        Msg::ProviderReloadFailed { error } =>
            format!("Provider 重载失败：{error}").into(),
        Msg::ProviderReloadSupersededNote =>
            "；较新的运行时世代赢得了切换，以运行时归属方状态为准".into(),
        Msg::ProviderRollbackFailed { error } =>
            format!("；配置回滚失败：{error}").into(),

        // ── daemon 实时线错误（WebUI 聊天面） ──
        Msg::LiveCompactFailed { error } => format!("压缩失败：{error}").into(),
        Msg::LiveSetModeFailed { error } => format!("切换模式失败：{error}").into(),
        Msg::LiveSubmitFailed { error } => format!("发送用户消息失败：{error}").into(),
        Msg::LiveProviderReloadFailed { error } =>
            format!("Provider 重载失败：{error}").into(),
        Msg::LiveProviderDeactivationFailed { error } =>
            format!("Provider 停用失败：{error}").into(),
        Msg::LiveSnapshotRestoreFailed { error } =>
            format!("快照恢复失败：{error}").into(),
        Msg::LiveUndoFailed { error } => format!("撤销失败：{error}").into(),
        Msg::LiveProviderNotConfigured =>
            "未配置 Provider——请在设置中添加第三方 API Key".into(),
        Msg::LiveProviderAuthRequired =>
            "Provider 认证失败——请在设置中检查或更新 API Key".into(),
        Msg::LiveProviderUnsupportedBuild =>
            "当前构建无法为受管签名网关签署请求——请使用带签名支持的发行版构建，或在设置中配置标准第三方 Provider".into(),

        // ── daemon 登录轮询错误 ──
        Msg::DaemonApiLoginSessionGone => "登录会话已不存在，请重新发起登录".into(),
        Msg::DaemonApiLoginPollUnavailable => "登录服务暂时不可用".into(),
        Msg::DaemonApiLoginExchangeFailed => "登录授权交换失败".into(),
        Msg::DaemonApiAuthPersistFailed => "登录凭证保存失败".into(),
        Msg::LiveApiSessionNamingFailed { error } => {
            format!("会话自动命名失败：{error}").into()
        }
        Msg::LiveApiRuntimeStoppedEarly => "编码运行时在回合到达终态前已停止".into(),
        Msg::LiveApiProviderRetry {
            reason,
            backoff_secs,
            attempt,
            max_attempts,
        } => format!(
            "API 错误 {reason}，{backoff_secs} 秒后重试（{attempt}/{max_attempts}）..."
        )
        .into(),
        Msg::LiveApiStreamRecovered => "已从中断的流中恢复".into(),
        Msg::LiveApiStreamTimeout {
            attempt,
            max_attempts,
        } => format!(
            "流超时；正从已保存的进度安全继续（{attempt}/{max_attempts}）"
        )
        .into(),
        Msg::LiveApiOutputLimit {
            attempt,
            max_attempts,
        } => format!("已达输出长度上限；自动继续中（{attempt}/{max_attempts}）").into(),
        Msg::LiveApiRuntimeStopped { reason } => {
            format!("编码运行时已停止：{reason}").into()
        }
        Msg::LiveApiRuntimeStoppedForcedSuffix => "（已强制停止）".into(),
        Msg::LiveApiEventSerializationFailed { error } => {
            format!("实时事件序列化失败：{error}").into()
        }
        Msg::LiveApiEventNotObject => "实时事件不是 JSON 对象".into(),
        Msg::LiveApiStreamLagged { skipped } => {
            format!("实时流滞后，已跳过 {skipped} 个事件；请重连").into()
        }
        Msg::LiveApiActiveTurnModelSwitch => "有回合正在运行；切换模型前请先停止该回合".into(),
        Msg::LiveApiGoalConditionEmpty => "目标条件为空".into(),
        Msg::PermissionReasonRequiresApproval => "需要批准".into(),
        Msg::DaemonSessionSaveEarlyStopFailed { error } => format!(
            "警告：提前停止后保存本地会话失败：{error}"
        )
        .into(),
        Msg::DaemonPanicHook { loc, msg } => {
            format!("[rustcode] 发生崩溃，位置 {loc}：{msg}").into()
        }

        Msg::StreamStalled => "按 esc 可取消".into(),
        Msg::StreamRecoveryRunning { attempt, max_attempts } => format!(
            "流响应超时，正在从已保存进度安全续接（{attempt}/{max_attempts}）..."
        )
        .into(),
        Msg::StreamRecoverySucceeded => "[+] 已从流中断处恢复".into(),
        Msg::OutputTruncationRunning { attempt, max_attempts } =>
            format!("输出达到上限，正在自动续写（{attempt}/{max_attempts}）").into(),
        Msg::OutputTruncationHeader => "输出达到上限".into(),
        Msg::OutputTruncationQuestion =>
            "模型的单次输出达到上限，自动续写仍未完成。下一步如何处理？".into(),
        Msg::OutputTruncationContinue => "继续完成".into(),
        Msg::OutputTruncationContinueDesc => "从已保留的内容继续，并改用分段写入".into(),
        Msg::OutputTruncationStop => "停止".into(),
        Msg::OutputTruncationStopDesc => "保留当前已生成的内容并结束回合".into(),
        Msg::ConhostScrollHint =>
            "提示：经典 Windows 控制台功能受限----任务执行中无法上滚查看历史，字符与吉祥物也会降级显示。\
             换用 \x1b[1;96mWindows Terminal\x1b[0m 体验更佳。"
                .into(),

        // ── rustcode-daemon 启动横幅与致命错误 ──
        Msg::DaemonIdleTimeout { minutes } => format!("空闲超时：{minutes} 分钟").into(),
        Msg::DaemonIdleTimeoutDisabled => "空闲超时：已禁用".into(),
        Msg::DaemonWarnNonLoopback { host } => format!(
            "警告：正在绑定到非回环地址 '{host}'。守护进程暴露了敏感端点（聊天、文件编辑、工具执行）。\
             请确保所在网络可信，或改用带认证的反向代理。"
        )
        .into(),
        Msg::DaemonWarnDangerousTools { env } => {
            format!("警告：{env}=1 会启用 bash 以及具备写入能力的守护进程工具。").into()
        }
        Msg::DaemonListening { addr } => {
            format!("RustCode API 服务已启动，监听地址 http://{addr}").into()
        }
        Msg::DaemonApiEndpoints => "API 端点：".into(),
        Msg::DaemonEpHealth => "健康检查".into(),
        Msg::DaemonEpProject => "获取当前工作目录".into(),
        Msg::DaemonEpCd => "切换工作目录（同 /cd 命令）".into(),
        Msg::DaemonEpProjects => "列出历史项目".into(),
        Msg::DaemonEpProjectSessions => "列出项目内的会话".into(),
        Msg::DaemonEpSessionDetail => "获取会话详情".into(),
        Msg::DaemonEpSessionDelete => "删除会话".into(),
        Msg::DaemonEpSessionRename => "重命名会话".into(),
        Msg::DaemonEpSessionRepair => "检查或修复会话".into(),
        Msg::DaemonEpSessionsAll => "列出全部会话（跨项目）".into(),
        Msg::DaemonEpSessionsSearch => "按名称搜索会话".into(),
        Msg::DaemonEpModels => "列出可用模型".into(),
        Msg::DaemonEpChat => "流式聊天响应（SSE）".into(),
        Msg::DaemonEpConfigGet => "获取脱敏后的配置".into(),
        Msg::DaemonEpConfigReload => "从磁盘重新加载配置".into(),
        Msg::DaemonEpProvidersList => "列出供应商".into(),
        Msg::DaemonEpProvidersCreate => "创建/替换供应商".into(),
        Msg::DaemonEpProvidersUpdate => "部分更新供应商".into(),
        Msg::DaemonEpProvidersDelete => "删除供应商".into(),
        Msg::DaemonEpProviderDefault => "设置默认供应商".into(),
        Msg::DaemonEpProviderThinking => "更新思考（thinking）设置".into(),
        Msg::DaemonEpSkills => "列出用户可调用的技能".into(),
        Msg::DaemonEpLoginCancel => "取消登录会话".into(),
        Msg::DaemonCdBodyHeading => "切换目录请求体：".into(),
        Msg::DaemonCdBodyHint => r#"或用 {"path": "-"} 返回上一级目录"#.into(),
        Msg::DaemonChatBodyHeading => "聊天请求体：".into(),
        Msg::DaemonFatalBind { addr, error } => {
            format!("致命错误：无法绑定到 {addr}：{error}").into()
        }
        Msg::DaemonFatalServer { error } => format!("致命错误：守护进程服务出错：{error}").into(),

        // ── rustcode-daemon HTTP API 错误消息（机器面 code 保持英文）──
        Msg::DaemonApiManagedUnavailable => "当前构建未提供托管登录服务。请在供应商设置中用你自己的 API 密钥配置第三方供应商。".into(),
        Msg::DaemonApiLoginSessionLimit => "进行中的登录会话过多；请取消或等待一个已有登录完成".into(),
        Msg::DaemonApiLoginStartFailed => "发起登录失败".into(),
        Msg::DaemonApiLoginTaskFailed => "登录任务失败".into(),
        Msg::DaemonApiInvalidLoginId => "登录会话 ID 无效".into(),
        Msg::DaemonApiLogoutFailed { error } => format!("退出登录失败：{error}").into(),
        Msg::DaemonApiCdNoPrevious => "没有可返回的上一个目录".into(),
        Msg::DaemonApiCdNotExist { path } => format!("目录不存在：{path}").into(),
        Msg::DaemonApiCdNotDir { path } => format!("不是目录：{path}").into(),
        Msg::DaemonApiCdChanged { path } => format!("已切换到 {path}").into(),
        Msg::DaemonApiSessionNotFound => "未找到会话".into(),
        Msg::DaemonApiSearchEmpty => "搜索关键字不能为空".into(),
        Msg::DaemonApiSessionActive =>
            "该会话正在使用中。请切换到或新建另一个会话后重试。".into(),
        Msg::DaemonApiDeleteNotFound => "未找到该会话。".into(),
        Msg::DaemonApiDeleteInvalidId => "会话标识符无效。".into(),
        Msg::DaemonApiDeleteFailed => "删除会话失败。详情请查看 RustCode 日志。".into(),
        Msg::DaemonApiRepairFailed => "检查或修复会话失败。详情请查看 RustCode 日志。".into(),
        Msg::DaemonApiDeleteReleaseFailed => "删除前未能释放当前会话。".into(),
        Msg::DaemonApiRenameFailed { error } => format!("重命名会话失败：{error}").into(),
        Msg::DaemonApiMetadataNotFound => "未找到该会话的元数据。".into(),
        Msg::DaemonApiProjectInvalid => "项目或会话标识符无效。".into(),
        Msg::DaemonApiSessionActiveTurn => "该会话有正在执行的回合。请先停止后再试。".into(),
        Msg::DaemonApiSessionDeleted { id } => format!("会话 {id} 已删除").into(),
        Msg::DaemonApiSessionRenamed { id, name } => format!("会话 {id} 已重命名为「{name}」").into(),
        Msg::DaemonApiChatBusySession => "该会话已有正在进行的对话操作".into(),
        Msg::DaemonApiChatBusyRequest => "该请求 ID 已有正在进行的对话操作".into(),
        Msg::DaemonApiLoginExpired => "登录会话已过期，请重新发起登录".into(),
        Msg::DaemonApiLoginCancelled => "登录会话已取消".into(),
        Msg::DaemonProvNameEmpty => "提供商名称不能为空".into(),
        Msg::DaemonProvNameDot => "提供商名称不能为 '.' 或 '..'".into(),
        Msg::DaemonProvNameInvalidChars =>
            "提供商名称不能包含 /、\\、NUL、换行、回车或制表符".into(),
        Msg::CliAcpPolicyInterventionNotice =>
            "凭据保护已拦截一次不安全的 shell 操作。请在独立终端中完成需要认证的步骤，然后让 \
             RustCode 继续、跳过被拦截的步骤或结束任务。请勿把凭据粘贴到对话中。"
                .into(),
        Msg::DaemonCmdInvalidBucket => "项目会话桶标识无效".into(),
        Msg::DaemonCmdSessionNotFound { id } => format!("未找到会话 {id}").into(),
        Msg::DaemonCmdProviderBuildPanicked { error } => {
            format!("供应商构建任务发生 panic：{error}").into()
        }
        Msg::DaemonCmdProviderBuildFailed { error } => {
            format!("供应商构建失败：{error}").into()
        }
        Msg::DaemonCmdSessionIdRequired { cmd } => format!("{cmd} 命令需要 session_id").into(),
        Msg::DaemonCmdRememberNeedsContent => "/remember 需要提供内容".into(),
        Msg::DaemonCmdForgetNeedsKeyword => "/forget 需要提供关键词".into(),
        Msg::DaemonCmdUnknown { name } => format!("未知命令：{name}").into(),
        Msg::DaemonProvDiscoveryScheme => "模型发现仅支持 http 和 https 地址".into(),
        Msg::DaemonProvDiscoveryNoCreds => "模型发现地址中不能包含凭证信息".into(),
        Msg::DaemonProvInvalidModelId => "模型选择 id 为空或包含非法字符".into(),
        Msg::DaemonProvModelCountRange => "一次请选择 1 到 100 个模型".into(),
        Msg::DaemonProvDupModelInRequest { model } => {
            format!("请求中模型 `{model}` 重复").into()
        }
        Msg::DaemonProvModelExistsInAccount { model, account } => {
            format!("账号 `{account}` 下已存在模型 `{model}`").into()
        }
        Msg::DaemonProvDupSelectionInRequest { selection } => {
            format!("请求中模型选择 `{selection}` 重复").into()
        }
        Msg::DaemonProvSelectionExists { selection } => {
            format!("模型选择 `{selection}` 已存在").into()
        }
        Msg::DaemonProvAccountNotFoundId { id } => format!("未找到供应商账号 `{id}`").into(),
        Msg::DaemonProvRuntimeReadOnly => "运行时内置的供应商账号不可修改".into(),
        Msg::DaemonProvModelEmpty => "模型名称不能为空".into(),
        Msg::DaemonProvContextWindowPositive => "context_window 必须大于零".into(),
        Msg::DaemonProvMaxTokensPositive => "max_tokens 必须大于零".into(),
        Msg::DaemonProvAccountNotFound => "未找到供应商账号".into(),
        Msg::DaemonProvModelExists { name } => format!("模型选择 {name} 已存在").into(),
        Msg::DaemonProvProviderExists { name } => format!("供应商“{name}”已存在").into(),
        Msg::DaemonProvAccountForModelNotFound { name } => {
            format!("未找到模型 {name} 对应的账号").into()
        }
        Msg::DaemonProvProviderNotFound { name } => format!("未找到供应商“{name}”").into(),
        Msg::DaemonProvManagedReserved => "该供应商名称或 base URL 为托管账号保留，而当前构建不提供托管服务。\
             请重命名供应商（或更改其 base URL），并用你自己的 api_key 完成配置。"
            .into(),
        Msg::DaemonProvDiscoveryNoListing => "该供应商协议不支持列出模型，请手动输入模型名称".into(),
        Msg::DaemonProvDiscoveryTimeout => "模型发现请求超时".into(),
        Msg::DaemonProvDiscoveryTooLarge => "模型列表响应超过 4 MiB 上限".into(),
        Msg::DaemonProvDiscoveryHttpStatus { status } => {
            format!("模型端点返回 HTTP {status}").into()
        }
        Msg::DaemonProvDiscoveryHttpStatusAuth { status } => {
            format!("模型端点返回 HTTP {status}；请检查 API 密钥").into()
        }
        Msg::DaemonProvDiscoveryUnreachable => "无法连接到模型端点".into(),
        Msg::DaemonProvDiscoveryOllamaParse => "Ollama 返回的模型列表中没有有效的 models 数组".into(),
        Msg::DaemonProvDiscoveryParse => "模型列表响应中没有有效的 data 数组，请手动输入模型名称".into(),
        Msg::DaemonProvTypeEmpty => "供应商类型不能为空".into(),
        Msg::DaemonProvThinkingBudgetMin => "thinking_budget 必须大于等于 1024".into(),
        Msg::DaemonProvVanished { name } => format!("供应商“{name}”在更新后消失了").into(),
        Msg::DaemonApiEffortUnsupported { level, target } => {
            format!("{target} 不支持 reasoning_effort 取值 {level}").into()
        }
        Msg::DaemonApiEffortUnsupportedTarget { target } => {
            format!("{target} 不支持所请求的 reasoning_effort 取值").into()
        }
        Msg::DaemonApiProviderSaveFailed { error } => {
            format!("保存供应商配置失败：{error}").into()
        }
        Msg::DaemonChatSessionStolen { session_id } => {
            format!("本轮启动时聊天会话 {session_id} 已被另一个回合占用").into()
        }
        Msg::DaemonChatOperationInactive => "聊天操作已不再处于活动状态".into(),
        Msg::DaemonApiCannotOpenFile { error } => format!("无法打开文件：{error}").into(),
        Msg::DaemonApiFileResolveFailed { error } => format!("文件解析任务失败：{error}").into(),
    }
}

/// 同 `en.rs` 的对应函数：重载在这条提示打印之前就已经做完，这里报的是结果。
fn plugin_reload_summary(loaded: usize, skipped: usize, show_details_hint: bool) -> String {
    let mut out = format!("已加载 {loaded} 个技能");
    if skipped > 0 {
        out.push_str(&format!("，跳过 {skipped} 个"));
    }
    if show_details_hint {
        out.push_str("（Ctrl+O 查看详情）");
    }
    out
}

#[cfg(test)]
mod message_text_tests {
    use super::*;
    use crate::i18n::Msg;

    #[test]
    fn zh_conhost_scroll_hint_recommends_windows_terminal() {
        let s = zh_cn(Msg::ConhostScrollHint);
        assert!(s.contains("Windows Terminal"));
        assert!(s.contains("滚"));
    }
}
