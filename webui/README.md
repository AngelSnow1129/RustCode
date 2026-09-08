# RustCode webui（浏览器界面）

RustCode 的本地浏览器界面（Preact + Vite + Tailwind），由 `rustcode-daemon` 的
HTTP 服务提供。可以在 TUI 中输入 `/webui`，或在命令行执行 `rustcode webui` 启动；
两种方式都只会在浏览器中打开一个仅监听回环地址的页面。

## 前端开发

```bash
cd webui
npm install
npm run dev          # vite dev server on http://localhost:5173
```

若要在已运行的后端上获得热重载，请设置 `RUSTCODE_WEBUI_DEV`，让后端把页面请求
重定向到 vite 开发服务器，而不是提供内嵌的产物包：

```bash
RUSTCODE_WEBUI_DEV=http://localhost:5173 rustcode webui
# (or run the daemon directly)
RUSTCODE_WEBUI_DEV=http://localhost:5173 cargo run -p rustcode-daemon -- --port 13456
```

API 请求仍然直接打到后端，只有静态页面会被重定向，因此你既能保留实时 HMR，又能与真实的后端通信。

## 发布构建

```bash
cd webui
npm run build        # outputs webui/dist/
cargo build          # re-embeds webui/dist/ into the binary
```

`webui/dist/` 中已编译的产物会提交到仓库，并在构建时通过 `rust-embed` 内嵌进二进制文件
（见 `crates/rustcode-daemon/src/webui.rs`）。修改前端代码后，请运行
`npm run build` 并提交更新后的 `dist/`，使内嵌的产物包保持同步。

## 构建产物

`webui/dist/` 是**有意提交**到仓库的。这样在任何环境下执行 `cargo build` 都能产出可用的二进制，
而无需 Node.js / npm 工具链 —— 内嵌的界面始终来自已提交的快照。

以下发布脚本（`scripts/release.sh`、`scripts/release-daemon.sh`、
`scripts/build-official.sh`、`scripts/linux-release-linux.sh`、
`scripts/macos-release-linux.sh`、`scripts/macos-release-windows.sh`）都会先通过
`npm ci && npm run build` 重建 `webui/dist/`（使发行二进制总是内嵌最新前端），再调用 `cargo
build`, so release binaries always embed the latest frontend. If `npm`在构建环境中不可用，脚本会回退到已提交的
`webui/dist/` 并给出警告，而不是直接失败。
