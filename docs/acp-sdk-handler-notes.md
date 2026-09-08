# ACP SDK handler/transport 笔记(2026-06-29)

在把 `rustcode acp` agent 接入 `agent-client-protocol` 时记录的历史
spike 笔记。仅作参考 —— 以代码为准。

## Handler 闭包的写法

- `responder.respond(response)`:
  `fn respond(self, response: T) -> Result<(), agent_client_protocol::Error>`,
  其中对 request handler 而言 `T = Req::Response`。

- `on_receive_request!()` 展开为
  `|f: &mut _, req, responder, cx| Box::pin(f(req, responder, cx))`
  (在返回类型标记法稳定之前必须这样写;该闭包需作为最后一个参数传入)。

- `on_receive_dispatch!()` 展开为
  `|f: &mut _, dispatch, cx| Box::pin(f(dispatch, cx))`。

- `util::internal_error(message)`:
  `fn internal_error(message: impl ToString) -> agent_client_protocol::Error`
  (内部调用 `Error::internal_error().data(message.to_string())`)。

## Dispatch 循环的并发性

按设计为单一 async task、非并发。crate 源码注释的大意是:
"连接上的消息处理跑在单个 async task 上。当某个 handler 正在
执行时,其他消息都无法被处理。"handler 会一直阻塞该循环直到返回;
需要并发执行时请使用 `cx.spawn()`。

## 非 stdio 的内存 transport

`agent_client_protocol::Channel` —— 调用 `Channel::duplex()` 可得到
一对 `(Channel, Channel)`。每个 `Channel` 对任意 `Role` 都实现了
`ConnectTo<R>`,因此无需子进程即可用于进程内集成测试。
它由 crate root 重新导出(来自 `jsonrpc`)。