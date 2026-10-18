@echo off
rem Mousetroke 一键构建：前端 → mousetroke.exe（引擎与设置界面同一个 exe，release）→ dist\
setlocal
cd /d "%~dp0.."

echo [1/3] 构建设置界面前端
pushd crates\mousetroke-settings\ui
if not exist node_modules call npm install || goto :fail
call npm run build || goto :fail
popd

echo [2/3] 编译 mousetroke.exe（含设置界面）
cargo build --release -p mousetroke || goto :fail

echo [3/3] 汇总到 dist\
if not exist dist mkdir dist
copy /y target\release\mousetroke.exe dist\ >nul || goto :fail
rem 旧版的独立设置程序已合并进 mousetroke.exe
if exist dist\mousetroke-settings.exe del /q dist\mousetroke-settings.exe 2>nul
for %%F in (dist\mousetroke.exe) do echo   %%~nxF  %%~zF bytes
echo 完成。运行 dist\mousetroke.exe 启动手势，托盘图标左键打开设置。
exit /b 0

:fail
echo 构建失败。
exit /b 1
