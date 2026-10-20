//! 新包中的独立助手等待旧进程退出，随后回收旧包并交给 LaunchServices 启动。
use super::{
    install::{run, trash},
    Error,
};
use crate::update_helpers::{bundle_root, UpdatePaths};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
}

fn wait_for_previous(pids: &[u32]) -> Result<(), Error> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let mut running = false;
        for &pid in pids.iter().filter(|&&pid| pid != 0) {
            if pid > i32::MAX as u32 || pid == std::process::id() {
                return Err(Error::Restart);
            }
            // 安全：signal=0 只观察进程，不发信号；只有 ESRCH=3 确认进程不存在。
            if unsafe { kill(pid as i32, 0) } == 0 {
                running = true;
            } else if std::io::Error::last_os_error().raw_os_error() != Some(3) {
                return Err(Error::Restart);
            }
        }
        if !running {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(Error::Restart);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn open(bundle: &Path, config: &Path) -> Result<(), Error> {
    run(
        Command::new("/usr/bin/open")
            .arg("-n")
            .arg(bundle)
            .args(["--args", "--config"])
            .arg(config)
            .stdin(Stdio::null()),
        Error::Restart,
    )
    .map(|_| ())
}

/// 入口在任何引擎/设置单实例锁之前调用；bundle 路径只由助手自己的可执行文件推导。
pub fn relaunch(engine: u32, settings: u32, config: &Path, ready: &Path) -> Result<(), Error> {
    let _ = crate::log::init();
    let result = objc2::rc::autoreleasepool(|_| {
        if settings == 0
            || settings > i32::MAX as u32
            || engine > i32::MAX as u32
            || settings == std::process::id()
            || engine == std::process::id()
        {
            return Err(Error::Restart);
        }
        let exe = std::env::current_exe()
            .and_then(|p| p.canonicalize())
            .map_err(|_| Error::Restart)?;
        let paths = UpdatePaths::new(bundle_root(&exe)?)?;
        if !paths.old.is_dir() {
            return Err(Error::Restart);
        }
        // 写就绪文件后不再依赖临时目录；父进程完成交接后会立即清理它。
        std::fs::write(ready, std::process::id().to_string()).map_err(|_| Error::Restart)?;
        crate::log!("更新助手等待引擎与设置进程退出：{engine}, {settings}");
        if let Err(error) = wait_for_previous(&[engine, settings]) {
            crate::log!("等待旧进程失败或超时，回滚应用包：{error:?}");
            paths.rollback()?;
            let _ = trash(&paths.staging);
            return Err(error);
        }
        let previous = match trash(&paths.old) {
            Ok(previous) => previous,
            Err(error) => {
                crate::log!("旧包无法移到废纸篓，回滚并启动旧版");
                paths.rollback()?;
                let _ = trash(&paths.staging);
                open(&paths.bundle, config)?;
                return Err(error);
            }
        };
        crate::log!("旧进程已退出，启动新版：{}", paths.bundle.display());
        if let Err(error) = open(&paths.bundle, config) {
            crate::log!("启动新版失败，从废纸篓恢复旧包");
            paths.restore_from(&previous)?;
            let _ = trash(&paths.staging);
            open(&paths.bundle, config)?;
            return Err(error);
        }
        crate::log!("更新重启已交给系统，旧包位于：{}", previous.display());
        Ok(())
    });
    if let Err(error) = result {
        crate::log!("更新重启失败：{error:?}");
    }
    crate::log::shutdown();
    result
}
