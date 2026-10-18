//! 程序图标：exe → 32×32 PNG（data URL），供作用域列表与程序选择器显示。
//!
//! 作用域里常常只存了文件名（`chrome.exe`），先按 完整路径 → 运行中的同名进程 →
//! 注册表 App Paths → PATH 的顺序找到真实路径，再用系统 API 取图标。

use std::path::{Path, PathBuf};

use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS, HGDIOBJ,
};
use windows::Win32::UI::Shell::SHDefExtractIconW;
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

const SIZE: u32 = 32;

/// 找到 exe 的完整路径。
pub fn resolve_exe(exe: &str) -> Option<PathBuf> {
    let p = Path::new(exe);
    if p.is_absolute() {
        return p.exists().then(|| p.to_path_buf());
    }
    let name = mousetroke_core::matcher::file_name(exe);
    if let Some(path) = mousetroke_win::targeting::running_exe_path(name) {
        return Some(PathBuf::from(path));
    }
    if let Some(path) = app_paths(name) {
        return Some(path);
    }
    std::env::var_os("PATH").and_then(|paths| std::env::split_paths(&paths).map(|d| d.join(name)).find(|c| c.exists()))
}

/// 注册表 `…\CurrentVersion\App Paths\<name>` 的默认值（大多数安装程序会登记）。
fn app_paths(name: &str) -> Option<PathBuf> {
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};
    let key: Vec<u16> = format!(r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\{name}").encode_utf16().chain([0]).collect();
    for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        let mut buf = [0u16; 1024];
        let mut len = (buf.len() * 2) as u32;
        let ok = unsafe {
            RegGetValueW(root, PCWSTR(key.as_ptr()), PCWSTR::null(), RRF_RT_REG_SZ, None, Some(buf.as_mut_ptr().cast()), Some(&mut len))
        };
        if ok.is_ok() {
            let n = buf.iter().position(|&c| c == 0).unwrap_or(0);
            let path = PathBuf::from(String::from_utf16_lossy(&buf[..n]).trim_matches('"'));
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

/// exe 的图标（`data:image/png;base64,…`）；取不到返回 None。
pub fn icon_data_url(exe: &str) -> Option<String> {
    let path = resolve_exe(exe)?;
    let rgba = extract_rgba(&path)?;
    Some(format!("data:image/png;base64,{}", base64(&encode_png(&rgba, SIZE, SIZE)?)))
}

fn extract_rgba(path: &Path) -> Option<Vec<u8>> {
    let wide: Vec<u16> = path.as_os_str().to_string_lossy().encode_utf16().chain([0]).collect();
    let mut icon = HICON::default();
    unsafe { SHDefExtractIconW(PCWSTR(wide.as_ptr()), 0, 0, Some(&mut icon), None, SIZE) }.ok().ok()?;
    if icon.is_invalid() {
        return None;
    }
    let out = icon_to_rgba(icon);
    unsafe {
        let _ = DestroyIcon(icon);
    }
    out
}

/// HICON → RGBA（非预乘）。老式无 alpha 的图标用掩码补透明。
fn icon_to_rgba(icon: HICON) -> Option<Vec<u8>> {
    unsafe {
        let mut info = ICONINFO::default();
        GetIconInfo(icon, &mut info).ok()?;
        let read = |bmp: windows::Win32::Graphics::Gdi::HBITMAP| -> Option<(Vec<u8>, i32, i32)> {
            let mut bm = BITMAP::default();
            if GetObjectW(HGDIOBJ(bmp.0), std::mem::size_of::<BITMAP>() as i32, Some((&mut bm as *mut BITMAP).cast())) == 0 {
                return None;
            }
            let (w, h) = (bm.bmWidth, bm.bmHeight);
            let mut bi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w,
                    biHeight: -h,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut buf = vec![0u8; (w * h * 4) as usize];
            let dc = CreateCompatibleDC(None);
            let n = GetDIBits(dc, bmp, 0, h as u32, Some(buf.as_mut_ptr().cast()), &mut bi, DIB_RGB_COLORS);
            let _ = DeleteDC(dc);
            (n > 0).then_some((buf, w, h))
        };
        let color = if info.hbmColor.is_invalid() { None } else { read(info.hbmColor) };
        let mask = read(info.hbmMask);
        let _ = DeleteObject(HGDIOBJ(info.hbmColor.0));
        let _ = DeleteObject(HGDIOBJ(info.hbmMask.0));
        let (mut px, w, h) = color?;
        if (w, h) != (SIZE as i32, SIZE as i32) {
            return None;
        }
        let has_alpha = px.chunks(4).any(|p| p[3] != 0);
        for (i, p) in px.chunks_mut(4).enumerate() {
            p.swap(0, 2); // BGRA → RGBA
            if !has_alpha {
                let transparent = mask.as_ref().is_some_and(|(m, _, _)| m[i * 4] != 0);
                p[3] = if transparent { 0 } else { 255 };
            }
        }
        Some(px)
    }
}

fn encode_png(rgba: &[u8], w: u32, h: u32) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().ok()?.write_image_data(rgba).ok()?;
    Some(out)
}

fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            s.push(if i <= c.len() { T[(n >> (18 - 6 * i)) as usize & 63] as char } else { '=' });
        }
    }
    s
}

#[cfg(test)]
mod tests {
    #[test]
    fn base64_matches_reference() {
        assert_eq!(super::base64(b"Man"), "TWFu");
        assert_eq!(super::base64(b"Ma"), "TWE=");
        assert_eq!(super::base64(b"M"), "TQ==");
    }

    #[test]
    fn explorer_icon_resolves_by_name() {
        let url = super::icon_data_url("explorer.exe").expect("explorer.exe 应能取到图标");
        assert!(url.starts_with("data:image/png;base64,iVBOR"));
    }
}

#[cfg(test)]
mod preview {
    /// 手动查看：`cargo test -p mousetroke-settings icon_preview -- --ignored`，输出到 target/icons/。
    #[test]
    #[ignore]
    fn icon_preview() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/icons");
        std::fs::create_dir_all(&dir).unwrap();
        for exe in ["chrome.exe", "explorer.exe", "cmd.exe", "notepad.exe", "WindowsTerminal.exe", "Code.exe", "uTools.exe"] {
            let path = super::resolve_exe(exe);
            let rgba = path.as_deref().and_then(super::extract_rgba);
            println!("{exe}: {:?} {}", path, rgba.is_some());
            if let Some(px) = rgba {
                std::fs::write(dir.join(format!("{exe}.png")), super::encode_png(&px, 32, 32).unwrap()).unwrap();
            }
        }
    }
}
