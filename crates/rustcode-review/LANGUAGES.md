# RustCode Review —— 支持的语言与文件类型

本文档列出 `rustcode-review` 依据 diff 中改动的文件、自动注入 reviewer system prompt 的内置评审规则。

规则按每个改动文件的**小写基名**匹配。具体文件名先于通用扩展名模式检查，第一个命中即生效。

无需重新编译即可在运行时覆盖任意内置规则：把名为 `<rule>.md` 的文件放进某个目录，再用 `--rules-dir <dir>` 传给评审命令。

## 编程语言

| 语言 | 规则 | 文件匹配模式 |
| --- | --- | --- |
| ArkTS 语言 | `arkts` | `*.ets` |
| C 语言 | `c` | `*.c`, `*.h` |
| C++ 语言 | `cpp` | `*.cc`, `*.cpp`, `*.cxx`, `*.hpp` |
| C# 语言 | `csharp` | `*.cs` |
| Cangjie (仓颉) | `cangjie` | `*.cj` |
| Clojure 语言 | `clojure` | `*.clj`, `*.cljs`, `*.cljc` |
| Dart 语言 | `dart` | `*.dart` |
| Elixir 语言 | `elixir` | `*.ex`, `*.exs` |
| Erlang 语言 | `erlang` | `*.erl`, `*.hrl` |
| Go 语言 | `go` | `*.go` |
| Groovy 语言 | `groovy` | `*.groovy` |
| Haskell 语言 | `haskell` | `*.hs` |
| Java 语言 | `java` | `*.java` |
| Kotlin 语言 | `kotlin` | `*.kt`, `*.kts` |
| Lua 语言 | `lua` | `*.lua` |
| Objective-C 语言 | `objc` | `*.m`, `*.mm` |
| Perl 语言 | `perl` | `*.pl`, `*.pm` |
| PHP 语言 | `php` | `*.php` |
| Python 语言 | `python` | `*.py` |
| R 语言 | `r` | `*.r` |
| Ruby 语言 | `ruby` | `*.rb` |
| Rust 语言 | `rust` | `*.rs` |
| Scala 语言 | `scala` | `*.scala` |
| Shell 脚本 | `shell` | `*.sh`, `*.bash` |
| Solidity 语言 | `solidity` | `*.sol` |
| SQL 语言 | `sql` | `*.sql` |
| Swift 语言 | `swift` | `*.swift` |
| TypeScript / Web 前端 | `ts` | `*.js`, `*.jsx`, `*.ts`, `*.tsx`, `*.mjs`, `*.vue` |

## Web 与标记语言

| 类别 | 规则 | 文件匹配模式 |
| --- | --- | --- |
| HTML 文档 | `html` | `*.html`, `*.htm` |
| CSS 样式表 | `css` | `*.css`, `*.scss`, `*.sass`, `*.less` |
| GraphQL 查询 | `graphql` | `*.graphql`, `*.gql` |
| Markdown 文档 | `markdown` | `readme*.md`, `*.md`, `*.markdown` |

## 构建工具与依赖清单

| 工具 / 文件 | 规则 | 文件匹配模式 |
| --- | --- | --- |
| Gradle 构建文件 | `build_gradle` | `build.gradle` |
| Maven 的 POM 文件 | `pom_xml` | `pom.xml` |
| npm 包清单 | `package_json` | `package.json` |
| CMake 构建文件 | `cmake` | `cmakelists.txt`, `*.cmake` |
| Makefile 构建文件 | `makefile` | `makefile`, `gnumakefile`, `*.mk` |
| Dockerfile 镜像构建文件 | `dockerfile` | `dockerfile`, `dockerfile.*`, `*.dockerfile` |
| Python 依赖清单 | `python_deps` | `requirements*.txt`, `pyproject.toml`, `setup.py`, `setup.cfg`, `pipfile` |

## 数据、配置与基础设施

| 类别 | 规则 | 文件匹配模式 |
| --- | --- | --- |
| JSON 数据文件 | `json` | `*.json`, `*.json5` |
| YAML 配置文件 | `yaml` | `*.yaml`, `*.yml` |
| TOML 配置文件 | `toml` | `*.toml` |
| XML 文件 | `xml` | `*.xml` |
| Properties 配置文件 | `properties` | `*.properties` |
| Protobuf 接口定义 | `protobuf` | `*.proto` |
| Terraform 基础设施 | `terraform` | `*.tf`, `*.tfvars` |
| MyBatis / DAO 映射 XML | `mapper_dao_xml` | `*mapper*.xml`, `*dao*.xml` |

## 匹配流程

1. 解析 diff 中的 `+++ b/<path>` 行，得到改动文件列表。
2. 把每个文件的基名转为小写，再到匹配表里逐一检查。
3. 为该文件选中第一个命中的规则。
4. 文件按规则分组；每条规则文档只渲染一次，并在作用域内列出命中的文件名。

示例：一段同时改动 `src/main/java/Foo.java` 与 `web/App.vue` 的 diff，会注入 `java` 与 `ts` 两条规则，各自只作用于对应的文件。

## 运行时定制

无需重新编译即可调整某条规则：

```bash
mkdir -p ./my-rules
cp crates/rustcode-review/rules/cangjie.md ./my-rules/cangjie.md
# edit ./my-rules/cangjie.md
rustcode review --rules-dir ./my-rules
```

只有目录中实际存在的规则会被覆盖；其余规则继续使用内置版本。

## 新增一门语言

要新增一条内置语言规则：

1. 新建 `crates/rustcode-review/rules/<name>.md`，写入评审检查清单。
2. 在 `crates/rustcode-review/src/rules.rs` 的 `MATCHERS` 中注册文件模式。
3. 在同一文件的 `RULE_DOCS` 下注册该规则文档。
4. 运行 `cargo test -p rustcode-review`，确认匹配器与文档的一致性测试通过。
