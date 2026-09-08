---
kind: test-report
id: TEST-001
from: test-engineer
to: [project-manager, code-implementer]
feature: 2026-09-09-omo-skills-import
status: done
decision: proceed
requires: [DESIGN-001, OMO-SKILLS-IMPORT]
files_owned:
  - crates/rustcode-capabilities/tests/tmp_omo_skills_load.rs  # 临时夹具，已删除，未入库
  - .codebuddy/artifacts/2026-09-09-omo-skills-import/05-test-report.md
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-09
---

# 05 · 测试报告 —— 6 个新写入运行时 skills 的真实加载验证

## 0. 结论速览

| 验证项 | 结论 | exit code | 证据强度 |
| --- | --- | --- | --- |
| V1 真实加载（核心） | **PASS** | 0 | 强 —— 走公开 API `load_dir` + `reload`，8/8 用例绿 |
| V2 既有 skills 回归 | **PASS** | 0 | 强 —— 49/49 绿（需 `--features skills`）|
| V3 README 门禁复核 | **PASS** | 0 | 强 —— gate 与 check 双绿 |
| V4 工具名真实性抽查 | **PASS（13/13 存在）** | 0 | 强 —— 逐名比对 `Tool::name()` 实现 |
| V4b 前序 `cd`/`output_artifact` 结论复核 | **属实** | 0 | 强 —— `cd.rs:48`=`change_dir`，`output_artifact.rs:360`=`fetch_output` |

**用例总数 8（临时加载夹具）+ 49（既有回归）+ 2（门禁）+ 13（工具名抽查）；失败 0。**
**决策：`proceed`。** 但 V4 附带 1 条**非阻塞**文档观察项（见 §6.2），建议编排者转 doc-writer 处置，不阻塞发布。

执行路径说明（按任务书要求如实声明）：
- V1 走的是 **`SkillRegistry::load_dir(&PathBuf::from("/root/.rustcode/skills"), Some("skills"))` 这一条公开路径**（未触发备选路径）。理由：`load_dir` 是 `pub`，`SkillRegistry::new/load_dir/get/list/all/user_invocable/render_catalog/reload` 全部 `pub`，`Skill` 各字段亦 `pub`，因此**没有 pub 可见性问题**，`parse_skill_dir`（`pub(crate)`）这条降级路径最终**未使用**。
- 额外叠加了更强的 `standard_skill_dirs` / `runtime_skill_dirs` / `SkillRegistry::reload` 运行时路径（用例 t6/t7），因此证据等级高于「只验证解析」。
- 关键前置：**`skills` 是 opt-in cargo feature**（`Cargo.toml` `skills = []`，不在 `default = ["provider","tools"]` 内），所以所有命令都显式带 `--features skills`。

---

## 1. 测试策略

| 层次 | 划分 | 理由 |
| --- | --- | --- |
| 临时集成夹具（`tests/tmp_omo_skills_load.rs`） | 集成 | 直接对真实磁盘上的 `/root/.rustcode/skills` 跑公开加载入口，是「能否被真正加载」的唯一可信证据；静态文本比对无法覆盖 frontmatter 解析、命名归一化、命名空间前缀、目录式技能发现等运行时行为。 |
| 既有单元回归（`--lib skills`） | 单元/回归 | 本次不改生产代码，需证明 `skills` 模块既有 49 个用例未被破坏。 |
| 门禁脚本（`scripts/check-zh-docs.py`） | 门禁回归 | 复核前序未提交 README 改动。 |
| 源码 grep 抽查 | 静态核对 | 工具名真实性无法在加载路径上断言（`allowed_tools` 只是元数据、L1 明确不校验），故用 `Tool::name()` 实现做权威比对。 |

未采用「只断言不 panic」的任何用例；所有断言均为可观测值（key 集合、description/template 长度、`expand` 输出内容、`reload` 后的索引集合）。

---

## 2. 用例清单

### 2.1 AC → 用例映射（无漏测）

| 验收点 | 用例 id | 覆盖场景 | 结果 |
| --- | --- | --- | --- |
| 6 个技能全部被索引（命名空间后名字） | `t0` + `t1` | 先 dump 实际 key 再断言，避免猜名字；并断言 `len == 6` 无多余项 | PASS |
| 每个技能 `description` 非空 | `t2` | 逐个断言 `!desc.trim().is_empty()`，打印长度 | PASS |
| 每个技能 `template` 非空 | `t2` | 逐个断言 `!template.trim().is_empty()` + `source_path.is_file()` | PASS |
| `user_invocable()` 能列出它们 | `t3` | `user_invocable()` 迭代器集合比对；并断言渲染出的 catalog 含全部 6 个全名 | PASS |
| `expand("示例参数","sess-test")` 非空且含参数 | `t4` | 主验证 `rustcode-ast-grep`；另对 6 个全跑一遍 | PASS |
| 运行时路径（`standard_skill_dirs`/`runtime_skill_dirs`）包含真实技能根 | `t6` | 断言两个函数输出均含 `/root/.rustcode/skills` | PASS |
| 真实 driver 入口 `reload()` 能加载 6 个 | `t7` | 用 `dirs::home_dir()` 真实解析后的 `reload` | PASS |
| 菜单/`$` 触发路径（裸名解析） | `t5` | 裸名 + 大小写不敏感 + 未知名必须拒绝 | PASS |

### 2.2 用例总表

| 用例 id | 类型 | 覆盖场景（AC 映射） | 文件路径 | 结果 |
| --- | --- | --- | --- | --- |
| `t0_dump_actual_registry_keys` | 集成（调试） | 打印真实 key，为后续断言提供事实基线 | `crates/rustcode-capabilities/tests/tmp_omo_skills_load.rs`（已删） | PASS |
| `t1_all_six_skills_are_indexed_under_skills_namespace` | 集成 | 6/6 索引 + 恰好 6 个 | 同上 | PASS |
| `t2_every_skill_has_non_empty_description_and_template` | 集成 | description/template/source_path | 同上 | PASS |
| `t3_all_six_are_user_invocable` | 集成 | `user_invocable()` + `render_catalog()` | 同上 | PASS |
| `t4_expand_injects_arguments_and_session` | 集成 | `$ARGUMENTS` 追加行为 | 同上 | PASS |
| `t5_bare_name_resolves_for_menu_and_dollar_trigger` | 集成 | 裸名/大小写/未知名 | 同上 | PASS |
| `t6_standard_and_runtime_dirs_include_the_real_skill_root` | 集成 | 目录优先级路径 | 同上 | PASS |
| `t7_reload_through_the_real_runtime_path_loads_all_six` | 集成 | driver 真实入口 `reload()` | 同上 | PASS |
| `skills::*` 49 个既有用例 | 单元/回归 | 既有 skills 行为未被破坏 | `crates/rustcode-capabilities/src/skills/**` | PASS (49/49) |
| `check-zh-docs.py gate` | 门禁 | README 改动后全仓门禁 | `scripts/check-zh-docs.py` | PASS |
| `check-zh-docs.py check --files README.md` | 门禁 | README 单文件 | 同上 | PASS |
| 13 × 工具名存在性 | 静态抽查 | `allowed-tools` 真实性 | `crates/rustcode-capabilities/src/tools/**` | PASS (13/13) |

---

## 3. 执行证据

### 3.1 验证 1（核心）：真实加载

命令：

```
cd /workspace/RustCode && cargo test -p rustcode-capabilities --features skills --test tmp_omo_skills_load -- --nocapture
```

exit code：`0`

原始输出（编译 warning 已保留尾部；`rustcode-config` 的 `parse_scutil_proxy` dead_code warning 为既有告警，与本次无关）：

```
warning: function `parse_scutil_proxy` is never used
  --> crates/rustcode-config/src/system_proxy.rs:80:15
   |
80 | pub(crate) fn parse_scutil_proxy(raw: &str) -> SystemProxy {
   |               ^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) by default

warning: `rustcode-config` (lib) generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.08s
     Running tests/tmp_omo_skills_load.rs (target/debug/deps/tmp_omo_skills_load-2ea6c6ad7c6401d8)

running 8 tests

===== ACTUAL REGISTRY KEYS (len=6) =====
key = "skills:rustcode-ast-grep"
  desc[..80] = 结构性代码搜索与批量改写。当用户要求找出所有形如 X 的函数/调用/导入/声明、按语法形状批量改名或改写、清理某种代码模式（如裸 unwrap、空 catch、

key = "skills:rustcode-debugging"
  desc[..80] = 真实运行时问题调试。当用户报告崩溃、panic、静默失败、结果不对、进程挂住、内存泄漏、间歇性失败、只在 CI 失败、或说\\\"为什么 X 不工作\\\"\\\"帮我定位

key = "skills:rustcode-git-master"
  desc[..80] = 原子提交与 git 历史调查。当用户要求提交改动、拆分提交、写提交信息、整理暂存区、或调查历史（谁写的、什么时候加的、哪次提交引入的、这行为什么这样）时使用。r

key = "skills:rustcode-parallel-review"
  desc[..80] = 实现完成后的多视角并行补充审查。当改动已写完、想在提交或提 PR 前再做一轮覆盖更广的检查，或用户要求交叉审查、多角度看一遍、找内置审查漏掉的问题时使用。是 r

key = "skills:rustcode-refactor"
  desc[..80] = 行为不变的重构。当用户要求重构、拆分、提取、简化、现代化、降低复杂度、收敛重复代码、或移动/重命名模块时使用。原则是行为锁定在先：先补测试或固化行为证据，再动手

key = "skills:rustcode-remove-ai-slops"
  desc[..80] = 清除 AI 生成代码异味（slop）。当用户要求去除 AI 味、清理生成代码、清理本次改动里的冗余与浮夸写法、或做提交前代码卫生检查时使用。先锁定行为再清理，分

===== END DUMP =====
test t1_all_six_skills_are_indexed_under_skills_namespace ... ok
user_invocable = ["skills:rustcode-ast-grep", "skills:rustcode-debugging", "skills:rustcode-git-master", "skills:rustcode-parallel-review", "skills:rustcode-refactor", "skills:rustcode-remove-ai-slops"]
rustcode-ast-grep: desc_len=164 template_len=2994 src=/root/.rustcode/skills/rustcode-ast-grep/SKILL.md
test t0_dump_actual_registry_keys ... ok
rustcode-debugging: desc_len=171 template_len=3167 src=/root/.rustcode/skills/rustcode-debugging/SKILL.md
rustcode-git-master: desc_len=151 template_len=2770 src=/root/.rustcode/skills/rustcode-git-master/SKILL.md
rustcode-parallel-review: desc_len=162 template_len=3276 src=/root/.rustcode/skills/rustcode-parallel-review/SKILL.md
rustcode-refactor: desc_len=114 template_len=2569 src=/root/.rustcode/skills/rustcode-refactor/SKILL.md
rustcode-remove-ai-slops: desc_len=156 template_len=2956 src=/root/.rustcode/skills/rustcode-remove-ai-slops/SKILL.md
test t2_every_skill_has_non_empty_description_and_template ... ok
catalog head:
=== AVAILABLE SKILLS ===
Skills are reusable instruction templates for specific tasks. The names listed below are the only skill names you may pass directly to `use_skill`; never invent or guess a skill name from memory, task type, or common workflows. Match a task only against descriptions actually shown below. If a task clearly matches a shown skill's description -- not only when the user names the skill -- you MUST load that exact skill with `use_skill` and follow it BEFORE doing the work, INCLUDING before asking clarifying questions, exploring, or planning. If this catalog says skills were
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
RUSTCODE_HOME = None
HOME = Some("/root")
test t3_all_six_are_user_invocable ... ok
test t6_standard_and_runtime_dirs_include_the_real_skill_root ... ok
test t5_bare_name_resolves_for_menu_and_dollar_trigger ... ok
reload warnings = []
reload loaded 6 skills
  reloaded: skills:rustcode-ast-grep
  reloaded: skills:rustcode-debugging
  reloaded: skills:rustcode-git-master
  reloaded: skills:rustcode-parallel-review
  reloaded: skills:rustcode-refactor
  reloaded: skills:rustcode-remove-ai-slops
test t7_reload_through_the_real_runtime_path_loads_all_six ... ok
---- expand(rustcode-ast-grep) head ----

# rustcode-ast-grep

按语法结构（AST 形状）搜索代码，并在证据充分后做批量改写。

## 何时使用

[+] 用本技能：
- 找"所有形如 X 的调用/声明/导入"，例如所有 `unwrap()`、所有 `fn $N($$$) -> Result<_, String>`、所有 `impl Trait for Type` 块。
- 按结构批量改写：给某个函数加参数、把构造式换成构造器、统一错误类型
rustcode-ast-grep: expand_len=3010
rustcode-debugging: expand_len=3228
rustcode-git-master: expand_len=2786
rustcode-parallel-review: expand_len=3292
rustcode-refactor: expand_len=2585
rustcode-remove-ai-slops: expand_len=2972
test t4_expand_injects_arguments_and_session ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

**失败用例：无**（0 failed）。

关键事实提取：
- 实际 key 与预期**完全一致**，无需猜测：`skills:rustcode-ast-grep` / `skills:rustcode-debugging` / `skills:rustcode-git-master` / `skills:rustcode-parallel-review` / `skills:rustcode-refactor` / `skills:rustcode-remove-ai-slops`。
- `reg.len() == 6`，即该目录下**没有**多余/被误吞的技能。
- `expand` 后长度均大于 template 长度（如 ast-grep 2994 → 3010），证明 `ARGUMENTS: 示例参数` 追加生效；`t4` 亦断言 `contains("示例参数")`。
- `reload()`（`dirs::home_dir()` 真实解析，`HOME=/root`，`RUSTCODE_HOME` 未设置）同样加载出 6 个 —— 这是 driver `/skills` 实际调用的入口。

#### 临时文件删除与残留证明

```
cd /workspace/RustCode && rm -f crates/rustcode-capabilities/tests/tmp_omo_skills_load.rs
```

```
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
 M README.md
?? .codebuddy/artifacts/2026-09-09-omo-skills-import/
?? .codebuddy/artifacts/2026-09-09-readme-account-cmds/

$ git status --porcelain | grep -E '\.rs$' || echo "(no .rs entries in git status)"
(no .rs entries in git status)

$ ls crates/rustcode-capabilities/tests/tmp_omo_skills_load.rs
ls: cannot access 'crates/rustcode-capabilities/tests/tmp_omo_skills_load.rs': No such file or directory
```

**结论：无残留新增 .rs 文件。** `git status` 与改动前完全一致（仅 `M README.md` + 2 个未跟踪 artifacts 目录），未回滚 README、未 commit。

### 3.2 验证 2：既有 skills 回归

```
cd /workspace/RustCode && cargo test -p rustcode-capabilities --lib skills
```
exit code `0`，但：
```
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 827 filtered out; finished in 0.00s
```
> ⚠️ **必须说明**：`skills` 是 opt-in feature，默认 `default = ["provider","tools"]` 不含它。裸跑 `--lib skills` 会**匹配到 0 个用例**（827 个被过滤掉），不构成回归证据。因此补跑带 feature 的版本作为真实证据：

```
cd /workspace/RustCode && cargo test -p rustcode-capabilities --features skills --lib skills
```
exit code `0`

```
test skills::catalog_hook::tests::fresh_inserts_after_leading_system_run ... ok
test skills::catalog_hook::tests::none_catalog_is_noop_on_fresh ... ok
test skills::catalog_hook::tests::resume_prunes_stale_when_no_skills_left ... ok
test skills::catalog_hook::tests::resume_refreshes_in_place_no_growth ... ok
test skills::registry::tests::discovers_nested_skills_under_grouping_dirs ... ok
test skills::registry::tests::get_bare_name_is_ambiguous_across_namespaces ... ok
test skills::registry::tests::get_bare_name_prefers_sole_user_invocable_over_hidden ... ok
test skills::registry::tests::get_resolves_namespaced_skill_by_bare_name ... ok
test skills::registry::tests::instruction_skill_reference_requires_exact_token_boundaries ... ok
test skills::registry::tests::later_dir_overrides_same_name ... ok
test skills::registry::tests::loads_flat_and_dir_skills_with_precedence ... ok
test skills::registry::tests::missing_dir_is_skipped ... ok
test skills::registry::tests::runtime_dirs_honor_rustcode_home_for_duplicate_directory_skills ... ok
test skills::registry::tests::runtime_dirs_redirect_every_user_rustcode_dir_and_leave_others ... ok
test skills::registry::tests::runtime_dirs_treat_empty_rustcode_home_as_unset ... ok
test skills::registry::tests::standard_dirs_include_agents_skills_between_claude_and_rustcode ... ok
test skills::render::tests::a_home_named_rustcode_is_unchanged ... ok
test skills::render::tests::always_emits_top_ranked_even_if_alone_over_budget ... ok
test skills::render::tests::an_empty_home_does_not_make_everything_native ... ok
test skills::render::tests::a_relocated_config_tree_still_outranks_third_party_dirs ... ok
test skills::render::tests::empty_yields_none ... ok
test skills::render::tests::explicit_instruction_reference_outranks_source_tier ... ok
test skills::render::tests::long_description_is_truncated ... ok
test skills::render::tests::over_budget_omits_lowest_rank_and_counts ... ok
test skills::render::tests::ranks_curated_before_community ... ok
test skills::render::tests::small_catalog_emits_all_with_guidance_and_no_omitted_note ... ok
test skills::render::tests::source_rank_tiers ... ok
test skills::skill::tests::appends_args_when_no_arguments_token ... ok
test skills::skill::tests::argument_containing_dollar_token_is_not_re_expanded ... ok
test skills::skill::tests::dollar_n_boundary ... ok
test skills::skill::tests::expand_arguments_full_and_positional ... ok
test skills::skill::tests::frontmatter_close_at_eof ... ok
test skills::skill::tests::frontmatter_parse ... ok
test skills::skill::tests::frontmatter_parses_user_invocable ... ok
test skills::skill::tests::frontmatter_single_quotes_and_space_tools ... ok
test skills::skill::tests::name_validation ... ok
test skills::skill::tests::no_frontmatter_is_all_body ... ok
test skills::skill::tests::out_of_range_positional_stays_literal ... ok
test skills::skill::tests::shell_injection_runs ... ok
test skills::skill::tests::variable_substitution ... ok
test skills::use_skill::tests::list_skills_empty ... ok
test skills::use_skill::tests::list_skills_formats ... ok
test skills::use_skill::tests::use_skill_directory_skill_includes_install_path_reminder ... ok
test skills::use_skill::tests::use_skill_expands ... ok
test skills::use_skill::tests::use_skill_finds_plugin_namespaced_skill ... ok
test skills::use_skill::tests::use_skill_not_found_lists_available ... ok
test skills::use_skill::tests::use_skill_plugin_namespace_shows_in_available_list ... ok
test skills::use_skill::tests::use_skill_schema_requires_an_exact_available_name ... ok
test skills::use_skill::tests::use_skill_single_file_skill_omits_install_path_reminder ... ok

test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 827 filtered out; finished in 0.01s
```

（上面 49 行为完整用例名清单，按 `sort` 输出；与 `test result` 行的 `49 passed` 一一对应。）

**结论：PASS，49/49 绿，0 跳过。**

补充编译门禁（确认临时夹具删除后 `--all-targets` 仍可编译）：
```
cd /workspace/RustCode && cargo check -p rustcode-capabilities --features skills --all-targets
EXIT=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.78s
```

### 3.3 验证 3：README 门禁复核

```
cd /workspace/RustCode && python3 scripts/check-zh-docs.py gate
```
exit code `0`

```
gate: artifacts 豁免域 `.codebuddy/artifacts/` 已排除：102 个（与 SKIP_A..SKIP_D 同级，按路径前缀判定、与基线无关；不进 AC-1 分母，也不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束；域内文件仍是 AC-8 段 2 的历史域）

gate: base=3ee655e3

PASS AC-1 清单自洽
  SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=27 ZH=167 total=248
PASS AC-2/3/4/6/7b/32 全量 check
  全量 check 受检 194，FAIL 0
  AC-4 授权放行合计 12 条（D1 11 / D2 1）
    .codebuddy/agents/doc-writer.md | removed=README.zh-CN.md | added=- | cause=D1
    AGENTS.md | removed=README.zh-CN.md | added=- | cause=D1
    AGENTS.md | removed=README.zh-CN.md | added=- | cause=D1
    AGENTS.md | removed=README.zh-CN.md:94 | added=- | cause=D1
    README.md | removed=- | added=./scripts/build-webui.sh | cause=D2
    docs/codex-claude-config-analysis.md | removed=README.zh-CN.md | added=- | cause=D1
    docs/phase1-refactor-design.md | removed=README.zh-CN.md | added=- | cause=D1
    docs/phase1-refactor-design.md | removed=README.zh-CN.md:151 | added=- | cause=D1
    docs/phase1-refactor-design.md | removed=[2] README.md / README.zh-CN.md   零遥测口径统一；清除 Emoji；补"自定义网关"配置章节 | added=[2] README.md   零遥测口径统一；清除 Emoji；补"自定义网关"配置章节 | cause=D1
    docs/platform-neutralization.md | removed=README.zh-CN.md | added=- | cause=D1
    docs/superpowers/plans/2026-05-29-webui.md | removed=README.zh-CN.md | added=- | cause=D1
    docs/superpowers/plans/2026-05-29-webui.md | removed=git add webui/ crates/ README.md README.zh-CN.md | added=git add webui/ crates/ README.md | cause=D1
PASS AC-5 运行时载荷
  AC-5a rules 目录无改动: OK
  AC-5b setup-seeds 有改动 (6 个文件): OK
  AC-5b 无 ^[+-]name: 变更: OK
PASS AC-7a 无改名/删除
  AC-7a 无 md 删除/重命名（README.zh-CN.md 与 A/C 类除外）: OK
PASS AC-8 外链残留
  段 1 正式域（候选 515，已排除 .codebuddy/artifacts/）：0 命中 -> OK
  段 2 历史域（候选 102）：命中 274 处，全部位于 .codebuddy/artifacts/ -> OK（历史痕迹仍在）
  段 3 兜底域 site/.github/docs/extensions（全扩展名）：0 命中 -> OK

--- 附录 · 被 R5 跳过的行（供抽检，共 48 行）---
  （48 行附录此处省略，均为 R5 规则跳过的既有行，其中 README.md:549 / README.md:551 两条属本次改动文件）
  ...

gate: PASS
```

```
cd /workspace/RustCode && python3 scripts/check-zh-docs.py check --files README.md
```
exit code `0`

```
PASS README.md en=0/374=0.0000
  [AC-4-authorized] README.md | removed=- | added=./scripts/build-webui.sh | cause=D2

check: 受检 1，PASS 1，FAIL 0
```

复核范围（未提交改动，未回滚、未 commit）：
```
$ git diff --stat README.md
 README.md | 20 ++++++++++----------
 1 file changed, 10 insertions(+), 10 deletions(-)
```
改动内容为「Provider 与账号」命令表 + OAuth 登录条目的口径更新（`/login` → 平台账号 / `RUSTCODE_PLATFORM_SERVER`，`/logout` / `/whoami` / `/status` 描述同步）。

**结论：PASS，两条命令均 exit 0。**

### 3.4 验证 4：工具名真实性抽查

权威清单 —— `crates/rustcode-capabilities/src/tools/` 下所有 `Tool::name()` 实现：

```
$ cd /workspace/RustCode/crates/rustcode-capabilities/src/tools
$ grep -rn -A1 'fn name(&self) -> &' --include=*.rs .
./ast_grep.rs:42            "ast_grep"
./atomgit.rs:107            "atomgit_repo"
./atomgit.rs:340            "atomgit_issue"
./atomgit.rs:526            "atomgit_api"
./atomgit.rs:686            "atomgit_pr"
./bash.rs:121               "bash"
./cd.rs:48                  "change_dir"
./edit.rs:50                "edit_file"
./glob.rs:29                "glob"
./grep.rs:47                "grep"
./list.rs:27                "list_directory"
./memory.rs:60              "memory"
./open_file.rs:137          "open_file"
./output_artifact.rs:360    "fetch_output"
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

6 个技能的 `allowed-tools` 原始清单：

```
$ cd /root/.rustcode/skills && for d in ...; do grep -m1 '^allowed-tools:' $d/SKILL.md; done
rustcode-ast-grep          ast_grep, grep, glob, read_file, search_replace, edit_file, parallel_edit_files, bash, todowrite
rustcode-debugging         bash, read_file, grep, glob, edit_file, write_file, search_replace, task, todowrite, report_finding
rustcode-git-master        bash, grep, read_file, request_user_input, todowrite
rustcode-parallel-review   task, bash, read_file, grep, glob, ast_grep, todowrite, report_finding, request_user_input
rustcode-refactor          read_file, grep, glob, ast_grep, search_replace, edit_file, parallel_edit_files, task, todowrite, bash, request_user_input
rustcode-remove-ai-slops   read_file, grep, glob, ast_grep, search_replace, edit_file, parallel_edit_files, bash, task, todowrite, report_finding, request_user_input
```

#### 已核实存在（13/13，去重后并集）

| 工具名 | 注册文件:行 | 出现在哪些技能 |
| --- | --- | --- |
| `ast_grep` | `ast_grep.rs:42` | ast-grep, parallel-review, refactor, remove-ai-slops |
| `grep` | `grep.rs:47` | 全部 6 个 |
| `glob` | `glob.rs:29` | ast-grep, debugging, parallel-review, refactor, remove-ai-slops |
| `read_file` | `read.rs:272` | 全部 6 个 |
| `search_replace` | `search_replace.rs:33` | ast-grep, debugging, refactor, remove-ai-slops |
| `edit_file` | `edit.rs:50` | ast-grep, debugging, refactor, remove-ai-slops |
| `parallel_edit_files` | `parallel_edit.rs:101` | ast-grep, refactor, remove-ai-slops |
| `bash` | `bash.rs:121` | 全部 6 个 |
| `todowrite` | `todo.rs:363` | 全部 6 个 |
| `write_file` | `write.rs:23` | debugging |
| `task` | `task.rs:669` | debugging, parallel-review, refactor, remove-ai-slops |
| `report_finding` | `report_finding.rs:77` | debugging, parallel-review, remove-ai-slops |
| `request_user_input` | `request_user_input.rs:240` | git-master, parallel-review, refactor, remove-ai-slops |

#### 未找到

**空清单 —— 0 个未找到。**

#### 前序结论复核：`cd` / `output_artifact` → `change_dir` / `fetch_output`

```
$ grep -n '^        "change_dir"'     crates/rustcode-capabilities/src/tools/cd.rs
48:        "change_dir"
$ grep -n '^        "fetch_output"'   crates/rustcode-capabilities/src/tools/output_artifact.rs
360:        "fetch_output"
```

**结论：前序报告该条结论属实（TRUE）。** 文件名是 `cd.rs` / `output_artifact.rs`，但对外的 `Tool::name()` 分别是 `change_dir` / `fetch_output`。
补充：本次 6 个技能的 `allowed-tools` 中**并未出现** `cd` 或 `output_artifact`（`grep -rn 'output_artifact\|\bcd\b' /root/.rustcode/skills/*/SKILL.md` 无命中），故该结论对本次 6 个技能**无影响**，属前序任务上下文的正确性记录。

#### 附加（超出任务书要求，但影响可用性判断）

`parallel_edit_files` / `task` / `report_finding` 三个名字**不在** `crates/rustcode-capabilities/src/tools/mod.rs::coding_tool_names()` 这个 L1 默认挂载白名单里（`mod.rs:161-175`），但确实由上层挂载：

| 工具名 | 上层挂载点 | 结论 |
| --- | --- | --- |
| `task` | `crates/rustcode-coding/src/parts.rs:743` | 真实可用 |
| `parallel_edit_files` | `crates/rustcode-coding/src/controllers.rs:550`、`crates/rustcode-review/src/assemble.rs:519` | 真实可用 |
| `report_finding` | `crates/rustcode-review/src/assemble.rs:59/546`、`crates/rustcode-clix/src/main.rs:850/1461` | 真实可用 |

即：**13 个工具名全部真实存在且在产品中挂载，无假工具名。**

---

## 4. 覆盖分析

### 4.1 已覆盖

| 路径 | 覆盖情况 |
| --- | --- |
| `load_dir` + `skills` 命名空间前缀 | ✅ t0/t1，实际 key 已 dump 后断言 |
| 目录式技能发现（`*/SKILL.md`） | ✅ 6/6 均为目录式，已隐式覆盖 |
| frontmatter 解析（name/description/allowed-tools） | ✅ t2（description 非空、长度已打印）、t4 |
| `user_invocable()` | ✅ t3 |
| `render_catalog()`（模型侧技能目录） | ✅ t3 |
| `get()` 全名 / 裸名 / 大小写不敏感 / 未知名拒绝 | ✅ t1/t5 |
| `expand()` `$ARGUMENTS` 追加行为 | ✅ t4（6 个全跑，长度增长 + 内容断言） |
| `standard_skill_dirs` / `runtime_skill_dirs` | ✅ t6（含 `RUSTCODE_HOME` 未设置时的等价性） |
| `reload()`（driver 真实入口） | ✅ t7 |
| 既有 skills 单元回归 | ✅ 49/49 |
| README 门禁 | ✅ gate + check |

### 4.2 未覆盖（含原因与负责人）

| 未覆盖项 | 原因 | 负责人 |
| --- | --- | --- |
| `parse_frontmatter` / `validate_skill_name` / `make_name` 的**直接单测**（针对这 6 个文件的具体输入） | 三者均为私有 `fn`（`skill.rs:223/297/319`），集成测试不可达；已通过 `load_dir` 的端到端行为间接覆盖（key 正确 = `make_name` + `validate_skill_name` 通过，description 正确 = `parse_frontmatter` 通过） | 无需补（既有 49 个单测已覆盖这些函数的通用语义） |
| `expand_for_injection()` 的 `<system-reminder>` 安装目录提示 | 任务书只要求 `expand`；`t4` 断言了 `expand` 行为。`expand_for_injection` 属 `use_skill` 工具路径，既有单测 `use_skill_directory_skill_includes_install_path_reminder` 已覆盖 | test-engineer（既有覆盖已足够） |
| TUI / CLI / daemon / headless / ACP / clix 等**宿主入口的端到端技能调用** | 本次改动仅新增 6 个 SKILL.md 数据文件 + README，不改生产代码、不改 runtime 生命周期；`t7` 已验证所有宿主共享的 `reload()` 入口 | 无需补 |
| `allowed-tools` 的**运行时强制**（L1 明确不校验，属 L2 审批策略） | 按设计如此（`skill.rs:17-19` 注释）；本项只做名字真实性静态核对 | 无需补 |
| 持久化兼容（旧格式只读导入） | 本次不涉及持久化 | 无需补 |
| 取消 / 重试 / 并发路径 | 加载器是同步、无 cancel/no-retry/no-concurrency 的纯函数式扫描；无这些路径可覆盖 | 无需补 |

---

## 5. 回归结论

| 入口 | 是否受影响 | 是否验证 | 结论 |
| --- | --- | --- | --- |
| CLI / TUI / daemon / headless / background / ACP / clix | 否（本次仅新增 6 个数据文件 + README 文案，零生产代码改动） | 共享的 `SkillRegistry::reload()` 入口已在 t7 验证；其余入口无变更面 | 无需逐入口回归，`git status` 证明无生产代码改动 |
| `rustcode-capabilities` crate | 是（测试夹具曾临时落在该 crate） | 是 | `cargo check -p rustcode-capabilities --features skills --all-targets` exit 0；`--lib skills` 49/49 绿 |
| 中文文档门禁 | 是（README 未提交改动） | 是 | gate + check 双 exit 0 |

---

## 6. 失败与阻塞

### 6.1 失败用例

**无。** 0 failed / 0 ignored / 无跳过。

### 6.2 非阻塞观察项（建议转 doc-writer，不阻塞发布）

| # | 现象 | 证据 | 影响 | 建议归属 |
| --- | --- | --- | --- | --- |
| OBS-1 | `rustcode-debugging/SKILL.md` 的 `description` 内转义引号未被还原，解析后字面量保留反斜杠 | 原始 `sed -n '3p'` 输出：`或说\\\"为什么 X 不工作\\\"\\\"帮我定位这个 bug\\\"时使用`；运行时 `desc_len=171`，dump 显示 `或说\\\"为什么 X 不工作\\\"` | 仅影响模型侧技能目录里该技能描述的**显示美观**（多出 `\`），不影响加载、不影响 `use_skill`、不影响模板执行 | doc-writer（改法：去掉 `\"` 转义，改用中文引号「」或不加引号）。**本次只记录未改** |

> 依据任务书：禁止修改 6 个 SKILL.md 的内容。OBS-1 仅记录，交编排者决定是否回退 doc-writer。

### 6.3 阻塞项

无。环境（Linux / zsh / HOME=/root / RUSTCODE_HOME 未设置）具备完整验证条件，无「以跳过冒充通过」的情形。

---

## 7. 附：本次未修改任何生产代码 / 未 commit 的证明

```
$ git status --short        # 验证前
 M README.md
?? .codebuddy/artifacts/2026-09-09-omo-skills-import/
?? .codebuddy/artifacts/2026-09-09-readme-account-cmds/

$ git status --short        # 验证后（完全一致）
 M README.md
?? .codebuddy/artifacts/2026-09-09-omo-skills-import/
?? .codebuddy/artifacts/2026-09-09-readme-account-cmds/

$ git status --porcelain | grep -E '\.rs$' || echo "(no .rs entries in git status)"
(no .rs entries in git status)

$ git rev-parse --abbrev-ref HEAD
dev
```

- 6 个 SKILL.md：**未修改**（全程只读 `sed`/`grep`）。
- 生产源码：**未修改**。
- 临时测试夹具：已删除，磁盘与 git 均无残留。
- git commit：**未执行**；README 改动与 artifacts 目录保持原状。
