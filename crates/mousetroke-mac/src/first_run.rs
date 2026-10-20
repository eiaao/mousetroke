//! 从临时挂载位置启动时只提示一次，避免把系统权限绑定到临时路径。
pub fn needs_install(path: &std::path::Path) -> bool {
    let path = path.to_string_lossy();
    path.starts_with("/Volumes/") || path.contains("/AppTranslocation/")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn temporary_locations_require_copying_to_applications() {
        for path in ["/Volumes/Mousetroke/Mousetroke.app/Contents/MacOS/mousetroke",
            "/private/var/folders/x/AppTranslocation/UUID/d/Mousetroke.app/Contents/MacOS/mousetroke"] {
            assert!(needs_install(std::path::Path::new(path)));
        }
        for path in [
            "/Applications/Mousetroke.app/Contents/MacOS/mousetroke",
            "/Users/me/Applications/Mousetroke.app/Contents/MacOS/mousetroke",
            "/Users/me/Volumes/example",
            "/tmp/AppTranslocation-backup/mousetroke",
        ] {
            assert!(!needs_install(std::path::Path::new(path)));
        }
    }
}
