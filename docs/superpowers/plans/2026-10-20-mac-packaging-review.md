# macOS Packaging Review Fix Implementation Plan

> **For agentic workers:** Implement the authorized review fixes in this worktree; steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 Apple lipo 的单架构校验兼容性，严格限定 arm64/x86_64，并阻止旧的、单架构或签名不合格的产物进入发布 DMG。

**Architecture:** bundle 在构建前删除旧 app，失败时清除半成品；DMG 在校验前删除旧镜像，所有模式要求通用二进制和有效签名，发布模式额外逐架构要求证书签名。CI 显式允许 ad-hoc，发布工作流始终要求证书签名。

**Tech Stack:** Bash、Apple lipo/codesign/hdiutil、Python unittest、GitHub Actions。

**Spec:** 本次 feat/mac-m5a 审查修复任务。

## Global Constraints

- 只修改当前 worktree，保留 feat/mac-m5a；一次本地提交，不推送。
- Cargo 使用 `CARGO_TARGET_DIR=D:\project\reaWGestures2\.claude\worktrees\target-mac-m5a`、`CARGO_BUILD_JOBS=4`。
- 提交首行中文，包含 `Co-Authored-By: Codex <noreply@openai.com>`。

## Review Focus

- 构建失败后旧 app 必须消失，打包失败后旧或半成品 DMG 必须消失。
- `lipo` 每次 `-verify_arch` 只传一个架构；`-archs` 必须恰好包含 arm64 和 x86_64，顺序不限。
- 签名验证失败不能创建 DMG；发布不能接受任一架构的 ad-hoc 签名。
- 开发/CI 通用 ad-hoc 包可打包，正式证书通用包可发布。
- 路径包含空格时仍正确引用；plist 预览不删除产物。

### Task 1: 修复打包门禁与调用方

**Files:** `scripts/mac/{bundle,dmg}.sh`、`.github/workflows/{ci,release}.yml`、`crates/mousetroke-mac/README.md`。

**Interfaces:** 保留 `bundle.sh [--universal]` 和 `dmg.sh`；`MOUSETROKE_REQUIRE_SIGNING=1` 为发布门禁。

- [x] 核对 Xcode/Apple man page 的 lipo、codesign、hdiutil 参数约定；修复 `-verify_arch`。
- [x] 构建/打包开始清理旧产物，失败时清理半成品；DMG 校验双架构、完整签名及发布证书身份。
- [x] CI 显式关闭强制证书签名，release job 显式开启；更新使用说明。

### Task 2: 验证和交付

**Files:** `scripts/test_mac_packaging.py`。

**Interfaces:** `python -m unittest discover -s scripts -p test_mac_packaging.py`，用临时隔离目录和替身命令验证失败门禁；不调用真实签名工具。

- [x] 回归覆盖 Review Focus 中的失败与成功路径，检查失败不会继续创建镜像。
- [x] 对所有 shell 脚本执行 `bash -n`；运行完整 locked workspace 测试和站点占位符检查。
- [x] 审查 diff，确认唯一一次本地提交使用中文首行和指定 co-author。

### Task 3: 第二轮实机兼容性修复

**Files:** `scripts/mac/{bundle,dmg}.sh`、`scripts/test_mac_packaging.py`、`.github/workflows/{ci,release}.yml`。

**Interfaces:** 保留打包脚本入口；fake 的 `-verify_arch` 只接受一个架构，`-archs` 返回 `MOCK_SLICES`，CI/release 在打包前运行回归测试。

- [x] 将 fake 改为拒绝多个架构；正向断言两次独立校验和一次架构列表查询。先运行现有脚本，确认回归失败。
- [x] bundle 和 DMG 使用 `for architecture in arm64 x86_64`，再断言 `lipo -archs` 的完整列表恰好为这两个架构。
- [x] 覆盖架构顺序相反、单架构、额外/重复架构、空列表、列表查询失败；失败不留下 app/DMG，也不继续签名或创建镜像。
- [x] 对所有脚本和工作流 Bash 运行 `bash -n`，运行打包回归和指定 Cargo 测试；审查 diff 后一次本地提交。

## 核对与验证记录

- 对照 Xcode 附带的 Apple 手册镜像：[lipo(1)](https://keith.github.io/xcode-man-pages/lipo.1.html)、[codesign(1)](https://keith.github.io/xcode-man-pages/codesign.1.html)、[hdiutil(1)](https://keith.github.io/xcode-man-pages/hdiutil.1.html)。
- 第二轮实机验证推翻了上一轮的多架构参数假设：macOS 27 CommandLineTools 的 `lipo -verify_arch` 每次只接受一个架构；必须逐片校验，并以 `-archs` 额外检查完整架构集合。`-create` 的多输入及 `-output` 用法不变。
- `codesign` 的签名、校验、显示操作均为选项后接路径；`--verbose=2/4`、`--timestamp=none` 与显示 requirement 的 `-r-` 用法有效。校验默认覆盖所有架构，显示默认只看宿主架构，因此发布身份逐片检查。
- `hdiutil create` 的镜像路径和格式/卷名/源目录选项有效，`verify` 后接镜像有效；移除 `-ov`，显式删除旧文件后创建。
- 第一轮：10 项回归测试通过；7 个 shell 脚本和 20 段工作流 Bash 命令语法通过；`cargo test --workspace --locked` 与 `python site/check.py --allow-mac-size-placeholder` 通过。
- 第二轮：严格 fake 复现旧调用的实机错误；修复后 15 项打包回归、7 个 shell 脚本和 22 段工作流 Bash 语法检查、`cargo test --workspace --locked` 全部通过。Cargo 使用约定的独立 target 目录及 4 个构建任务。
- 当前为 Windows 环境，Apple 工具行为由隔离替身验证；真实通用构建、签名与 DMG 创建仍由 macOS CI/审查机复验。
