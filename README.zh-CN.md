# Mousetroke

[![CI](https://github.com/eiaao/mousetroke/actions/workflows/ci.yml/badge.svg)](https://github.com/eiaao/mousetroke/actions/workflows/ci.yml) [![Release](https://img.shields.io/github/v/release/eiaao/mousetroke?include_prereleases)](https://github.com/eiaao/mousetroke/releases/latest) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE) ![Windows 10/11](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-0098d9)

**免费开源的 Windows 鼠标手势工具。** 按住右键画一笔：`←` 后退，`↓→` 关闭标签页，`↙` 最小化窗口。
鼠标推到屏幕左下角打开开始菜单，贴着上边滚动滚轮调音量。只是单击右键时，菜单照常弹出。

**[产品主页](https://eiaao.github.io/mousetroke/) · [下载](https://github.com/eiaao/mousetroke/releases/latest) · [English](README.md)**

![Mousetroke 设置界面](docs/images/settings-zh.png)

- 内置五十多个手势，全部可改；在设置里画一遍就能录一个自己的
- 屏幕边角：开始菜单、显示桌面、音量、亮度（外接显示器走 DDC/CI）
- 八种动作：发送按键、窗口操作、打开程序、打开网址、运行命令、媒体与系统、弹出菜单、Lua 脚本
- 按程序分别设置，或在远程桌面、3D、设计软件里整体禁用手势
- 全屏游戏、视频、演示时自动暂停；`Win+Esc` 随时暂停
- 常驻内存约 19 MB；单个 exe（约 13 MB）免安装；界面支持 10 种语言

## 安装

1. 从 [最新版本](https://github.com/eiaao/mousetroke/releases/latest) 下载 `mousetroke.exe`，并用同页的 `SHA256SUMS.txt` 核对校验值。
2. 双击运行。任务栏右下角出现图标：左键打开设置，右键暂停或退出。第一次运行有简短的使用引导。

系统要求：Windows 10 1809 及以上或 Windows 11，64 位。设置界面使用 Microsoft Edge WebView2（Windows 11 自带）。
exe 暂未做代码签名，首次运行时 SmartScreen 可能提示，点 **更多信息 → 仍要运行**。

卸载：在偏好设置里关掉开机自启，右键托盘图标选「退出」，再删除 `mousetroke.exe` 和 `%APPDATA%\mousetroke` 文件夹。

## 从源码构建

在 Windows 上需要 Rust（stable，MSVC 工具链）、Node.js 20.19+、Python 3。

```bat
scripts\build.cmd            :: 前端 + release exe → dist\mousetroke.exe
cargo test --workspace       :: 引擎、导入器、多语言与设置界面的测试
```

目录说明见 [README.md](README.md#build-from-source)。

## 安全

"运行命令"和 Lua 脚本拥有你的用户权限，只导入可信来源的配置。程序做了什么、怎么报告漏洞，见 [SECURITY.md](SECURITY.md)。

## 参与贡献

欢迎提问题、补翻译、提 PR，见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

[MIT](LICENSE)
