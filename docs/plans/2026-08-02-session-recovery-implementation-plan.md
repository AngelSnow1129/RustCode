# 会话持久化恢复实施计划

## 步骤 1：新增受 lease 保护的修复接缝

- 向 `rustcode-capabilities::session` 新增原生会话检查能力与
  缺失呈现的修复类型。
- 变更前先校验原生元数据与快照。
- 只在呈现 sidecar 缺失时，于活动 lease 下原子地创建它。
- 修复后重新加载严格的原生聚合。
- 测试健康、可修复、损坏、缺快照与会话忙等场景。

## 步骤 2：暴露显式的 daemon 端点

- 新增 `POST /projects/:hash/sessions/:id/repair`。
- 默认为 dry-run；执行变更需要显式传入 `apply: true`。
- 保留结构化的存储错误与会话占用冲突。
- 测试响应状态码，以及检查过程不产生任何写入。

## 步骤 3：暴露转写持久化失败

- 扩展共享的持久化状态，增加一个有界的辅助告警。
- 在 runtime 组装期间把它传给 `TranscriptHook`。
- 上报 JSONL 追加失败，并在权威的轮次终态把它们投影为 `ControllerWarning`
  事件。
- 快照不确定性保持 fail-closed，转写失败则保持非权威。

## 步骤 4：验证并记录运维恢复

- 运行受影响的 capabilities、coding 与 daemon 测试。
- 运行跨 crate 的 CLI/daemon 编译。
- 确认现有读取方保持严格，且不存在 JSONL 到快照的回退。
- 先从其既有存储中恢复被中断的旧版 code-Rewind 事务，然后才允许运维人员
  移除历史对象；绝不把该检查点保留给普通的
  轮次捕获。
- 记录：历史 Rewind 对象需要运维人员显式清理，且合成出来的
  JSONL 有意被排除在 v5.0.5 修复路径之外。
