//! macOS M1 平台层：AppKit、输入 tap 与引擎均在主线程运行。
#![cfg(target_os = "macos")]
#![deny(unsafe_op_in_unsafe_fn)]

mod app;
mod events;
mod host;
mod instance;
pub mod log;
mod menu;
mod permissions;
mod selftest;

pub use app::run;
pub use host::system_langs;

#[cfg(test)]
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
        child.stdin.take().unwrap().write_all(xml.as_bytes()).unwrap();
        assert!(child.wait().unwrap().success());
    }
}
