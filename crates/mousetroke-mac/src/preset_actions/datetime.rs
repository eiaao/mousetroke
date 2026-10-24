use mousetroke_core::action_datetime::LocalDateTime;
use std::{
    ffi::{c_char, c_long},
    time::{SystemTime, UNIX_EPOCH},
};

// Darwin 的 struct tm；本地时区和夏令时由系统 localtime_r 处理。
#[repr(C)]
#[derive(Default)]
struct Tm {
    second: i32,
    minute: i32,
    hour: i32,
    day: i32,
    month: i32,
    year: i32,
    weekday: i32,
    yearday: i32,
    is_dst: i32,
    utc_offset: c_long,
    zone: *const c_char,
}
unsafe extern "C" {
    fn localtime_r(time: *const i64, result: *mut Tm) -> *mut Tm;
}

pub fn local_now() -> Result<LocalDateTime, String> {
    let timestamp: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Read system time: {e}"))?
        .as_secs()
        .try_into()
        .map_err(|_| "System time out of range")?;
    let mut tm = Tm::default();
    // 安全：Darwin time_t 为 64 位；输出结构布局与系统 struct tm 一致。
    if unsafe { localtime_r(&timestamp, &mut tm) }.is_null() {
        return Err(format!(
            "Read local time: {}",
            std::io::Error::last_os_error()
        ));
    }
    let result = LocalDateTime {
        year: (tm.year + 1900)
            .try_into()
            .map_err(|_| "Invalid local year")?,
        month: (tm.month + 1)
            .try_into()
            .map_err(|_| "Invalid local month")?,
        day: tm.day.try_into().map_err(|_| "Invalid local day")?,
        hour: tm.hour.try_into().map_err(|_| "Invalid local hour")?,
        minute: tm.minute.try_into().map_err(|_| "Invalid local minute")?,
        second: tm.second.try_into().map_err(|_| "Invalid local second")?,
    };
    if result.valid() {
        Ok(result)
    } else {
        Err("System returned invalid local time".into())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn local_time_does_not_require_desktop() {
        assert!(mousetroke_core::action_datetime::format_now(
            "datetime",
            super::local_now().unwrap()
        )
        .is_ok());
    }
}
