# Project-scope 记忆目录可配置实施计划

> **面向 agentic worker：** 必备子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实施本计划。步骤使用复选框（`- [ ]`）语法进行跟踪。

**Goal:** 让 project-scope 记忆文件目录可经环境变量 `RUSTCODE_PROJECT_MEMORY_DIR` 覆盖（默认 `.rustcode` 不变），消除与 global scope（`RUSTCODE_HOME`）的不对称。

**Architecture:** 两份字节兼容的 `MemoryStore`（capabilities 端口 + config 原版）各自把 `project()` 的硬编码 `.rustcode` 拆成"纯函数 `project_memory_path` + 读 env"两层。读/写/prompt 注入都经各自 `project()`，故每份改一处即覆盖该 crate 全部路径，无调用点改动。

**Tech Stack:** Rust；`rustcode-capabilities`、`rustcode-config`；各自 `#[cfg(test)] mod tests`。

## 全局约束

- 默认（env 未设/空）行为**与现状逐字节一致**：`<project_root>/.rustcode/memory.md`。
- **两份 store 都要改，且逐字镜像**：`rustcode-capabilities/src/memory/store.rs` 与 `rustcode-config/src/config/memory.rs`（保持它们既有的 verbatim-port 关系）。
- env 变量名 **`RUSTCODE_PROJECT_MEMORY_DIR`**（逐字，两份一致）。值语义：相对 → 挂 `project_root`；绝对 → 直接用（`Path::join` 语义）；空 → 回退默认。
- **不做 env 集成测试**（进程全局 env + crate 内并行 → 会泄漏进 `remember_writes_project_entry` 致 flaky）；只测纯函数。`project()` 的 env 读取是 2 行 glue。
- 不新增缓存/装配期参数/迁移逻辑（MVP）。
- 分支 `feat/project-memory-dir`（基于 main）。提交用显式 pathspec。
- 设计源文档：`docs/superpowers/specs/2026-07-31-project-memory-dir-override-design.md`。

---

### 任务 1：capabilities `MemoryStore::project` 读 env

**文件：**
- 修改：`crates/rustcode-capabilities/src/memory/store.rs`（`project()` 约 L29-31；`mod tests` 约 L172+）

**接口：**
- 消费： 既有 `MemoryStore::new(PathBuf) -> Self`。
- 产出： `fn project_memory_path(project_root: &Path, override_dir: Option<&str>) -> PathBuf`（私有纯函数）；`pub fn project(project_root: &Path) -> Self` 行为变更（读 env）。

- [ ] **步骤 1：写失败测试**（`mod tests` 内新增）

```rust
    #[test]
    fn project_memory_path_resolves_override() {
        use std::path::Path;
        let root = Path::new("/proj");
        // Default (unset/empty) is byte-identical to today's hardcoded path.
        assert_eq!(
            super::project_memory_path(root, None),
            Path::new("/proj/.rustcode/memory.md")
        );
        assert_eq!(
            super::project_memory_path(root, Some("")),
            Path::new("/proj/.rustcode/memory.md")
        );
        // Relative override nests under project_root.
        assert_eq!(
            super::project_memory_path(root, Some(".myapp")),
            Path::new("/proj/.myapp/memory.md")
        );
        // Absolute override is used as-is (Path::join replaces the base).
        assert_eq!(
            super::project_memory_path(root, Some("/opt/brand/mem")),
            Path::new("/opt/brand/mem/memory.md")
        );
    }
```

- [ ] **步骤 2：跑测试确认失败**

运行：`cargo test -p rustcode-capabilities --lib project_memory_path_resolves_override`
预期：FAIL — `project_memory_path` 未定义（编译错误）。

- [ ] **步骤 3：实现纯函数 + 改 `project()`**

把 store.rs 的：

```rust
    pub fn project(project_root: &Path) -> Self {
        Self::new(project_root.join(".rustcode").join("memory.md"))
    }
```

改成：

```rust
    /// Resolve the project-scope memory file. `override_dir` = the value of
    /// `RUSTCODE_PROJECT_MEMORY_DIR` (None/empty → default ".rustcode"). A relative value
    /// nests under `project_root`; an absolute value is used as-is (std `Path::join`
    /// semantics). `memory.md` is appended in either case.
    fn project_memory_path(project_root: &Path, override_dir: Option<&str>) -> PathBuf {
        let dir = override_dir.filter(|s| !s.is_empty()).unwrap_or(".rustcode");
        project_root.join(dir).join("memory.md")
    }

    /// Project-scope store. Honors `RUSTCODE_PROJECT_MEMORY_DIR` (host rebrand parity with
    /// the global scope's `RUSTCODE_HOME`); default `.rustcode` is unchanged.
    pub fn project(project_root: &Path) -> Self {
        let override_dir = std::env::var("RUSTCODE_PROJECT_MEMORY_DIR").ok();
        Self::new(Self::project_memory_path(project_root, override_dir.as_deref()))
    }
```

注意：`project_memory_path` 作为 `impl MemoryStore` 的关联函数（`Self::project_memory_path`）；测试里以 `super::MemoryStore::project_memory_path` 调用——若测试写的是 `super::project_memory_path`，改成关联函数路径或把纯函数放模块级。**本计划采用模块级私有 `fn project_memory_path`（非关联函数）**，故测试用 `super::project_memory_path` 正确；`project()` 内改调 `project_memory_path(...)`（不加 `Self::`）。`PathBuf` 已在文件顶部 `use std::path::{Path, PathBuf}` 导入。

- [ ] **步骤 4：跑测试确认通过**

运行：`cargo test -p rustcode-capabilities --lib project_memory_path_resolves_override`
预期：测试通过（PASS）。

- [ ] **步骤 5：跑 crate 全量确认无回归**

运行：`cargo test -p rustcode-capabilities`
预期：全绿；尤其 `tools/memory.rs::remember_writes_project_entry`（env 未设 → 默认 `.rustcode`）与 `memory/hook.rs` 测试仍通过。

- [ ] **步骤 6：提交**

```bash
git add crates/rustcode-capabilities/src/memory/store.rs
git commit -m "feat(memory): honor RUSTCODE_PROJECT_MEMORY_DIR in capabilities store" -- crates/rustcode-capabilities/src/memory/store.rs
```

---

### 任务 2：config `MemoryStore::project` 读 env(逐字镜像)+ 文档

**文件：**
- 修改：`crates/rustcode-config/src/config/memory.rs`（`project()` 约 L22-24；`mod tests` 约 L155+）
- 修改（文档，尽力而为）：`site/docs/en/configuration.html` 与 `site/docs/zh/configuration.html`（在 `RUSTCODE_HOME` 说明旁补一条）

**接口：**
- 消费： 既有 `MemoryStore::new(PathBuf) -> Self`。
- 产出： 与 任务 1 同签名的模块级 `fn project_memory_path`；`pub fn project` 行为变更。二者与 任务 1 **逐字一致**（保持字节兼容端口关系）。

- [ ] **步骤 1：写失败测试**（`config/memory.rs` 的 `mod tests` 内新增，与 任务 1 同）

```rust
    #[test]
    fn project_memory_path_resolves_override() {
        use std::path::Path;
        let root = Path::new("/proj");
        assert_eq!(
            super::project_memory_path(root, None),
            Path::new("/proj/.rustcode/memory.md")
        );
        assert_eq!(
            super::project_memory_path(root, Some("")),
            Path::new("/proj/.rustcode/memory.md")
        );
        assert_eq!(
            super::project_memory_path(root, Some(".myapp")),
            Path::new("/proj/.myapp/memory.md")
        );
        assert_eq!(
            super::project_memory_path(root, Some("/opt/brand/mem")),
            Path::new("/opt/brand/mem/memory.md")
        );
    }
```

- [ ] **步骤 2：跑测试确认失败**

运行：`cargo test -p rustcode-config --lib project_memory_path_resolves_override`
预期：FAIL — `project_memory_path` 未定义。

- [ ] **步骤 3：实现(逐字镜像 任务 1)**

把 config/memory.rs 的：

```rust
    pub fn project(project_root: &Path) -> Self {
        Self::new(project_root.join(".rustcode").join("memory.md"))
    }
```

改成与 任务 1 **逐字相同**的 `project_memory_path`（模块级私有 `fn`）+ `project()`（读 `RUSTCODE_PROJECT_MEMORY_DIR`）。`PathBuf` 已在 config/memory.rs 顶部 `use std::path::{Path, PathBuf}` 导入。

- [ ] **步骤 4：跑测试确认通过**

运行：`cargo test -p rustcode-config --lib project_memory_path_resolves_override`
预期：测试通过（PASS）。

- [ ] **步骤 5：补文档(best-effort)**

在 `site/docs/en/configuration.html` 与 `site/docs/zh/configuration.html` 里，找到描述 `RUSTCODE_HOME` 的环境变量小节，紧随其后补一条：
- en: `<code>RUSTCODE_PROJECT_MEMORY_DIR</code> — overrides the per-project memory directory (default <code>.rustcode</code>); relative values nest under the working directory, absolute paths are used as-is. Only affects the project-scope memory file.`
- zh: `<code>RUSTCODE_PROJECT_MEMORY_DIR</code> — 覆盖每项目记忆目录（默认 <code>.rustcode</code>）；相对值挂当前工作目录，绝对路径直接使用。仅影响 project-scope 记忆文件位置。`

若该页结构不便插入（找不到清晰的 env 小节），跳过文档改动并在提交信息/报告中说明，不阻塞 Task。

- [ ] **步骤 6：跑 crate 全量确认无回归**

运行：`cargo test -p rustcode-config`
预期：全绿（含 config/memory.rs 既有测试）。

- [ ] **步骤 7：提交**

```bash
git status --short   # 确认只暂存 config/memory.rs (+ 可能的两个 configuration.html)
git add crates/rustcode-config/src/config/memory.rs site/docs/en/configuration.html site/docs/zh/configuration.html
git commit -m "feat(memory): honor RUSTCODE_PROJECT_MEMORY_DIR in config store + docs"
```
（若跳过文档，则只 `git add` config/memory.rs。）

---

## 自审

**1. 规格覆盖：**
- env 覆盖 `RUSTCODE_PROJECT_MEMORY_DIR` + 默认不变 + 值语义（相对/绝对/空）→ 任务 1/2 步骤 3 + 纯函数测试。
- **两份 store 都改**（capabilities + config，daemon 路径）→ 任务 1（capabilities）+ 任务 2（config）。
- prompt 注入 hook 走 `project()` 自动覆盖 → 无需独立任务；任务 1 步骤 5 断言 hook 测试仍绿。
- 只测纯函数、不做 env 集成测试（flakiness）→ Global Constraints + 两任务测试步。
- 文档 → 任务 2 步骤 5。
- 不迁移/不缓存/不装配期 → 全局约束。 [+] 无缺口。

**2. 占位符扫描：** 无 TBD/TODO；代码步骤含完整代码；文档步骤给了逐字文案。 [x]

**3. 类型一致性：** 两任务 `project_memory_path(&Path, Option<&str>) -> PathBuf` 签名逐字一致；`project()` 读同一 env 名 `RUSTCODE_PROJECT_MEMORY_DIR`；测试断言路径与实现一致（`.rustcode`/`.myapp`/`/opt/brand/mem` + `memory.md`）。 [x]
