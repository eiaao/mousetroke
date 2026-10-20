//! 锁守卫显式解锁；不删除文件，避免不同 inode 同时上锁。
use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;
use std::path::Path;

extern "C" {
    fn flock(fd: i32, operation: i32) -> i32;
}

/// 只由当前实例拥有的锁；不暴露 File，避免克隆句柄改变锁的生命周期。
pub struct InstanceLock(File);

impl Drop for InstanceLock {
    fn drop(&mut self) {
        // fork 后、exec 前，子进程可能暂时持有同一 open file description。
        // 仅关闭父进程的 fd 不一定释放 flock；LOCK_UN=8 显式释放共享的锁。
        // 安全：File 在 Drop 返回后才关闭，fd 在调用期间有效。
        let _ = unsafe { flock(self.0.as_raw_fd(), 8) };
    }
}

pub fn acquire(dir: &Path) -> io::Result<Option<InstanceLock>> {
    acquire_named(dir, "mousetroke.lock")
}

/// 两种进程各持有自己的锁文件，绝不删除或替换已经上锁的 inode。
pub fn acquire_named(dir: &Path, name: &str) -> io::Result<Option<InstanceLock>> {
    std::fs::create_dir_all(dir)?;
    // Rust 的 OpenOptions 在 Unix 上原子设置 O_CLOEXEC，exec 后不会泄漏锁 fd。
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(dir.join(name))?;
    // 安全：fd 在调用期间有效；Darwin 的 LOCK_EX=2、LOCK_NB=4，不等待其他实例。
    if unsafe { flock(file.as_raw_fd(), 2 | 4) } == 0 {
        Ok(Some(InstanceLock(file)))
    } else {
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::WouldBlock {
            Ok(None)
        } else {
            Err(error)
        }
    }
}

pub fn engine_running() -> bool {
    let path = crate::default_config_path();
    path.parent()
        .is_some_and(|dir| matches!(acquire(dir), Ok(None)))
}

/// 极小的状态快照供设置页判断事件 tap 是否已生效；临时文件 + rename 保证读不到半行。
pub(crate) fn publish_input_state(active: bool) {
    let path = crate::default_config_path();
    let Some(dir) = path.parent() else { return };
    let temporary = dir.join("input-state.tmp");
    let result = std::fs::write(
        &temporary,
        format!("{} {}\n", std::process::id(), u8::from(active)),
    )
    .and_then(|()| std::fs::rename(&temporary, dir.join("input-state")));
    if let Err(error) = result {
        crate::log!("写入输入状态失败: {error}");
    }
}

/// 同一用户的本机进程通信；过期快照不能让设置页把已退出的引擎显示为就绪。
pub fn input_state() -> Option<(u32, bool)> {
    if !engine_running() {
        return None;
    }
    let path = crate::default_config_path().parent()?.join("input-state");
    let age = std::fs::metadata(&path)
        .ok()?
        .modified()
        .ok()?
        .elapsed()
        .ok()?;
    if age > std::time::Duration::from_secs(6) {
        return None;
    }
    let text = std::fs::read_to_string(path).ok()?;
    let mut parts = text.split_whitespace();
    let pid = parts.next()?.parse().ok()?;
    let active = match parts.next()? {
        "1" => true,
        "0" => false,
        _ => return None,
    };
    Some((pid, active))
}

pub fn request_restart() -> io::Result<()> {
    let (pid, _) = input_state().ok_or_else(|| io::Error::from(io::ErrorKind::NotConnected))?;
    let path = crate::default_config_path();
    let dir = path
        .parent()
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    std::fs::write(dir.join("restart-request"), pid.to_string())
}

pub(crate) fn take_restart_request() -> bool {
    let path = crate::default_config_path();
    let Some(dir) = path.parent() else {
        return false;
    };
    let request = dir.join("restart-request");
    if std::fs::read_to_string(&request).is_ok_and(|pid| pid == std::process::id().to_string()) {
        return std::fs::remove_file(request).is_ok();
    }
    false
}

#[cfg(test)]
mod tests {
    use std::os::{fd::AsRawFd, unix::fs::MetadataExt};

    extern "C" {
        fn fcntl(fd: i32, command: i32, ...) -> i32;
    }

    #[test]
    fn locks_are_independent_and_released_without_unlinking() {
        let dir = std::env::temp_dir().join(format!("mousetroke-flock-{}", std::process::id()));
        let engine = super::acquire(&dir).unwrap().unwrap();
        let engine_inode = std::fs::metadata(dir.join("mousetroke.lock"))
            .unwrap()
            .ino();
        assert!(super::acquire(&dir).unwrap().is_none());
        let settings = super::acquire_named(&dir, "settings.lock")
            .unwrap()
            .unwrap();
        let settings_inode = std::fs::metadata(dir.join("settings.lock")).unwrap().ino();
        assert!(super::acquire_named(&dir, "settings.lock")
            .unwrap()
            .is_none());
        for lock in [&engine, &settings] {
            // 安全：锁的 fd 有效；Darwin 的 F_GETFD=1、FD_CLOEXEC=1，无可变参数。
            let flags = unsafe { fcntl(lock.0.as_raw_fd(), 1) };
            assert_ne!(flags, -1);
            assert_ne!(flags & 1, 0, "instance lock must be close-on-exec");
        }
        // dup 与 fork 继承的 fd 共享同一 open file description；保留它们，
        // 确定性覆盖并发 bundle_tests 启动子进程时的 fork/exec 间隙。
        let inherited_engine = engine.0.try_clone().unwrap();
        let inherited_settings = settings.0.try_clone().unwrap();
        drop(engine);
        let next_engine = super::acquire(&dir).unwrap().unwrap();
        assert!(super::acquire(&dir).unwrap().is_none());
        assert!(super::acquire_named(&dir, "settings.lock")
            .unwrap()
            .is_none());
        assert_eq!(
            std::fs::metadata(dir.join("mousetroke.lock"))
                .unwrap()
                .ino(),
            engine_inode
        );
        drop(settings);
        let next_settings = super::acquire_named(&dir, "settings.lock")
            .unwrap()
            .unwrap();
        assert!(super::acquire_named(&dir, "settings.lock")
            .unwrap()
            .is_none());
        assert!(super::acquire(&dir).unwrap().is_none());
        assert_eq!(
            std::fs::metadata(dir.join("settings.lock")).unwrap().ino(),
            settings_inode
        );
        drop(inherited_engine);
        drop(inherited_settings);
        assert!(super::acquire(&dir).unwrap().is_none());
        assert!(super::acquire_named(&dir, "settings.lock")
            .unwrap()
            .is_none());
        drop(next_engine);
        drop(next_settings);
        std::fs::remove_file(dir.join("settings.lock")).unwrap();
        std::fs::remove_file(dir.join("mousetroke.lock")).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
