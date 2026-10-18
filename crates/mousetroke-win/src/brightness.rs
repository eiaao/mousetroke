//! 显示器亮度：外接显示器走 DDC/CI（dxva2），笔记本内屏回落到 WMI。
//!
//! 两条路径都可能不可用（显示器关了 DDC、虚拟机等），失败返回 `None`。
//! WMI 通过 PowerShell 调用（约数百毫秒），仅作回落；探测结果按显示器缓存，
//! 连续滚轮调节时不会每次都先试一遍 DDC。

use std::collections::HashMap;
use std::sync::Mutex;

use windows::Win32::Devices::Display::{
    DestroyPhysicalMonitors, GetMonitorBrightness, GetNumberOfPhysicalMonitorsFromHMONITOR,
    GetPhysicalMonitorsFromHMONITOR, SetMonitorBrightness, PHYSICAL_MONITOR,
};
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{MonitorFromPoint, HMONITOR, MONITOR_DEFAULTTONEAREST};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Backend {
    Ddc,
    Wmi,
    None,
}

static BACKENDS: Mutex<Option<HashMap<isize, Backend>>> = Mutex::new(None);

fn cached(m: HMONITOR) -> Option<Backend> {
    BACKENDS.lock().ok()?.as_ref()?.get(&(m.0 as isize)).copied()
}

fn remember(m: HMONITOR, b: Backend) {
    if let Ok(mut g) = BACKENDS.lock() {
        g.get_or_insert_with(HashMap::new).insert(m.0 as isize, b);
    }
}

/// 读取 (x, y) 所在显示器亮度（0..=100）。
pub fn get(x: i32, y: i32) -> Option<u32> {
    adjust(x, y, 0)
}

/// 调整 (x, y) 所在显示器亮度 `delta` 个百分点，返回新的亮度（0..=100）。
pub fn adjust(x: i32, y: i32, delta: i32) -> Option<u32> {
    let mon = unsafe { MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST) };
    match cached(mon) {
        Some(Backend::Ddc) => ddc(mon, delta),
        Some(Backend::Wmi) => wmi(delta),
        Some(Backend::None) => None,
        None => {
            let (r, b) = match ddc(mon, delta) {
                Some(v) => (Some(v), Backend::Ddc),
                None => match wmi(delta) {
                    Some(v) => (Some(v), Backend::Wmi),
                    None => (None, Backend::None),
                },
            };
            remember(mon, b);
            r
        }
    }
}

fn ddc(mon: HMONITOR, delta: i32) -> Option<u32> {
    unsafe {
        let mut n = 0u32;
        GetNumberOfPhysicalMonitorsFromHMONITOR(mon, &mut n).ok()?;
        if n == 0 {
            return None;
        }
        let mut phys = vec![PHYSICAL_MONITOR::default(); n as usize];
        GetPhysicalMonitorsFromHMONITOR(mon, &mut phys).ok()?;
        let mut result = None;
        for p in &phys {
            let (mut min, mut cur, mut max) = (0u32, 0u32, 0u32);
            if GetMonitorBrightness(p.hPhysicalMonitor, &mut min, &mut cur, &mut max) == 0 || max <= min {
                continue;
            }
            // 按百分比换算到显示器自己的量程
            let range = (max - min) as f64;
            let pct = ((cur - min) as f64 / range * 100.0).round() as i32;
            let target = (pct + delta).clamp(0, 100);
            if delta != 0 {
                let raw = min + (target as f64 / 100.0 * range).round() as u32;
                if SetMonitorBrightness(p.hPhysicalMonitor, raw) == 0 {
                    continue;
                }
            }
            result = Some(target as u32);
        }
        let _ = DestroyPhysicalMonitors(&phys);
        result
    }
}

fn wmi(delta: i32) -> Option<u32> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let script = format!(
        "$c=(Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness -ErrorAction Stop|Select-Object -First 1).CurrentBrightness;\
         $n=[Math]::Max(0,[Math]::Min(100,[int]$c+({delta})));\
         if({delta} -ne 0){{Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods|Select-Object -First 1|\
         Invoke-CimMethod -MethodName WmiSetBrightness -Arguments @{{Timeout=[uint32]0;Brightness=[byte]$n}}|Out-Null}};$n"
    );
    let out = std::process::Command::new(crate::input::system_exe(r"System32\WindowsPowerShell\v1.0\powershell.exe"))
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}
