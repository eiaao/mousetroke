use mousetroke_core::action_datetime::LocalDateTime;

pub fn local_now() -> LocalDateTime {
    let t = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
    LocalDateTime {
        year: t.wYear,
        month: t.wMonth as u8,
        day: t.wDay as u8,
        hour: t.wHour as u8,
        minute: t.wMinute as u8,
        second: t.wSecond as u8,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn system_local_time_can_be_formatted_without_desktop() {
        assert!(
            mousetroke_core::action_datetime::format_now("datetime", super::local_now()).is_ok()
        );
    }
}
