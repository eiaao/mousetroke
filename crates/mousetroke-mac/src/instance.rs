//! 锁文件一直持有到进程退出；不删除文件，避免不同 inode 同时上锁。
use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;
use std::path::Path;

extern "C" {
    fn flock(fd: i32, operation: i32) -> i32;
}

pub fn acquire(dir: &Path) -> io::Result<Option<File>> {
    std::fs::create_dir_all(dir)?;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(dir.join("mousetroke.lock"))?;
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
