//! 锁守卫显式解锁；不删除文件，避免不同 inode 同时上锁。
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
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
    let Some(mut lock) = acquire_named(dir, "mousetroke.lock")? else {
        return Ok(None);
    };
    // 锁文件不能替换 inode；只在取得锁时记录 PID，供设置页排除上个实例的快照。
    lock.0.set_len(0)?;
    write!(lock.0, "{}", std::process::id())?;
    Ok(Some(lock))
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
        .is_some_and(|dir| held_engine_lock(dir).is_some())
}

/// 只读探测现有锁，不创建目录、锁文件或改写 PID。
fn held_engine_lock(dir: &Path) -> Option<File> {
    let file = File::open(dir.join("mousetroke.lock")).ok()?;
    // 安全：fd 在调用期间有效；与 acquire_named 使用同一种非阻塞 flock。
    if unsafe { flock(file.as_raw_fd(), 2 | 4) } == 0 {
        // 取得锁意味着引擎已退出；关闭这个独立 fd 即释放探测锁。
        return None;
    }
    (io::Error::last_os_error().kind() == io::ErrorKind::WouldBlock).then_some(file)
}

/// 极小的状态快照供设置页判断事件 tap 是否已生效；临时文件 + rename 保证读不到半行。
pub(crate) fn publish_input_state(active: bool) {
    static PUBLISHER: std::sync::Mutex<crate::instance_state::Publisher> =
        std::sync::Mutex::new(crate::instance_state::Publisher::new());
    let path = crate::default_config_path();
    let Some(dir) = path.parent() else { return };
    let mut publisher = PUBLISHER.lock().unwrap_or_else(|error| error.into_inner());
    if let Err(error) = publisher.publish(dir, std::process::id(), active) {
        crate::log!("写入输入状态失败: {error}");
    }
}

/// 同一用户的本机进程通信；只接受当前锁持有者的快照，不需要定期写入心跳。
pub fn input_state() -> Option<(u32, bool)> {
    let path = crate::default_config_path();
    input_state_in(path.parent()?)
}

fn input_state_in(dir: &Path) -> Option<(u32, bool)> {
    let mut lock = held_engine_lock(dir)?;
    let mut owner = String::new();
    lock.read_to_string(&mut owner).ok()?;
    crate::instance_state::read(dir, owner.parse().ok())
}

/// 从锁文件读取当前引擎 PID；通知发送不依赖输入快照是否已发布。
pub fn engine_pid() -> Option<u32> {
    let path = crate::default_config_path();
    let mut lock = held_engine_lock(path.parent()?)?;
    let mut owner = String::new();
    lock.read_to_string(&mut owner).ok()?;
    owner.parse().ok().filter(|pid| *pid != 0)
}

pub fn request_permission(accessibility: bool) -> io::Result<()> {
    let pid = engine_pid().ok_or(io::ErrorKind::NotConnected)?;
    crate::control::send(pid, if accessibility {
        crate::control::Request::Accessibility
    } else {
        crate::control::Request::InputMonitoring
    })
}

pub fn request_restart() -> io::Result<()> {
    let pid = engine_pid().ok_or(io::ErrorKind::NotConnected)?;
    crate::control::send(pid, crate::control::Request::Restart)
}

/// 更新请求只控制当前锁持有者；退出和重启共用无文件轮询的通知通道。
pub fn request_quit(pid: u32) -> io::Result<()> {
    crate::control::send(pid, crate::control::Request::Quit)
}

#[cfg(test)]
mod tests {
    use std::os::{fd::AsRawFd, unix::fs::MetadataExt};

    extern "C" {
        fn fcntl(fd: i32, command: i32, ...) -> i32;
    }

    #[test]
    fn snapshots_require_the_current_lock_owner_without_probe_writes() {
        let dir =
            std::env::temp_dir().join(format!("mousetroke-state-lock-{}", std::process::id()));
        assert!(super::held_engine_lock(&dir).is_none());
        assert!(!dir.exists());
        let engine = super::acquire(&dir).unwrap().unwrap();
        let pid = std::process::id();
        let lock_path = dir.join("mousetroke.lock");
        assert_eq!(
            std::fs::read_to_string(&lock_path).unwrap(),
            pid.to_string()
        );
        let mut publisher = crate::instance_state::Publisher::new();
        publisher.publish(&dir, pid.wrapping_add(1), true).unwrap();
        assert_eq!(super::input_state_in(&dir), None);
        publisher.publish(&dir, pid, true).unwrap();
        assert_eq!(super::input_state_in(&dir), Some((pid, true)));
        let modified = std::fs::metadata(&lock_path).unwrap().modified().unwrap();
        drop(engine);
        assert_eq!(super::input_state_in(&dir), None);
        assert!(super::held_engine_lock(&dir).is_none());
        assert_eq!(
            std::fs::metadata(&lock_path).unwrap().modified().unwrap(),
            modified
        );
        std::fs::remove_dir_all(dir).unwrap();
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
