# 自建中继（frp 风格反向隧道）使用方案

> **状态：已实现，并通过端到端集成测试验证**（`cargo test -p rustcode-tunnel`：11 单测 + 4 个 e2e 用例全绿）。
> 没有现成中继也能用：仓库自带最小中继 `rustcode-relay`（frps 半边），与 daemon 内置的隧道客户端（frpc 半边）配对。两端协议由我们自己定义，不依赖任何外部平台、账号或运营商服务 —— **用户可以自己搭建并启动**。

## 1. 架构

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

## 2. 协议规范

- **控制连接**：客户端连 `RUSTCODE_TUNNEL_RELAY`（如 `ws://host:7000/tunnel`），token 以查询参数 `?token=` 传递；不匹配则中继在握手阶段直接返回 **401**。
- **帧格式**（binary WebSocket 消息，整数大端）：

  | 字段 | 长度 | 说明 |
  |---|---|---|
  | type | 1 B | 1 = Open，2 = Data，3 = Close |
  | stream id | 4 B | 一条 TCP 流的标识 |
  | payload | 变长 | 仅 Data 帧携带 |

  利用 WebSocket 的消息边界，一条消息即一帧，因此不需要额外长度字段。
- **纯双向字节泵**：不做请求/响应假设，所以 SSE 之类的长连接可用。
- **健壮性**：单条流出错只清理该流；未知流的迟到数据会被回 `Close` 清理。

## 3. 构建二进制（重要）

`rustcode-tunnel` **不在工作区 `default-members` 里**，所以直接 `cargo build --release` **不会**产出这个二进制。必须显式指定：

```bash
cargo build --release -p rustcode-tunnel --bin rustcode-relay
# 产物：target/release/rustcode-relay
```

## 4. 快速开始（三步）

**第 1 步 —— 中继主机**（需要有可达地址）：

```bash
rustcode-relay --control 0.0.0.0:7000 --public 0.0.0.0:8080 --token <强随机隧道 token>
# 也可用环境变量：RUSTCODE_RELAY_CONTROL / RUSTCODE_RELAY_PUBLIC / RUSTCODE_RELAY_TOKEN
# rustcode-relay --help 查看全部参数
```

**第 2 步 —— 开发机**：

```bash
export RUSTCODE_ENABLE_TUNNEL=1
export RUSTCODE_TUNNEL_RELAY=ws://<中继地址>:7000/tunnel
export RUSTCODE_TUNNEL_TOKEN=<与中继相同的 token>
```

然后在 TUI 执行 `/tunnel`。终端会打印中继 URL、当前 `access_key` 和本地回源端口；若开关与 URL/token 齐备，会**自动拉起 frpc 客户端**连上中继。

**第 3 步 —— 远程访问**：

```bash
curl -H "Authorization: Bearer <access_key>" http://<中继地址>:8080/
```

## 5. 作为服务常驻

**systemd**（`/etc/systemd/system/rustcode-relay.service`）：

```ini
[Unit]
Description=RustCode tunnel relay (frps)
After=network.target

[Service]
ExecStart=/usr/local/bin/rustcode-relay --control 0.0.0.0:7000 --public 0.0.0.0:8080
EnvironmentFile=/etc/rustcode/relay.env   # RUSTCODE_RELAY_TOKEN=<强随机>
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
```

**Docker Compose**：

```yaml
services:
  relay:
    image: rustcode-relay   # 自行构建
    command: ["--control","0.0.0.0:7000","--public","0.0.0.0:8080"]
    environment:
      RUSTCODE_RELAY_TOKEN: ${RUSTCODE_RELAY_TOKEN}
    ports:
      - "7000:7000"   # 控制通道：只应对开发机开放
      - "8080:8080"   # 公网入口
    restart: always
```

## 6. TLS（生产必做）

中继本身是明文 WebSocket / TCP，**生产环境务必放在 TLS 后面**（Caddy / nginx 终止 TLS）：

```
# Caddyfile
relay.example.com {
    # 远程客户端访问（HTTPS -> 中继公网端口）
    reverse_proxy 127.0.0.1:8080
}

ws.example.com {
    # 开发机的控制通道（wss -> 中继控制端口）
    reverse_proxy 127.0.0.1:7000
}
```

之后开发机改用 `RUSTCODE_TUNNEL_RELAY=wss://ws.example.com/tunnel`。

## 7. 防火墙

| 端口 | 用途 | 应对谁开放 |
|---|---|---|
| 7000（控制） | daemon ↔ 中继 | **仅开发机** |
| 8080（公网） | 远程客户端访问 | 需要访问的客户端 |

## 8. 两个密钥，别混淆

| 名称 | 作用 | 配置位置 | 谁校验 |
|---|---|---|---|
| **隧道 token** | 开发机 daemon ↔ 中继的控制连接鉴权 | 中继 `--token` / `RUSTCODE_RELAY_TOKEN`；开发机 `RUSTCODE_TUNNEL_TOKEN` | 中继 |
| **访问密钥 access_key** | 远程客户端访问 webui / API | `config.toml` 的 `access_key`，或 `RUSTCODE_ACCESS_KEY` / `RUSTCODE_DAEMON_TOKEN` | daemon（`require_webui_token`） |

两者互不相干：隧道 token 只保证"这台开发机有权占用这条隧道"；access_key 才是对外服务的凭证。

## 9. 故障排查

| 现象 / 日志 | 原因 | 处理 |
|---|---|---|
| 中继日志 `rejected control handshake (bad token)` | 两端 token 不一致 | 核对 `RUSTCODE_RELAY_TOKEN` 与 `RUSTCODE_TUNNEL_TOKEN` |
| 客户端日志 `local endpoint unreachable` | daemon 的隧道端点没起来 | 先在 TUI 执行 `/tunnel` 启动本地端点 |
| 客户端日志 `data for an unknown stream, closing it` | 某条流已被对端关闭，属正常清理 | 通常无需处理；频繁出现再查 |
| daemon 告警 `RUSTCODE_ENABLE_TUNNEL is set but the relay URL or token is missing` | 只设了开关，缺 URL 或 token | 配齐三个环境变量 |
| 远程访问返回 **401** | 没带或带错 `access_key` | 请求头加 `Authorization: Bearer <access_key>` |
| 连接挂住无响应 | 控制端口不通 / 中继不可达 | 检查防火墙 7000、`RUSTCODE_TUNNEL_RELAY` 地址 |
| 改了 access_key 不生效 | 密钥是启动时载入的，无热加载 | 重启 daemon |

## 10. 安全清单

- [ ] 隧道 token 与 access_key 都是强随机值（建议 32 字节以上随机串）
- [ ] 生产环境已启用 TLS（公网 HTTPS + 控制通道 `wss://`）
- [ ] 防火墙端口分离，控制端口不对外暴露
- [ ] daemon 隧道端点保持在回环（`127.0.0.1`）；只有 `/tunnel lan` 才绑 `0.0.0.0`
- [ ] 定期轮换 token 与 access_key（轮换需重启）

## 11. 没有中继时的替代方案

- `/tunnel lan`：把端点绑到 `0.0.0.0`，**同一局域网内**设备用同一个 access_key 直连；无需中继、无额外依赖（已实现）。
- 自带 frp：把外部 frp 客户端指向 `/tunnel` 打印的本地回源端口。

## 12. UDP 打洞（未来模式，未实现）

后续可选：借助 STUN 风格信令 + UDP 打洞让两端点对点直连，绕过中继降低延迟。需要额外一个信令服务器，当前不在范围内。

## 13. 实现清单

- [x] 新建 `crates/rustcode-tunnel`：`protocol`（帧编解码 + 单测）、`client`（frpc）、`server`（frps）
- [x] `rustcode-relay` 二进制（`--control` / `--public` / `--token`，亦支持环境变量）
- [x] daemon 接线：开关 + URL + token 齐备时自动拉起 frpc，缺配置仅告警
- [x] 依赖 `tokio-tungstenite`（仅落在本 crate，不污染 daemon）
- [x] **端到端验证**：`tests/e2e_tunnel.rs` 4 个用例（转发、错误 token 拒绝、并发多流、顺序流），`cargo test -p rustcode-tunnel` 15 passed / 0 failed
