# Contributing / 参与贡献

Thanks for helping! Bug reports, translations and pull requests are all welcome.
欢迎提问题、补翻译、提 PR。

## Development / 开发

Windows with Rust (stable, MSVC), Node.js 20.19+ and Python 3:

```bat
cd crates\mousetroke-settings\ui && npm ci && cd ..\..\..
scripts\build.cmd            :: UI + release exe → dist\mousetroke.exe
cargo test --workspace       :: all tests
```

Close other mouse-gesture tools before running a dev build — two tools hooking the right
button will fight each other. 运行开发版前先退出其他鼠标手势软件，否则会双重触发。

## Guidelines / 约定

- **Engine changes**: add a state-machine test in `crates/mousetroke-win/src/engine_tests.rs`
  (fake canvas and host, no real mouse needed) before trying it on a real machine.
  引擎改动先在 `engine_tests.rs` 写状态机测试。
- **UI text**: every user-visible string lives in `i18n/<lang>.json`; `scripts/i18n_lint.py --check`
  fails the build on hard-coded text. See `i18n/README.md` for adding strings or languages.
  界面文字一律走词条，见 `i18n/README.md`。
- **Built-in gestures**: `crates/mousetroke-core/assets/default-config.json`. Keep the
  [website](https://mousetroke.pages.dev/) in sync in its separate repository when they change.
  产品主页在独立仓库中维护和部署；内置手势改动要同步网站仓库。
- **Security**: never log key codes, typed text, clipboard or selected text. Start system programs
  by full path (`input::system_exe`). 日志不能记录按键、文字、剪贴板；系统程序用完整路径启动。

## Translations / 翻译

Copy `i18n/en-US.json` to `i18n/<tag>.json`, translate the values, and follow the steps in
`i18n/README.md`. The pseudo-locale (`qps-ploc`) helps find untranslated text.
