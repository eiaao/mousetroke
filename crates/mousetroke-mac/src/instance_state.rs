//! 输入状态只在变化后落盘；实例锁的持有者 PID 决定快照是否有效。
use std::{io, path::Path};

#[derive(Default)]
pub(crate) struct Publisher(Option<(u32, bool)>);

impl Publisher {
    pub(crate) const fn new() -> Self {
        Self(None)
    }

    pub(crate) fn publish(&mut self, dir: &Path, pid: u32, active: bool) -> io::Result<()> {
        if self.0 == Some((pid, active)) {
            return Ok(());
        }
        let temporary = dir.join("input-state.tmp");
        std::fs::write(&temporary, format!("{pid} {}\n", u8::from(active)))
            .and_then(|()| std::fs::rename(&temporary, dir.join("input-state")))?;
        // 写入失败时保留旧缓存，让现有的权限检查在下次重试。
        self.0 = Some((pid, active));
        Ok(())
    }
}

pub(crate) fn read(dir: &Path, owner: Option<u32>) -> Option<(u32, bool)> {
    let owner = owner?;
    let text = std::fs::read_to_string(dir.join("input-state")).ok()?;
    let mut parts = text.split_whitespace();
    let pid = parts.next()?.parse().ok()?;
    let active = match parts.next()? {
        "1" => true,
        "0" => false,
        _ => return None,
    };
    (pid != 0 && pid == owner && parts.next().is_none()).then_some((pid, active))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, FileTimes};
    use std::time::SystemTime;

    #[test]
    fn publishes_only_at_startup_or_when_pid_or_tap_changes() {
        let dir =
            std::env::temp_dir().join(format!("mousetroke-input-state-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("input-state");
        let mut publisher = Publisher::new();
        publisher.publish(&dir, 42, false).unwrap();
        assert_eq!(read(&dir, Some(42)), Some((42, false)));
        File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(FileTimes::new().set_modified(SystemTime::UNIX_EPOCH))
            .unwrap();
        let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
        for _ in 0..100 {
            publisher.publish(&dir, 42, false).unwrap();
        }
        assert_eq!(
            std::fs::metadata(&path).unwrap().modified().unwrap(),
            modified
        );
        // 很久没有变化仍然有效，不能依赖时间戳或心跳写入。
        assert_eq!(read(&dir, Some(42)), Some((42, false)));
        publisher.publish(&dir, 42, true).unwrap();
        assert_eq!(read(&dir, Some(42)), Some((42, true)));
        publisher.publish(&dir, 43, true).unwrap();
        assert_eq!(read(&dir, Some(43)), Some((43, true)));
        assert_eq!(read(&dir, Some(42)), None);
        assert_eq!(read(&dir, None), None);
        // 新实例首次发布，即使状态与已有文件相同，也重写一次。
        std::fs::write(&path, "old snapshot").unwrap();
        Publisher::default().publish(&dir, 43, true).unwrap();
        assert_eq!(read(&dir, Some(43)), Some((43, true)));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failed_publish_retries_and_invalid_snapshots_are_ignored() {
        let dir =
            std::env::temp_dir().join(format!("mousetroke-input-retry-{}", std::process::id()));
        let path = dir.join("input-state");
        std::fs::create_dir_all(&path).unwrap();
        let mut publisher = Publisher::default();
        assert!(publisher.publish(&dir, 42, true).is_err());
        std::fs::remove_dir(&path).unwrap();
        publisher.publish(&dir, 42, true).unwrap();
        assert_eq!(read(&dir, Some(42)), Some((42, true)));
        for text in ["", "42", "42 2", "other 1", "0 1", "42 1 extra", "43 1"] {
            std::fs::write(&path, text).unwrap();
            assert_eq!(read(&dir, Some(42)), None, "{text}");
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}
