//! 锁文件一直持有到进程退出；不删除文件，避免不同 inode 同时上锁。
use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;
use std::path::Path;

extern "C" {
    fn flock(fd: i32, operation: i32) -> i32;
}

pub fn acquire(dir: &Path) -> io::Result<Option<File>> {
    acquire_named(dir, "mousetroke.lock")
}

/// 两种进程各持有自己的锁文件，绝不删除或替换已经上锁的 inode。
pub fn acquire_named(dir: &Path, name: &str) -> io::Result<Option<File>> {
    std::fs::create_dir_all(dir)?;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(dir.join(name))?;
    // 安全：fd 在调用期间有效；Darwin 的 LOCK_EX=2、LOCK_NB=4，不等待其他实例。
    if unsafe { flock(file.as_raw_fd(), 2 | 4) } == 0 {
        Ok(Some(file))
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
    path.parent().is_some_and(|dir| matches!(acquire(dir), Ok(None)))
}

#[cfg(test)]
mod tests {
    #[test]
    fn locks_are_independent_and_released_without_unlinking() {
        let dir = std::env::temp_dir().join(format!("mousetroke-flock-{}", std::process::id()));
        let engine = super::acquire(&dir).unwrap().unwrap();
        assert!(super::acquire(&dir).unwrap().is_none());
        let settings = super::acquire_named(&dir, "settings.lock").unwrap().unwrap();
        assert!(super::acquire_named(&dir, "settings.lock").unwrap().is_none());
        drop(engine);
        assert!(super::acquire(&dir).unwrap().is_some());
        assert!(dir.join("mousetroke.lock").exists());
        drop(settings);
        std::fs::remove_file(dir.join("settings.lock")).unwrap();
        std::fs::remove_file(dir.join("mousetroke.lock")).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
