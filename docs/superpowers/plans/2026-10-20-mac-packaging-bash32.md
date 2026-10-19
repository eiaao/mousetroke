# macOS Bash 3.2 / Python 3.9 Packaging Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复第三轮评审发现的 macOS 打包门禁，并保证系统 Bash 3.2 与 Python 3.9 可用。

**Architecture:** 保留现有脚本和隔离工具替身；对失败显式退出，在函数中显式返回。测试优先使用 `/bin/bash`，静态检查覆盖所有 Mac shell 与工作流片段。

**Tech Stack:** Bash 3.2、Python 3.9 标准库、GitHub Actions、Rust workspace。

**Spec:** 本次用户任务（第三轮评审，feat/mac-m5a）。

## Global Constraints

- 只修改当前 worktree；仅一次本地提交，不 push。
- `CARGO_TARGET_DIR=D:\project\reaWGestures2\.claude\worktrees\target-mac-m5a`，`CARGO_BUILD_JOBS=4`。
- 提交首行中文，包含 `Co-Authored-By: Codex <noreply@openai.com>`。
- 用户已授权直接执行；由当前 agent 完成，无需计划审批或额外执行技能。

## Review Focus

- 多余、重复、缺失和空架构必须阻断签名与镜像创建。
- 函数在条件/OR 列表中调用时，内部错误仍须传播。
- 工具查询、管道及命令替换失败不能被后续成功命令掩盖。
- 空钥匙串数组在 `set -u` 下可用，保留原钥匙串读取失败须停止。
- 测试替身本身须遵守 Bash 3.2；LF 写入和脚本导入支持 Python 3.9。

---

### Task 1: Shell failure propagation and compatibility

**Files:** `scripts/mac/{bundle,dmg,sign,messages,make-dev-cert,make-release-cert}.sh`、`.github/workflows/{ci,release,cloudflare}.yml`。

**Interfaces:** 保留现有参数、环境变量和签名策略，显式传播失败；Mac 工作流指定系统 Bash。

- [x] 审计每个脚本及所有工作流 `run:`，修复检查、查询和空数组。
- [x] 补上架构、签名函数、工具失败以及证书流程回归测试。

### Task 2: Python baseline and regression guards

**Files:** `scripts/test_mac_packaging.py`、`scripts/changelog.py`、`scripts/test_changelog.py`。

**Interfaces:** 保留 unittest 入口和 changelog CLI；无额外 Python 包依赖。

- [x] 优先 `/bin/bash`，修复替身 LF 写入，增加 Bash 4+ 静态门禁。
- [x] 检查全部 `scripts/*.py`，去除 Mac 可用脚本中的 Python 3.10+ API 依赖。

### Task 3: Validation and local commit

**Files:** 上述文件及本计划。

- [x] 对所有 Mac shell 和提取的工作流 Bash 执行 `bash -n`；运行打包与 changelog 测试。
- [x] 使用可用的 Bash 3.2 / Python 3.9 运行时验证；无法取得时明确记录限制。
- [x] 按指定环境运行 `cargo test --workspace --locked`。
- [x] 检查 diff、完成审计清单、一次提交并确认未 push。

## 第三轮审计清单

| 位置 | 审计结果与修复 |
| --- | --- |
| `scripts/mac/bundle.sh` | 路径与版本查询、参数检查、构建/lipo、精确双架构谓词、plist 函数及预览、签名与完成消息均显式传播失败；失败清理保留退出码。 |
| `scripts/mac/dmg.sh` | 输入包、架构查询及完整集合检查、逐片证书元数据、签名验证、复制/创建/镜像验证均显式退出；临时目录和失败镜像清理聚合错误。 |
| `scripts/mac/sign.sh` | 密码读取不再嵌入工具参数；证书/身份查询先检查状态再解析，移除条件管道和上游 SIGPIPE 风险；签名及验证即使在条件中调用也显式返回失败；可选空数组采用 3.2 兼容展开。 |
| `scripts/mac/messages.sh` | `local` 与命令替换分开，awk 失败或缺少消息键返回失败，条件调用不能吞掉错误。 |
| `scripts/mac/make-dev-cert.sh` | 路径、目录权限、随机密码、证书生成/导入和钥匙串操作显式检查；先捕获钥匙串查询结果，替换无法传递失败的进程替换；普通 `if` 过滤及空数组安全展开；清理失败不被后续命令掩盖。 |
| `scripts/mac/make-release-cert.sh` | 路径与 Git 查询分别检查；密码输入 EOF、空值、不一致均明确退出；临时目录、证书生成/导出、消息与清理状态显式传播。 |
| `.github/workflows/ci.yml` | macos-engine 全部 6 段 `run:`：UI 构建测试、应用构建、core/engine/mac 测试、settings 测试、打包门禁、通用包/DMG 冒烟；固定系统 Bash，打包子进程也显式 `/bin/bash`。Windows PowerShell 非 Bash，另行确认无需 Bash 兼容修改。 |
| `.github/workflows/release.yml` | macos 全部 8 段 `run:`：Secrets 门禁、UI、打包测试、targets、Rust 测试、身份导入、通用签名包、清理；检查随机密码和 base64 管道，逐个检查 security；清理的空数组/读取/删除错误保留最终失败。Ubuntu 发布的校验和管道与 gh 最终退出状态亦已审计。Windows PowerShell/CMD 不属于 Bash。 |
| `.github/workflows/cloudflare.yml` | 全部 5 段 Bash：凭据、站点检查、准备、查建项目、部署。条件管道改为先捕获并检查 wrangler 查询，避免查询错误被当成“项目不存在”。 |
| `.github/workflows/pages.yml` | 全部 2 段 Bash：检查与准备；各只有单条外部命令，退出状态直接传递，无隐藏谓词或 Bash 4 特性。 |
| `scripts/test_mac_packaging.py` | POSIX 优先 `/bin/bash`；工具替身的谓词显式退出，写入使用 Python 3.9 支持的 `open(newline=...)`；隔离 HOME；检测常见 Bash 4+ 特性、声明掩盖命令替换、裸谓词，覆盖 shell/工作流语法与失败路径。 |
| `scripts/changelog.py`、`scripts/test_changelog.py` | 去除 Python 3.11 专属 tomllib，按现有清单约定仅提取 workspace.package.version；验证节作用域、注释及缺少版本，维持零第三方依赖。 |
| `scripts/gen_icon.py`、`scripts/i18n_lint.py`、`scripts/update-test-server.py` | 标准库/API 与语法审计，均无需修改；全部 6 个 Python 文件经 3.9 语法检查，Mac 相关模块经 3.9 导入检查。 |

## 验证记录与边界

- `cargo test --workspace --locked`：按指定 Windows target 目录和 4 个任务执行，通过。
- Python 3.9.25 + Git Bash 5.2：原生 Windows 全部回归通过；之后扩展了身份导入成功及 base64/下游工具失败回归。
- Python 3.9.25 + GNU Bash 3.2.57：WSL/Linux 内使用 GNU 源码构建的 Bash，脚本和替身全部由该版本执行；最终 29 项打包/兼容性测试及 4 项 changelog 测试通过。
- `bash -n`：6 个 Mac shell、工具替身、22 段工作流 Bash 全部通过；14 段 macOS `run:` 及全部 Mac shell 通过常见 Bash 4+ 特性静态门禁。
- 全部运行时下载、编译文件和测试输出位于当前 worktree 的忽略目录 `target/`；不提交这些临时内容。
- 当前宿主仍为 Windows，Apple 工具使用隔离替身。真实 Apple 构建、签名和 DMG 行为由 macOS CI/审查机复验；本轮已直接验证真实 Bash 3.2 的退出行为与 Python 3.9 API。
