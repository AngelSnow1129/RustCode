---
kind: test-report
id: TEST-002
from: test-engineer
to: [project-manager, code-implementer]
feature: 2026-09-09-omo-skills-import
status: done
decision: proceed
requires: [DESIGN-001, T-04, T-05]
files_owned:
  - crates/rustcode-capabilities/tests/tmp_omo_skills_load2.rs  # 临时夹具，已删除，未入库
  - .codebuddy/artifacts/2026-09-09-omo-skills-import/05-test-report-batch2.md
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-09
---

# 05 · 测试报告（第二批）：新增 2 个运行时 skills 的真实加载与 8 个技能回归

## 0. 结论速览

| 验证项 | 结论 | exit code | 证据强度 |
| --- | --- | --- | --- |
| V1 真实加载（核心，8 个技能） | **PASS** | 0 | 强 —— 走公开 API `load_dir` + `reload`，8/8 用例绿 |
| V2 既有 skills 回归 | **PASS** | 0 | 强 —— 49/49 绿（`--features skills`），用例集合与首批完全一致 |
| V3 工具名真实性（2 个新技能） | **PASS（16 项 / 10 个去重名全部注册）** | 0 | 强 —— 逐名比对 `Tool::name()` 实现 |
| V4 仓库卫生与门禁 | **PASS** | 0 | 强 —— `git status` 仅 2 个未跟踪交接件；`check-zh-docs.py gate` PASS |
| V5 config.toml LSP 键名独立核实 | **4/4 属实** | 0 | 强 —— 逐条读源码并给出 `文件:行号` |

**用例总数：8（临时加载夹具）+ 49（既有回归）+ 16（工具名抽查）+ 2（门禁）+ 4（LSP 键名核实）；最终失败 0。**
**决策：`proceed`。** 附带 2 条**非阻塞**观察项（§6.2），建议编排者知悉，不阻塞发布。

执行路径声明：
- V1 只走公开 API：`SkillRegistry::load_dir(&PathBuf::from("/root/.rustcode/skills"), Some("skills"))` 与 `SkillRegistry::reload(working_dir)`。`parse_skill_dir` 是 `pub(crate)`，集成测试不可达，**未使用**。
- `skills` 是 opt-in cargo feature（`crates/rustcode-capabilities/Cargo.toml:193` `skills = []`，不在 `default = ["provider","tools"]` 内），所有命令显式带 `--features skills`。
- 全程**未修改** 8 个 `SKILL.md`、未修改生产源码、未执行 `git add` / `git commit`。

---

## 1. 测试策略

| 层次 | 划分 | 理由 |
| --- | --- | --- |
| 临时集成夹具 `tests/tmp_omo_skills_load2.rs` | 集成 | 直接对真实磁盘的 `/root/.rustcode/skills` 跑公开加载入口，是「能否被真正加载」的唯一可信证据；静态文本比对无法覆盖 frontmatter 解析、命名归一化、命名空间前缀、目录式技能发现等运行时行为。 |
| 既有单元回归 `--lib skills` | 单元 / 回归 | 本批零生产代码改动，需证明 skills 模块既有 49 个用例未被破坏，且用例集合与首批同源。 |
| 门禁脚本 `scripts/check-zh-docs.py gate` | 门禁回归 | 复核仓库中文文档门禁。 |
| 源码 grep 抽查（工具名 + LSP schema） | 静态核对 | `allowed-tools` 只是元数据（L1 明确不强制，`skills/skill.rs:17-19`），无法在加载路径上断言；LSP 配置键名属「文档声称的事实」，只能回到源码核对。 |

失败路径设计：本批被测对象是**同步、纯函数式**的文件扫描加载器，无 cancel / retry / 并发路径；与之对应的可观测失败面是「未知名必须拒绝」「目录缺失必须跳过」「解析失败静默跳过」，分别由 `t5`、既有单测 `missing_dir_is_skipped`、`load_dir` 的 skip 语义覆盖（`t5` 为本批新增，后两者属既有 49 个用例，见 §4.1）。
**未采用任何「仅断言不 panic」的用例**；所有断言均为可观测值（key 集合、description / template 字符数、`expand` 输出首尾内容、`reload` 后的索引集合）。

---

## 2. 用例清单

### 2.1 AC → 用例映射（无漏测）

| 验收点（来自任务书） | 用例 id | 覆盖场景 | 结果 |
| --- | --- | --- | --- |
| 先 dump 全部实际 key，不猜名字 | `t0` | 打印 8 个真实 key + desc 预览后再断言 | PASS |
| 断言总数 == 8 | `t0` / `t1` / `t7` | `len() == 8` 且无多余项、无缺失项 | PASS |
| 8 个技能 description 与 template 均非空 | `t2` | 逐个断言 `trim().chars().count() > 0`，并打印长度 | PASS |
| 新增 2 个各自 `expand("rust","sess-test")` 非空且含参数 | `t4` | 尾部 `ARGUMENTS: rust` / 长度增长 / 正文开头保留 / 无残留 token | PASS |
| `user_invocable()` 列出 8 个 | `t3` | 集合比对 + `render_catalog()` 含全部 8 个全名 | PASS |
| driver 真实入口 `reload()`（HOME=/root）仍 8 个 | `t7` | `RUSTCODE_HOME` 未设置、`HOME=/root` 下真实解析 | PASS |
| 新增 2 个不得破坏已有 6 个（增量回归） | `t1` | 首批 6 个逐个 `get()` 仍命中 | PASS |
| 既有回归 49/49 | §3.2 | `--lib skills` | PASS |
| 工具名真实性 | §3.3 | 16 项逐名比对 | PASS |
| 仓库卫生 + 门禁 | §3.4 | `git status` + `gate` | PASS |
| LSP 键名独立核实 | §3.5 | 4 条逐条回源 | 4/4 属实 |

### 2.2 用例总表

| 用例 id | 类型 | 覆盖场景（AC 映射） | 文件路径 | 结果 |
| --- | --- | --- | --- | --- |
| `t0_dump_actual_registry_keys` | 集成（基线 dump） | 打印真实 key，避免猜名字；断言 key 集合 == 预期 8 个 | `crates/rustcode-capabilities/tests/tmp_omo_skills_load2.rs`（**已删除**） | PASS |
| `t1_all_eight_skills_are_indexed_under_skills_namespace` | 集成（正常 + 增量回归） | 8 个全名索引；首批 6 个仍在；新增 2 个已进 | 同上 | PASS |
| `t2_every_skill_has_non_empty_description_and_template` | 集成（边界） | description / template 非空 + `source_path.is_file()` | 同上 | PASS |
| `t3_all_eight_are_user_invocable` | 集成 | `user_invocable()` 集合 + `render_catalog()` 全名 | 同上 | PASS |
| `t4_expand_injects_arguments_for_the_two_new_skills` | 集成（正常路径） | 2 个新技能 `expand` 参数注入 | 同上 | PASS |
| `t5_bare_name_resolves_and_unknown_is_rejected` | 集成（失败路径） | 裸名解析 / 大小写不敏感 / 未知名必须返回 `None` | 同上 | PASS |
| `t6_standard_and_runtime_dirs_include_the_real_skill_root` | 集成 | 加载点未被绕过 | 同上 | PASS |
| `t7_reload_through_the_real_runtime_path_loads_all_eight` | 集成（driver 真实入口） | `HOME=/root`、`RUSTCODE_HOME` 未设置下 `reload()` | 同上 | PASS |
| `skills::*` 49 个既有用例 | 单元 / 回归 | 既有行为未被破坏 | `crates/rustcode-capabilities/src/skills/**` | PASS (49/49) |
| 16 × 工具名存在性 | 静态抽查 | `allowed-tools` 真实性 | `crates/rustcode-capabilities/src/tools/**` | PASS (16/16) |
| `check-zh-docs.py gate` | 门禁 | 全仓中文文档门禁 | `scripts/check-zh-docs.py` | PASS |
| `git status --short` | 卫生 | 无 `.rs` 残留、无生产改动 | — | PASS |
| LSP schema 4 条 | 静态核实 | doc-writer 声称事实的独立复核 | `crates/rustcode-config/src/config/mod.rs` 等 | 4/4 属实 |

---

## 3. 执行证据

### 3.1 验证 1（核心）：8 个技能的加载与回归

命令：

```
cd /workspace/RustCode && HOME=/root cargo test -p rustcode-capabilities --features skills --test tmp_omo_skills_load2 -- --nocapture
```

exit code：`0`

原始输出（编译段仅保留与首批一致的既有 `parse_scutil_proxy` dead_code 告警；`key =` / `desc[..80]` 的 8 行 dump 见 §3.1.1）：

```
warning: function `parse_scutil_proxy` is never used
  --> crates/rustcode-config/src/system_proxy.rs:80:15
   |
80 | pub(crate) fn parse_scutil_proxy(raw: &str) -> SystemProxy {
   |               ^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `rustcode-config` (lib) generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.30s
     Running tests/tmp_omo_skills_load2.rs (target/debug/deps/tmp_omo_skills_load2-ac1112efe1add620)

running 8 tests
skills:rustcode-ast-grep: desc_len=164 template_len=2992 src=/root/.rustcode/skills/rustcode-ast-grep/SKILL.md
skills:rustcode-debugging: desc_len=167 template_len=3165 src=/root/.rustcode/skills/rustcode-debugging/SKILL.md
skills:rustcode-git-master: desc_len=151 template_len=2768 src=/root/.rustcode/skills/rustcode-git-master/SKILL.md
skills:rustcode-lsp-setup: desc_len=287 template_len=5563 src=/root/.rustcode/skills/rustcode-lsp-setup/SKILL.md
skills:rustcode-parallel-review: desc_len=162 template_len=3274 src=/root/.rustcode/skills/rustcode-parallel-review/SKILL.md
skills:rustcode-refactor: desc_len=114 template_len=2567 src=/root/.rustcode/skills/rustcode-refactor/SKILL.md
skills:rustcode-remove-ai-slops: desc_len=156 template_len=2954 src=/root/.rustcode/skills/rustcode-remove-ai-slops/SKILL.md
skills:rustcode-rust-standards: desc_len=235 template_len=6004 src=/root/.rustcode/skills/rustcode-rust-standards/SKILL.md
===== ACTUAL REGISTRY KEYS (len=8) =====
test t1_all_eight_skills_are_indexed_under_skills_namespace ... ok
user_invocable = ["skills:rustcode-ast-grep", "skills:rustcode-debugging", "skills:rustcode-git-master", "skills:rustcode-lsp-setup", "skills:rustcode-parallel-review", "skills:rustcode-refactor", "skills:rustcode-remove-ai-slops", "skills:rustcode-rust-standards"]
===== END DUMP =====
test t0_dump_actual_registry_keys ... ok
standard_skill_dirs = [
    "/root/.claude/commands",
    "/root/.rustcode/commands",
    "/root/.claude/skills",
    "/root/.agents/skills",
    "/root/.rustcode/skills",
    "/workspace/RustCode/.claude/commands",
    "/workspace/RustCode/.rustcode/commands",
    "/workspace/RustCode/.claude/skills",
    "/workspace/RustCode/.agents/skills",
    "/workspace/RustCode/.rustcode/skills",
]
runtime_skill_dirs = [
    "/root/.claude/commands",
    "/root/.rustcode/commands",
    "/root/.claude/skills",
    "/root/.agents/skills",
    "/root/.rustcode/skills",
    "/workspace/RustCode/.claude/commands",
    "/workspace/RustCode/.rustcode/commands",
    "/workspace/RustCode/.claude/skills",
    "/workspace/RustCode/.agents/skills",
    "/workspace/RustCode/.rustcode/skills",
]
catalog len = 4631 bytes
test t6_standard_and_runtime_dirs_include_the_real_skill_root ... ok
test t3_all_eight_are_user_invocable ... ok
HOME = "/root"
RUSTCODE_HOME = ""
test t5_bare_name_resolves_and_unknown_is_rejected ... ok
reload warnings = []
reload loaded 8 skills
  reloaded: skills:rustcode-ast-grep
  reloaded: skills:rustcode-debugging
  reloaded: skills:rustcode-git-master
  reloaded: skills:rustcode-lsp-setup
  reloaded: skills:rustcode-parallel-review
  reloaded: skills:rustcode-refactor
  reloaded: skills:rustcode-remove-ai-slops
  reloaded: skills:rustcode-rust-standards
test t7_reload_through_the_real_runtime_path_loads_all_eight ... ok
rustcode-lsp-setup: template_len=8517 expand_len=8533
rustcode-rust-standards: template_len=9768 expand_len=9783
test t4_expand_injects_arguments_for_the_two_new_skills ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

#### 3.1.1 t0 dump 出的 8 个真实 key（未猜名字）

```
key = "skills:rustcode-ast-grep"
key = "skills:rustcode-debugging"
key = "skills:rustcode-git-master"
key = "skills:rustcode-lsp-setup"
key = "skills:rustcode-parallel-review"
key = "skills:rustcode-refactor"
key = "skills:rustcode-remove-ai-slops"
key = "skills:rustcode-rust-standards"
```

关键事实提取：

- 实际 key 与预期**完全一致**：`skills:` 前缀 + 目录裸名，`len() == 8`，目录内无多余项、无被吞项。
- `expand` 后字节长度 = 模板 + 16（即 `"\n\nARGUMENTS: rust"` 的长度）：lsp-setup 8517 → 8533，rust-standards 9768 → 9783。两个新技能模板内**无** `$ARGUMENTS` token（已 grep 确认命中数为 0），故参数按 `expand` 的「无 token 则尾部追加」语义注入，且 `t4` 断言了 `ends_with("ARGUMENTS: rust")`。
- `reload()` 在 `HOME=/root`、`RUSTCODE_HOME` 未设置下同样加载出 8 个，warnings 为空 —— 与 driver `/skills` 实际调用入口一致。
- 前置污染排查（保证 8 这个数字可信）：除 `/root/.rustcode/skills` 外，`standard_skill_dirs` / `runtime_skill_dirs` 列出的其余 9 个目录**全部不存在**（`/root/.claude`、`/root/.agents`、`/workspace/RustCode/.rustcode`、`/workspace/RustCode/.agents` 均 absent），因此 `reload` 计数不会被项目侧或 claude 侧技能污染。

#### 3.1.2 临时文件删除与残留证明

```
$ cd /workspace/RustCode && rm -f crates/rustcode-capabilities/tests/tmp_omo_skills_load2.rs

$ ls crates/rustcode-capabilities/tests/
anthropic_mock.rs
compaction_cache.rs
e2e.rs
fixtures
http_mock.rs
mcp.rs
memory.rs
ollama_mock.rs
session_fixture_invariants.rs
session.rs
setup_integration.rs
tools_integration.rs

$ git status --short
?? .codebuddy/artifacts/2026-09-09-omo-skills-import/03-impl/T-04.md
?? .codebuddy/artifacts/2026-09-09-omo-skills-import/03-impl/T-05.md

$ git status --porcelain | grep -E '\.rs$' || echo "(no .rs entries in git status)"
(no .rs entries in git status)

$ ls crates/rustcode-capabilities/tests/tmp_omo_skills_load2.rs
ls: cannot access 'crates/rustcode-capabilities/tests/tmp_omo_skills_load2.rs': No such file or directory
```

**结论：临时夹具已删除，磁盘与 git 均无 `.rs` 残留。**

#### 3.1.3 过程中出现过的 2 次失败（均属夹具自身缺陷，已修正后复跑全绿）

按「失败要如实记录」的要求，完整披露：

| # | 现象 | 根因 | 处置 | 是否产品缺陷 |
| --- | --- | --- | --- | --- |
| F-1 | `t4` panic：`byte index 40 is not a char boundary; it is inside '器'` | 夹具断言的失败信息里用 `&out[..40]` 按**字节**截断中文字符串 | 改为按字符截断的 `head()` / `tail()` 辅助函数 | 否（夹具 bug） |
| F-2 | `t4` 断言失败：`expand 输出未保留正文开头，head="\n# rustcode-lsp-setup\n\n把语言服务器接进 RustCode"` | 模板以 `\n# <name>` 开头（frontmatter 分隔符后残留一个换行），夹具断言写成了 `starts_with("# ...")` | 改为 `out.trim_start().starts_with(...)` | 否（夹具断言过严；产品行为正确：空白后紧跟 H1） |

两次失败均**不指向被测对象**：技能文件内容、加载器行为均无问题。修正后 `8 passed; 0 failed`。

### 3.2 验证 2：既有回归

```
cd /workspace/RustCode && cargo test -p rustcode-capabilities --features skills --lib skills
```

exit code `0`

```
running 49 tests
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 827 filtered out; finished in 0.02s
```

49 个用例名（与首批报告逐条比对，**集合完全一致**，无增无减）：

```
skills::catalog_hook::tests::fresh_inserts_after_leading_system_run
skills::catalog_hook::tests::none_catalog_is_noop_on_fresh
skills::catalog_hook::tests::resume_prunes_stale_when_no_skills_left
skills::catalog_hook::tests::resume_refreshes_in_place_no_growth
skills::registry::tests::discovers_nested_skills_under_grouping_dirs
skills::registry::tests::get_bare_name_is_ambiguous_across_namespaces
skills::registry::tests::get_bare_name_prefers_sole_user_invocable_over_hidden
skills::registry::tests::get_resolves_namespaced_skill_by_bare_name
skills::registry::tests::instruction_skill_reference_requires_exact_token_boundaries
skills::registry::tests::later_dir_overrides_same_name
skills::registry::tests::loads_flat_and_dir_skills_with_precedence
skills::registry::tests::missing_dir_is_skipped
skills::registry::tests::runtime_dirs_honor_rustcode_home_for_duplicate_directory_skills
skills::registry::tests::runtime_dirs_redirect_every_user_rustcode_dir_and_leave_others
skills::registry::tests::runtime_dirs_treat_empty_rustcode_home_as_unset
skills::registry::tests::standard_dirs_include_agents_skills_between_claude_and_rustcode
skills::render::tests::a_home_named_rustcode_is_unchanged
skills::render::tests::always_emits_top_ranked_even_if_alone_over_budget
skills::render::tests::an_empty_home_does_not_make_everything_native
skills::render::tests::a_relocated_config_tree_still_outranks_third_party_dirs
skills::render::tests::empty_yields_none
skills::render::tests::explicit_instruction_reference_outranks_source_tier
skills::render::tests::long_description_is_truncated
skills::render::tests::over_budget_omits_lowest_rank_and_counts
skills::render::tests::ranks_curated_before_community
skills::render::tests::small_catalog_emits_all_with_guidance_and_no_omitted_note
skills::render::tests::source_rank_tiers
skills::skill::tests::appends_args_when_no_arguments_token
skills::skill::tests::argument_containing_dollar_token_is_not_re_expanded
skills::skill::tests::dollar_n_boundary
skills::skill::tests::expand_arguments_full_and_positional
skills::skill::tests::frontmatter_close_at_eof
skills::skill::tests::frontmatter_parse
skills::skill::tests::frontmatter_parses_user_invocable
skills::skill::tests::frontmatter_single_quotes_and_space_tools
skills::skill::tests::name_validation
skills::skill::tests::no_frontmatter_is_all_body
skills::skill::tests::out_of_range_positional_stays_literal
skills::skill::tests::shell_injection_runs
skills::skill::tests::variable_substitution
skills::use_skill::tests::list_skills_empty
skills::use_skill::tests::list_skills_formats
skills::use_skill::tests::use_skill_directory_skill_includes_install_path_reminder
skills::use_skill::tests::use_skill_expands
skills::use_skill::tests::use_skill_finds_plugin_namespaced_skill
skills::use_skill::tests::use_skill_not_found_lists_available
skills::use_skill::tests::use_skill_plugin_namespace_shows_in_available_list
skills::use_skill::tests::use_skill_schema_requires_an_exact_available_name
skills::use_skill::tests::use_skill_single_file_skill_omits_install_path_reminder
```

数字说明：**49 与首批一致、无变化**，原因是本批零生产代码改动（见 §7），既有用例数不可能变化。

> ⚠️ 复现首批已记录的陷阱（说明为什么必须带 feature）：
> ```
> cd /workspace/RustCode && cargo test -p rustcode-capabilities --lib skills
> EXIT_BARE=0
> running 0 tests
> test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 827 filtered out; finished in 0.00s
> ```
> 裸跑匹配到 **0 个**用例（827 被过滤），`exit 0` 但**不构成任何回归证据**。真实证据必须是带 `--features skills` 的 49/49。

补充编译门禁（夹具删除后 `--all-targets` 仍可编译）：

```
cd /workspace/RustCode && cargo check -p rustcode-capabilities --features skills --all-targets
EXIT=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.37s

cd /workspace/RustCode && cargo clippy -p rustcode-capabilities --features skills --all-targets
EXIT=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.65s
```

clippy 告警归属（全部既有，与本批无关，均未阻断）：

| 告警 | 位置 | 归属 |
| --- | --- | --- |
| `function parse_scutil_proxy is never used` | `crates/rustcode-config/src/system_proxy.rs:80` | 既有（首批报告已记录） |
| `manual implementation of Option::map` | `crates/rustcode-config/src/config/memory.rs:67` | 既有，rustcode-config |
| （1 条） | `crates/rustcode-capabilities/src/askpass/server.rs:1` | 既有，与 skills 无关 |
| （1 条） | `crates/rustcode-config/src/store.rs:224` | 既有 |

### 3.3 验证 3：工具名真实性（2 个新技能）

权威清单 —— `crates/rustcode-capabilities/src/tools/` 下所有 `Tool::name()` 实现（24 个）：

```
$ cd /workspace/RustCode/crates/rustcode-capabilities/src/tools && grep -rn -A1 'fn name(&self) -> &' --include=*.rs .
./ast_grep.rs:42            "ast_grep"
./atomgit.rs:107            "atomgit_api"
./atomgit.rs:340            "atomgit_issue"
./atomgit.rs:526            "atomgit_pr"
./atomgit.rs:686            "atomgit_repo"
./bash.rs:121               "bash"
./cd.rs:48                  "change_dir"
./edit.rs:50                "edit_file"
./output_artifact.rs:360    "fetch_output"
./glob.rs:29                "glob"
./grep.rs:47                "grep"
./list.rs:27                "list_directory"
./memory.rs:60              "memory"
./open_file.rs:137          "open_file"
./parallel_edit.rs:101      "parallel_edit_files"
./read.rs:272               "read_file"
./report_finding.rs:77      "report_finding"
./request_user_input.rs:240 "request_user_input"
./search_replace.rs:33      "search_replace"
./task.rs:669               "task"
./todo.rs:363               "todowrite"
./web_fetch.rs:70           "web_fetch"
./web_search.rs:136         "web_search"
./write.rs:23               "write_file"
```

2 个新技能的 `allowed-tools` 原始清单：

```
$ cd /root/.rustcode/skills && grep -m1 '^allowed-tools:' rustcode-lsp-setup/SKILL.md rustcode-rust-standards/SKILL.md
rustcode-lsp-setup/SKILL.md:4:allowed-tools: bash, read_file, grep, glob, edit_file, search_replace
rustcode-rust-standards/SKILL.md:4:allowed-tools: read_file, write_file, edit_file, grep, glob, bash, ast_grep, search_replace, todowrite, report_finding
```

逐项核实（16 项 / 去重 10 名，**全部命中**）：

| 工具名 | 注册文件:行 | 出现在 |
| --- | --- | --- |
| `bash` | `tools/bash.rs:121` | lsp-setup, rust-standards |
| `read_file` | `tools/read.rs:272` | lsp-setup, rust-standards |
| `grep` | `tools/grep.rs:47` | lsp-setup, rust-standards |
| `glob` | `tools/glob.rs:29` | lsp-setup, rust-standards |
| `edit_file` | `tools/edit.rs:50` | lsp-setup, rust-standards |
| `search_replace` | `tools/search_replace.rs:33` | lsp-setup, rust-standards |
| `write_file` | `tools/write.rs:23` | rust-standards |
| `ast_grep` | `tools/ast_grep.rs:42` | rust-standards |
| `todowrite` | `tools/todo.rs:363` | rust-standards |
| `report_finding` | `tools/report_finding.rs:77` | rust-standards |

**未找到：0 个。**

关于 `lsp`（任务书特别提示的项）：

```
$ grep -rn "^[[:space:]]*\"lsp\"" --include=*.rs crates/rustcode-capabilities/src/codeintel/
crates/rustcode-capabilities/src/codeintel/lsp_tool.rs:53:        "lsp"
```

`lsp` 的真实注册点是 `crates/rustcode-capabilities/src/codeintel/lsp_tool.rs:53`，**不在** `tools/` 下 —— 与任务书提示一致。

但本次的判定结论是：**该提示不触发**。8 个技能（含 `rustcode-lsp-setup`）没有任何一个把 `lsp` 写进 `allowed-tools`：

```
$ grep -rn '^allowed-tools:' */SKILL.md | grep -w 'lsp'
rustcode-lsp-setup/SKILL.md:4:allowed-tools: bash, read_file, grep, glob, edit_file, search_replace
   ^ 该行被 -w 命中的原因是路径中的 "rustcode-lsp-setup" 里 lsp 是独立单词，
     而 allowed-tools 的实际内容里没有 lsp（已人工核对）
```

即 T-04（§6 决策记录第 1 条）主动把 `lsp` 排除在 `allowed-tools` 之外，改为全部走 `bash` 校验。这是**合规且不判 FAIL** 的取舍；若编排者希望把 `lsp` 补入，需回退 T-04 改一行（T-04 已自行标注）。

### 3.4 验证 4：仓库卫生与门禁

```
$ cd /workspace/RustCode && git status --short
?? .codebuddy/artifacts/2026-09-09-omo-skills-import/03-impl/T-04.md
?? .codebuddy/artifacts/2026-09-09-omo-skills-import/03-impl/T-05.md
EXIT=0
```

判定：worktree 干净（与任务书「刚提交过 2 个 commit，worktree 应基本干净」一致）。仅有的 2 项是本批两个任务的交接件，位于 `.codebuddy/artifacts/` 豁免域，**无 `.rs` / `Cargo.toml` / `README*` / `AGENTS.md` / `docs/**` 改动**。

```
$ cd /workspace/RustCode && python3 scripts/check-zh-docs.py gate
EXIT=0
gate: artifacts 豁免域 `.codebuddy/artifacts/` 已排除：108 个（...）
gate: base=3ee655e3
PASS AC-1 清单自洽
PASS AC-2/3/4/6/7b/32 全量 check
PASS AC-5 运行时载荷
PASS AC-7a 无改名/删除
PASS AC-8 外链残留
gate: PASS
```

**结论：PASS，exit 0。**

### 3.5 验证 5：config.toml LSP 键名独立核实（不采信 doc-writer 报告）

#### 5.1 `LspConfig` 是否存在于其声称的位置，字段与行号是否对得上

```
$ grep -rn "struct LspConfig" crates/rustcode-config/src/
crates/rustcode-config/src/config/mod.rs:1538:pub struct LspConfig {

$ grep -n "pub enabled: bool\|pub auto_detect: bool\|pub servers:\|pub diagnostics_settle_delay_ms" crates/rustcode-config/src/config/mod.rs
1541:    pub enabled: bool,
1547:    pub auto_detect: bool,
1550:    pub servers: std::collections::HashMap<String, crate::lsp_registry::LspServerConfig>,
1555:    pub diagnostics_settle_delay_ms: u64,
```

`sed -n '1530,1575p'` 原文（节选关键行）：

```
1538 pub struct LspConfig {
1540     #[serde(default)]
1541     pub enabled: bool,
1546     #[serde(default)]
1547     pub auto_detect: bool,
1548     /// Custom server configurations keyed by file extension.
1549     #[serde(default)]
1550     pub servers: std::collections::HashMap<String, crate::lsp_registry::LspServerConfig>,
1554     #[serde(default = "default_diagnostics_settle_delay_ms")]
1555     pub diagnostics_settle_delay_ms: u64,
```

字段上只有 `#[serde(default)]` / `#[serde(default = "...")]`，**无 `rename`、无 `flatten`**，故「键名即字段名」成立。
`[lsp.servers.<ext>]` 段的三个键：

```
crates/rustcode-config/src/lsp_registry.rs:1  //! The `LspServerConfig` config type (`[lsp.servers.<ext>]`).
crates/rustcode-config/src/lsp_registry.rs:7  pub struct LspServerConfig {
crates/rustcode-config/src/lsp_registry.rs:8      pub command: String,
crates/rustcode-config/src/lsp_registry.rs:10     pub args: Vec<String>,
crates/rustcode-config/src/lsp_registry.rs:12     pub root_markers: Vec<String>,
```

**判定：属实。** `LspConfig` 在 `crates/rustcode-config/src/config/mod.rs:1538`，四个字段行号 1541 / 1547 / 1550 / 1555 与 T-04 声称**完全一致**；`command`/`args`/`root_markers` 在 `lsp_registry.rs:8/10/12` 亦与声称一致。

#### 5.2 `lsp.servers` 的键是不是文件扩展名、内置表用 `rs` 还是 `rust`

- `config/mod.rs:1548` 注释原文：`/// Custom server configurations keyed by file extension.` → **键是文件扩展名**。
- 内置表：

```
$ sed -n '38,58p' crates/rustcode-capabilities/src/codeintel/lsp/registry.rs
        add("rs", "rust-analyzer", &[], &["Cargo.toml"]);
        add("ts", "typescript-language-server", &["--stdio"], &["tsconfig.json", "package.json"]);
        add("tsx", "typescript-language-server", &["--stdio"], &["tsconfig.json"]);
        add("js", "typescript-language-server", &["--stdio"], &["package.json"]);
        add("py", "pylsp", &[], &["pyproject.toml", "setup.py"]);
        add("go", "gopls", &["serve"], &["go.mod"]);
        add("java", "jdtls", &[], &["pom.xml", "build.gradle"]);

$ grep -n 'get("rs")' crates/rustcode-capabilities/src/codeintel/lsp/registry.rs
113:        assert_eq!(r.get("rs").unwrap().command, "rust-analyzer");
```

- 运行时尚有一次归一化（**T-04 未提及，但不构成错误**）：

```
crates/rustcode-capabilities/src/codeintel/mod.rs:140:            extension.trim_start_matches('.').to_ascii_lowercase(),
```

**判定：属实。** `servers` 键为文件扩展名；内置表用 **`rs`**（`rust` 无效）；源码在装配时还会 `trim_start_matches('.')` + 转小写，因此 `.rs` / `RS` 也会被归一成 `rs`。T-04 的落笔（用不带点的 `rs`）**可用且安全**。

#### 5.3 运行时 `settle_delay_ms` 是否对应 config 键 `diagnostics_settle_delay_ms`

```
$ sed -n '232,252p' crates/rustcode-coding/src/config.rs
232 pub fn lsp_settings_from_config(
233     config: &rustcode_config::config::LspConfig,
234 ) -> rustcode_capabilities::codeintel::LspSettings {
235     rustcode_capabilities::codeintel::LspSettings {
236         enabled: config.enabled,
237         auto_detect: config.auto_detect,
238         settle_delay_ms: config.diagnostics_settle_delay_ms,
239         servers: config
240             .servers
241             .iter()
242             .map(|(extension, server)| {
```

**判定：属实。** `crates/rustcode-coding/src/config.rs:238` 明确 `settle_delay_ms ← diagnostics_settle_delay_ms`，与 T-04 声称的行号一致。config 侧与运行时侧名字不同这一点是**真实存在的改名**。

#### 5.4 `migrate_legacy_lsp_default` 是否存在、四条同时成立是否整段重置

```
$ grep -n "migrate_legacy_lsp_default" -r crates/rustcode-config/src/
crates/rustcode-config/src/config/mod.rs:1594:fn migrate_legacy_lsp_default(cfg: &mut Config) {
crates/rustcode-config/src/config/mod.rs:1955:        migrate_legacy_lsp_default(&mut config);
crates/rustcode-config/src/config/mod.rs:1994:        migrate_legacy_lsp_default(&mut config);
（另有 2913/2942/2952/2963/2976 为测试用例调用点）
```

函数体原文（`sed -n '1594,1603p'`）：

```
1594 fn migrate_legacy_lsp_default(cfg: &mut Config) {
1595     let looks_auto_written = cfg.lsp.enabled
1596         && cfg.lsp.auto_detect
1597         && cfg.lsp.diagnostics_settle_delay_ms == 150
1598         && cfg.lsp.servers.is_empty();
1599     if looks_auto_written {
1600         cfg.lsp = LspConfig::default();
1601     }
1602 }
```

`LspConfig::default()`（`mod.rs:1564-1572`）= `enabled: false, auto_detect: false, servers: 空, diagnostics_settle_delay_ms: 150`。
函数上方注释亦自陈（`mod.rs:1589-1592`）：

```
/// False-positive risk: a user who manually wrote `enabled=true +
/// auto_detect=true + delay=150 + servers={}` exactly gets silently
/// reset.
```

**判定：属实。** 四条（`enabled == true` && `auto_detect == true` && `delay == 150` && `servers` 为空）同时成立时，整个 `[lsp]` 段被替换为 `LspConfig::default()`，即**开关全关**（delay 仍是 150，但 enabled/auto_detect 变 false）。
→ 因此 `enabled=true + auto_detect=true + diagnostics_settle_delay_ms=150 + servers 空` 这个「最小配置」组合**确实会失效**（下次加载即被重置为全关）。T-04 据此推荐 `diagnostics_settle_delay_ms = 300` 是**正确且必要**的规避手段。

#### 5.5 附带核实（T-04 §3 其余声称，同属抽查项）

| 声称 | 判定 | 证据 |
| --- | --- | --- |
| `register_lsp_tool` 在 `codeintel/mod.rs:129`，`!enabled` 直接 return false | 属实 | `codeintel/mod.rs:129` `pub fn register_lsp_tool(...)`；`:130-132` `if !settings.enabled { return false; }` |
| 仅 `enabled=true` 而 `auto_detect=false` + `servers` 空 → 注册表为空 | 属实 | `codeintel/mod.rs:133-137`：`auto_detect` 为 false 时 `LspServerRegistry::empty()`，随后无 server 可插入 |
| 工具名是 `lsp` | 属实 | `codeintel/lsp_tool.rs:53` |
| `LspSettings` 是**运行时**结构，不是 config 结构 | 属实 | `codeintel/mod.rs:108` `pub struct LspSettings { enabled / auto_detect / servers / settle_delay_ms }` |
| `Config` 顶层无 `deny_unknown_fields`（仅 `config/provider.rs:135` 有） | 属实 | `config/mod.rs` 内仅 `:3731` 出现一次**注释**提及；`config/provider.rs:135` 是真实属性 |

#### 5.6 V5 小结

| # | 声称 | 判定 | 源文件:行号 |
| --- | --- | --- | --- |
| 1 | `LspConfig` 位置与四个字段行号 | **属实** | `crates/rustcode-config/src/config/mod.rs:1538 / 1541 / 1547 / 1550 / 1555` |
| 2 | `servers` 键是扩展名、内置表用 `rs` | **属实** | `config/mod.rs:1548`；`codeintel/lsp/registry.rs:38`（测试 `:113`） |
| 3 | `settle_delay_ms` ← `diagnostics_settle_delay_ms` | **属实** | `crates/rustcode-coding/src/config.rs:238` |
| 4 | `migrate_legacy_lsp_default` 四条成立即整段重置为全关 | **属实** | `config/mod.rs:1594`（定义）、`1595-1601`（逻辑）、`1955`/`1994`（调用） |

**doc-writer（T-04）在该项上的 4 条声称全部属实，无臆造键名。**

---

## 4. 覆盖分析

### 4.1 已覆盖

| 路径 | 覆盖情况 |
| --- | --- |
| `load_dir` + `skills` 命名空间前缀 | ✅ t0/t1：先 dump 真实 key 后断言，集合与 `len()` 双向校验 |
| 目录式技能发现（`*/SKILL.md`） | ✅ 8/8 均为目录式，隐式覆盖 |
| frontmatter 解析（name / description / allowed-tools） | ✅ t2（description 非空 + 长度打印）、t4 |
| `user_invocable()` | ✅ t3 |
| `render_catalog()`（模型侧技能目录） | ✅ t3（8 个全名均在 catalog 内，4631 bytes） |
| `get()` 全名 / 裸名 / 大小写不敏感 / **未知名失败路径** | ✅ t1 / t5 |
| `expand()` 参数注入 | ✅ t4（2 个新技能，尾部注入 + 长度增长 + 正文保留 + 无残留 token） |
| `standard_skill_dirs` / `runtime_skill_dirs` | ✅ t6 |
| `reload()`（driver 真实入口） | ✅ t7（`HOME=/root`、`RUSTCODE_HOME` 未设置） |
| 增量回归（新增 2 个不破坏已有 6 个） | ✅ t1 显式断言首批 6 个仍命中 |
| 既有 skills 单元回归 | ✅ 49/49，集合与首批一致 |
| 目录缺失 / 解析失败静默跳过 | ✅ 既有单测 `skills::registry::tests::missing_dir_is_skipped`；`load_dir` 的 `read_dir`/`parse` 失败 skip 语义 |
| 中文文档门禁 | ✅ gate exit 0 |

### 4.2 未覆盖（含原因与负责人）

| 未覆盖项 | 原因 | 负责人 |
| --- | --- | --- |
| `parse_frontmatter` / `validate_skill_name` / `make_name` 针对这 8 个文件输入的**直接单测** | 三者均为私有 `fn`（`skills/skill.rs:223/297/319`），集成测试不可达；已由 `load_dir` 端到端行为间接覆盖（key 正确 ⇒ 命名与解析通过；description 正确 ⇒ frontmatter 解析通过） | 无需补（既有 49 个单测已覆盖通用语义） |
| `expand_for_injection()` 的 `<system-reminder>` 安装目录提示 | 任务书只要求 `expand`；`expand_for_injection` 属 `use_skill` 路径，既有单测 `use_skill_directory_skill_includes_install_path_reminder` 已覆盖 | 无需补 |
| CLI / TUI / daemon / headless / background / ACP / clix 各宿主**端到端**技能调用 | 本批零生产代码改动；所有宿主共享 `SkillRegistry::reload()`，已由 t7 验证 | 无需补 |
| `allowed-tools` 的**运行时强制** | 按设计不强制（`skills/skill.rs:17-19` 注释：L1 不校验，属 L2 审批策略）；本项只做名字真实性静态核对 | 无需补 |
| 持久化兼容（旧格式只读导入） | 本批不涉及持久化 | 无需补 |
| 取消 / 重试 / 并发 | 加载器是同步、纯函数式目录扫描，无 cancel / retry / 并发路径；对应失败面（未知名拒绝、目录缺失跳过、解析失败跳过）已覆盖 | 无需补 |
| `cargo test --workspace` / `cargo check --workspace --all-targets` | 本批**零 `.rs` / `Cargo.toml` 改动**（`git status` 证明），无跨 crate 影响面；G5 要求的 workspace 级检查针对「跨 crate 或公共协议变更」，本批不适用 | 无需补（若编排者要求可补跑） |
| `lsp` 工具**实机可用**（配好 `[lsp]` 重启后工具清单出现 `lsp`） | 需要真实写 `/root/.rustcode/config.toml` 并启动 runtime，会改变用户环境；T-04 §6.2 已自行标注为「未实机验证」 | 编排者决定是否安排实机验证 |
| 8 个 SKILL.md 正文条目的**语义正确性**（如 AGENTS.md 行号引用是否漂移） | 超出「能否加载 + 工具名真实性 + LSP 键名」的验证范围；本批只核实了任务书点名的 LSP schema 4 条 | 建议转 doc-writer 复核（尤其 T-05 的 `AGENTS.md` 行号引用） |

---

## 5. 回归结论

| 入口 / 面 | 是否受影响 | 是否验证 | 结论 |
| --- | --- | --- | --- |
| CLI / TUI / daemon / headless / background / ACP / clix | 否（本批仅新增 2 个 `SKILL.md` 数据文件 + 2 个交接件，零生产代码改动） | 共享入口 `SkillRegistry::reload()` 已由 t7 验证（8/8） | 无需逐入口回归 |
| `rustcode-capabilities` crate | 是（临时夹具曾落在该 crate，已删除） | 是 | `--lib skills` 49/49 绿；`cargo check --features skills --all-targets` exit 0；`cargo clippy --features skills --all-targets` exit 0 |
| 既有 6 个首批技能 | 是（增量回归重点） | 是 | t1 显式断言 6 个仍命中；t2/t3/t7 覆盖全部 8 个 |
| 中文文档门禁 | 是（新增交接件落在 artifacts 域） | 是 | `gate` exit 0 |

---

## 6. 失败与阻塞

### 6.1 失败用例

**最终失败：0。**（`8 passed; 0 failed; 0 ignored`；`49 passed; 0 failed; 0 ignored`）

过程中出现的 2 次失败（F-1 / F-2）见 §3.1.3：**均为本测试夹具自身缺陷，不影响被测对象，已修正后复跑全绿**，无需回退 `code-implementer`。

若需复现 F-1/F-2（夹具已删除，仅供追溯）：
1. 在 `tests/` 下新建夹具，对 `skills:rustcode-lsp-setup` 调 `expand("rust","sess-test")；
2. 失败信息里用 `&out[..out.len().min(40)]` 取片段 → panic `byte index 40 is not a char boundary`（中文多字节）；
3. 改用字符截断后，断言 `out.starts_with("# rustcode-lsp-setup")` → 失败，实际输出以 `\n#` 开头。

### 6.2 非阻塞观察项

| # | 现象 | 证据 | 影响 | 建议归属 |
| --- | --- | --- | --- | --- |
| OBS-B2-1 | **`rustcode-debugging/SKILL.md` 在首批验证之后被编辑过**，首批报告 OBS-1（description 内转义反斜杠）已被修复 | `stat` mtime：`rustcode-debugging/SKILL.md` = `07:30:14`，而其同批 5 个为 `07:10:05 ~ 07:12:48`（创建时间）；description 字符数 171 → **167**（4 个 `\"` 转义被清除）；`grep -c '\\"'` 在 8 个文件上**全部为 0**；本批 dump 显示已变为中文弯引号 `“为什么 X 不工作”“帮我定位这个 ` | 正面变化，首批 OBS-1 已闭环；本批已重新验证 8/8 加载正常 | 无需动作（记录备查）。**注：首批 5 个文件的 template 字符数与首批报告的 -2 差值属**度量口径差异**（首批按 `trim_end`、本批按 `trim()`，模板首尾共 3 个空白字符），非内容变更 —— 已用 `trim_end` 口径复算，5/6 模板长度与首批**逐字节一致** |
| OBS-B2-2 | `rustcode-lsp-setup` 的 `allowed-tools` **不含 `lsp`**（该技能自身就是为了配 LSP） | `SKILL.md:4: allowed-tools: bash, read_file, grep, glob, edit_file, search_replace`；`lsp` 注册于 `codeintel/lsp_tool.rs:53`（非 `tools/`） | 技能功能不受影响（校验全部走 `bash`）；但用户在「工具自动放行」语义下不会自动放行 `lsp` | 编排者决策：若要补入，回退 T-04 改一行（T-04 §6.1 已自行标注并预留） |

### 6.3 阻塞项

无。环境（Linux / zsh / `HOME=/root` / `RUSTCODE_HOME` 未设置）具备完整验证条件；无「以跳过冒充通过」的情形，`0 ignored` / `0 filtered out`。

---

## 7. 未修改生产代码 / 未 commit 的证明

```
$ git status --short              # 验证前（基线）
?? .codebuddy/artifacts/2026-09-09-omo-skills-import/03-impl/T-04.md
?? .codebuddy/artifacts/2026-09-09-omo-skills-import/03-impl/T-05.md

$ git status --short              # 验证后（完全一致）
?? .codebuddy/artifacts/2026-09-09-omo-skills-import/03-impl/T-04.md
?? .codebuddy/artifacts/2026-09-09-omo-skills-import/03-impl/T-05.md

$ git status --porcelain | grep -E '\.rs$' || echo "(no .rs entries)"
(no .rs entries)

$ git rev-parse --abbrev-ref HEAD
dev
$ git --no-pager log --oneline -2
d5b4f2e1 chore(artifacts): OMO skills 导入首批交接件与看板
8ae5d97c docs(readme): 账号命令改用运行时判据而非构建渠道
```

- 8 个 `SKILL.md`：**未修改**（全程只读 `grep` / `sed` / `stat` / python 只读测量）。
- 生产源码：**未修改**（无 `.rs` 进入 git status）。
- 临时测试夹具：已删除，磁盘与 git 均无残留。
- `git add` / `git commit` / `git push`：**未执行**。

---

## 8. 边界声明（已验证 / 未验证）

**已验证**：8 个技能能被 `SkillRegistry` 经 `load_dir` 与 driver 真实入口 `reload()` 加载；description / template 非空；`expand` 参数注入；`user_invocable` 与 catalog；既有 49 个单测；2 个新技能 16 项 `allowed-tools` 全部注册；`git status` 与中文文档门禁；LSP config schema 4 条声称。

**未验证**：
1. 各宿主（CLI/TUI/daemon/ACP/clix）端**到端**触发技能（无生产代码改动，共享入口已验证）。
2. `lsp` 工具在真实配置后的**实机可用性**（需写用户 config 并重启 runtime，本批不动用户环境）。
3. 8 个 SKILL.md **正文语义**与其对 `AGENTS.md` 行号引用的长期正确性（仅核实了任务书点名的 LSP schema 4 条）。
4. workspace 级 `cargo test --workspace` / `cargo check --workspace --all-targets`（零跨 crate 改动，判定不适用）。
