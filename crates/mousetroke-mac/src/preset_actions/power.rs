fn pmset(argument: &str) -> Result<(), String> {
    let output = std::process::Command::new("/usr/bin/pmset")
        .arg(argument)
        .output()
        .map_err(|e| format!("Start pmset {argument}: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "pmset {argument} failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

/// 阻塞调用，交给动作工作线程执行。
pub fn display_off() -> Result<(), String> {
    pmset("displaysleepnow")
}
pub fn sleep() -> Result<(), String> {
    pmset("sleepnow")
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore] // 在真实 Mac 桌面单独运行；显示器关闭后移动鼠标唤醒。
    fn desktop_display_off() {
        super::display_off().unwrap();
    }

    #[test]
    #[ignore] // 保存工作后单独运行；系统会睡眠，需要手动唤醒。
    fn desktop_sleep() {
        super::sleep().unwrap();
    }
}
