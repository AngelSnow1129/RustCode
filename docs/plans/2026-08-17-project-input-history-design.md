# 项目作用域的 TUI 输入历史实施计划

> **给 Claude：** 按顺序实施两个阶段，并在继续之前复查每一个检查点。

**目标：** 每个工作目录最多保留 200 条提示历史，把遗留的全局历史保留为
只读兜底，并防止并发的 TUI 窗口互相覆盖。

**架构：** TUI 仍是输入历史的唯一所有者。新历史存放在由既有原生会话哈希
派生出的 project-hash 命名空间下。遗留的 `~/.rustcode/history` 文件既不迁移
也不写入：它只在该项目填满 200 条之前用于填充空闲容量。工作目录提交后，
会先保存前台 `History` 的待写条目，再把它重新绑定到新目录。

**技术栈：** Rust、serde JSONL、fs2 文件锁、同目录原子替换、既有的 `SessionManager::project_hash`。

---

## 持久化布局与兼容性

```text
$RUSTCODE_HOME/history                    # legacy, read-only
$RUSTCODE_HOME/history-v2/<hash>/entries.jsonl
$RUSTCODE_HOME/history-v2/<hash>/write.lock
$RUSTCODE_HOME/history-v2/<hash>/images/
```

- 项目条目是最新权威数据，磁盘上限为 200 条。
- 若某项目只有 N < 200 条，则视图会把最新的 `200 - N` 条非重复遗留条目前置。
- 一旦项目达到 200 条，就不再读取遗留历史。
- 新的写入与图片 GC 只影响当前活跃的项目命名空间。
- 合并时使用严格的条目相等判定；项目条目胜过完全相同的遗留条目。

### 阶段 1：项目历史存储与并发

**文件：**
- 修改：`crates/rustcode-tuix/src/platform.rs`
- 修改：`crates/rustcode-tuix/src/input/history.rs`

1. 增加测试，覆盖项目路径稳定性、项目/遗留混合、200 条截断、
   只写项目历史，以及两个写入方合并且不丢更新。
2. 增加一个由 `SessionManager::project_hash(cwd)` 派生的项目历史路径集合。
3. 只把自上次加载/保存之后推入的条目记为待写。
4. 保存时先获取项目锁，重载当前项目 JSONL，合并待写条目，截断到 200 条，
   然后原子替换文件。
5. 保持遗留与项目的图片缓存相互独立；只对项目缓存做 GC。
6. 运行聚焦的历史与 platform 测试。

### 阶段 2：运行时工作目录重绑定

**文件：**
- 修改：`crates/rustcode-tuix/src/lib.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs`

1. 增加测试，证明启动时选中当前 cwd，且一次已提交的 cwd/会话切换会替换
   历史视图。
2. 以启动工作目录初始化历史。
3. 增加一个 TUI 持有的重绑定辅助函数：保存旧项目的待写条目，把失败的保存
   留在内存中重试，加载或复用目标项目，然后发布新的工作目录投影。
4. 让 `/cd`、跨项目 `/resume` 与前/后台会话替换都走同一个已提交投影辅助函数。
5. 重绑定时重置输入历史的导航/搜索状态，使索引不会指向旧项目。
6. 运行聚焦的 TUI 测试，然后运行受影响 crate 的测试套件。

## 失败语义

- 历史保存/加载失败绝不能让运行时的 cwd 或会话切换失败。
- 保存失败时，把旧项目的 `History`（含待写行）留在 TUI 持有的延迟队列中。
   之后的 cwd 变更与关机会重试；切回时复用该内存中的历史，而不是加载
   过期的磁盘视图。
- 加载失败仅作诊断用途，历史退化为空的项目视图，并尽可能叠加可读的遗留历史。
- 遗留文件绝不改名、删除、截断或追加。
- 锁竞争只在很小的本地合并/写入临界区内等待；持锁期间不做任何运行时/网络工作。
- 原子替换可避免崩溃后留下半个 JSONL。

## 验证

```bash
env -u RUSTCODE_HOME cargo test -p rustcode-tuix input::history --lib
env -u RUSTCODE_HOME cargo test -p rustcode-tuix working_dir_projection --lib
env -u RUSTCODE_HOME cargo test -p rustcode-tuix --lib
```

Windows 特有的加锁与替换行为还应由 CI 或 Windows 构建机额外验证。
