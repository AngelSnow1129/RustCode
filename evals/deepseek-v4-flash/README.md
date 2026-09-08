# DeepSeek V4 Flash 配对评测

本评测框架通过彼此独立的 RustCode headless 运行时，对比 RustCode 与火山引擎的
DeepSeek V4 Flash 模型配置。同一配对中的候选运行并发启动，且绝不共享可写会话
或 fixture。运行使用 `--ephemeral --output-format jsonl`；模型层用例另外使用
`--no-tools`。

前置条件：Python 3.7+、已构建或已安装的 `rustcode` 与 `codex`，以及一份已存在、
且 `benchmark.json` 中同时包含两个选型 ID 的 RustCode 配置。凭据始终留在常规的
RustCode 认证/配置存储中；本目录绝不会把它们复制进结果产物。

```bash
cd evals/deepseek-v4-flash
python3 eval.py prepare
python3 eval.py run --run-dir results/<run-id>
python3 eval.py judge --run-dir results/<run-id>
python3 eval.py report --run-dir results/<run-id>
```

用 `--case smoke-model --repetitions 1` 做一次低成本预检。`run` 只会对声明了
`allow_edits = true` 的用例追加 `--dangerously-skip-permissions`；这类用例请
只使用可丢弃的 fixture。评估当前工作区时，把 `rustcode_bin` 设为
`../../target/debug/rustcode`。

用例目录包含 `case.json` 和 `prompt.md`。可选的 `fixture` 会为每个候选/重复
各复制一份。可选的 `verify` 是一个参数数组，以避免 shell 插值。结果产物包含重建后的
助手输出、脱敏后的 stderr、元数据、校验输出、匿名 Codex 数据包、判定结果、
`summary.json`，以及最终的 `report.md`。`events.jsonl` 是权威的
RustCode 事件流。请勿提交 `results/`。
