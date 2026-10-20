//! macOS M3b 平台层：AppKit、tap、覆盖层与引擎在主线程，AX/进程/Lua 在工作线程。
#![deny(unsafe_op_in_unsafe_fn)]

/// 供导出的 `log!` 宏引用，调用方无需直接依赖引擎 crate。
#[cfg(target_os = "macos")]
#[doc(hidden)]
pub use mousetroke_engine as __engine;

#[cfg(target_os = "macos")]
mod actions;
#[cfg(target_os = "macos")]
mod ax;
#[cfg(any(target_os = "macos", test))]
mod helpers;
#[cfg(any(target_os = "macos", test))]
mod hot_corners;
#[cfg(target_os = "macos")]
mod lua;
#[cfg(target_os = "macos")]
mod targeting;

#[cfg(target_os = "macos")]
mod app;
#[cfg(target_os = "macos")]
mod events;
#[cfg(target_os = "macos")]
mod host;
#[cfg(target_os = "macos")]
mod input;
#[cfg(target_os = "macos")]
pub mod instance;
#[cfg(target_os = "macos")]
pub mod log;
#[cfg(target_os = "macos")]
mod menu;
#[cfg(target_os = "macos")]
mod overlay;
#[cfg(target_os = "macos")]
mod permissions;
#[cfg(target_os = "macos")]
mod selftest;

#[cfg(target_os = "macos")]
pub use app::run;
#[cfg(target_os = "macos")]
pub use host::system_langs;
#[cfg(target_os = "macos")]
pub use mousetroke_core::schema::default_config_path;
#[cfg(target_os = "macos")]
pub use app::run_with_config;
#[cfg(target_os = "macos")]
pub use targeting::{ignored_bundle, pick_app_at};
#[cfg(target_os = "macos")]
mod settings;

#[cfg(all(test, not(target_os = "macos")))]
#[path = "overlay/geometry.rs"]
mod overlay_geometry;

#[cfg(all(test, target_os = "macos"))]
mod bundle_tests {
    #[test]
    fn generated_plist_has_bundle_identity_and_workspace_version() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let result = std::process::Command::new("bash")
            .arg(root.join("scripts/mac/bundle.sh"))
            .arg("--print-plist")
            .output()
            .unwrap();
        assert!(result.status.success());
        let xml = String::from_utf8(result.stdout).unwrap();
        for value in [
            "io.github.eiaao.mousetroke",
            "<key>LSUIElement</key><true/>",
            "<key>LSMinimumSystemVersion</key><string>11.0</string>",
            env!("CARGO_PKG_VERSION"),
        ] {
            assert!(xml.contains(value), "{value}");
        }
        assert!(!xml.contains("@VERSION@"));
        use std::io::Write;
        let mut child = std::process::Command::new("plutil")
            .args(["-lint", "--", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(xml.as_bytes())
            .unwrap();
        assert!(child.wait().unwrap().success());
    }
}

#[cfg(any(target_os = "macos", test))]
mod first_run;
#[cfg(target_os = "macos")]
pub mod update;
