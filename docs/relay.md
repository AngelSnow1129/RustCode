# 自建中继（frp 风格反向隧道）方案

> **状态：已实现（frpc + frps 两半均已落地），待端到端联调。**
> 由于没有可用的现成中继，本仓库自带一个最小中继 `rustcode-relay`（frps 半边），与 daemon 内置的隧道客户端（frpc 半边）配对使用。两端协议由我们自己定义，不依赖任何外部平台、账号或运营商服务。

## 1. 背景

原「App 远程访问」依赖运营商中专用的中继（移动端 App + 扫码配对），已随移动端 App 一并移除。其底层就是标准的 frp 结构（frpc + frps）。本方案把这套结构**平台中立化并自带服务端**。

## 2. 架构

```
远程客户端（浏览器 / 手机 / 其它机器）
        |  HTTP / SSE，带 Authorization: Bearer <access_key>
        v
+----------------------------+
|  rustcode-relay  (frps)    |  <- 部署在有可达地址的主机上
|   :8080 公网入口           |
|   :7000 控制通道 (WS)      |
+-------------+--------------+
              |  WebSocket 控制连接（隧道 token 鉴权）
              |  帧: [1B type][4B stream id][payload]
              v
+----------------------------+
|  daemon 隧道客户端 (frpc)   |  <- 开发机，随 /tunnel 启动
+-------------+--------------+
              |  TCP -> 127.0.0.1:<本地回源端口>
              v
    本地 webui / HTTP·SSE API（Bearer access_key 鉴权）
```

## 3. 协议规范

- **控制连接**：客户端向 `RUSTCODE_TUNNEL_RELAY`（如 `ws://host:7000/tunnel`）发起 WebSocket 连接，并用**隧道 token** 鉴权（`Authorization: Bearer <token>` 握手头，或 `?token=`；两端必须一致并在代码里注明）。
- **帧格式**（binary WebSocket 消息，整数大端）：

  | 字段 | 长度 | 说明 |
  |---|---|---|
  | type | 1 B | 1 = Open，2 = Data，3 = Close |
  | stream id | 4 B | 一条 TCP 流的标识 |
  | payload | 变长 | 仅 Data 帧携带 |

- **建流（中继侧）**：每收到一个公网入站 TCP 连接 -> 分配 stream id -> 发 `Open(id)`；随后把 socket 字节以 `Data(id, ..)` 下发，把收到的 `Data(id, ..)` 回写 socket；任一端关闭 -> `Close(id)` 并清理。
- **回源（客户端侧）**：收到 `Open(id)` -> 连本地 `127.0.0.1:<port>`；`Data(id, ..)` 写入该 TCP；从 TCP 读到的字节以 `Data(id, ..)` 回传；EOF -> `Close(id)`。
- **长连接**：纯双向字节泵，不做请求/响应假设，因此 SSE 流式响应可用。
- **健壮性**：单条流出错只记日志并清理该流，不影响整条隧道；stream id 需回收，避免无界增长。

## 4. 部署步骤

1. **起中继**（一台有可达地址的机器）：`rustcode-relay --control 0.0.0.0:7000 --public 0.0.0.0:8080 --token <强随机隧道 token>`

2. **开发机开隧道**：

   ```
   export RUSTCODE_ENABLE_TUNNEL=1
   export RUSTCODE_TUNNEL_RELAY=ws://<中继地址>:7000/tunnel
   export RUSTCODE_TUNNEL_TOKEN=<与中继相同的隧道 token>
   ```

   然后在 TUI 执行 `/tunnel`：终端会打印中继 URL、当前 `access_key` 与本地回源端口。

3. **远程访问**：访问 `http://<中继地址>:8080`，请求头带 `Authorization: Bearer <access_key>`。

## 5. 两个密钥，别混淆

| 名称 | 作用 | 配置位置 | 谁校验 |
|---|---|---|---|
| **隧道 token** | 开发机 daemon <-> 中继 的控制连接鉴权 | 中继 `--token`；daemon `RUSTCODE_TUNNEL_TOKEN` | 中继 |
| **访问密钥 access_key** | 远程客户端访问 webui / API | `config.toml` 的 `access_key`，或 `RUSTCODE_ACCESS_KEY` / `RUSTCODE_DAEMON_TOKEN` | daemon（`require_webui_token`） |

两者互不相干：隧道 token 只保证"这台开发机有权占用这条隧道"；access_key 才是对外服务的凭证。

## 6. 安全与运维

- 隧道 token 与 access_key 都必须是强随机值（建议 32 字节以上随机串）。
- **务必在 TLS 后面跑中继**：控制通道用 `wss://`，公网入口用 HTTPS；可用 Caddy / nginx 终止 TLS 后反代到 `rustcode-relay`。
- 防火墙：控制端口（7000）只对开发机开放；公网端口（8080）对需要访问的客户端开放。
- daemon 的隧道端点默认只监听回环（`127.0.0.1`），中继是它对外唯一的出口；只有 `/tunnel lan` 才会绑 `0.0.0.0`。
- 改 access_key 需重启 daemon（无热加载）。

## 7. 没有中继时的替代方案

- `/tunnel lan`：把端点绑到 `0.0.0.0`，**同一局域网内**设备用同一个 access_key 直连；无需中继、无额外依赖（已实现）。
- 自带 frp：把外部 frp 客户端指向 `/tunnel` 打印的本地回源端口。

## 8. UDP 打洞（未来模式，未实现）

后续可选：借助 STUN 风格信令 + UDP 打洞让两端点对点直连，绕过中继降低延迟。需要额外一个信令服务器，当前不在范围内。

## 9. 实现清单

- [x] 新建 `crates/rustcode-tunnel`：`protocol`（帧编解码 + 单测）、`client`（frpc）、`server`（frps）
- [x] `rustcode-relay` 二进制（`--control` / `--public` / `--token`，亦支持环境变量）
- [x] daemon 接线：`RUSTCODE_ENABLE_TUNNEL=1` 且中继 URL 与 token 齐备时，`/tunnel` 一并启动客户端
- [x] 新增依赖：`tokio-tungstenite`（仅落在本 crate，不污染 daemon）
- [ ] **端到端联调**：本机起中继 -> `/tunnel` -> 远程带 Bearer 访问（尚未实机验证）
