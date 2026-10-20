//! 可在 Windows 验证的 macOS 纯逻辑，不调用系统 API。
#[cfg(target_os = "macos")]
use crate::overlay::geometry::Rect;
#[cfg(not(target_os = "macos"))]
use crate::overlay_geometry::Rect;
use mousetroke_core::vars::{self, Vars};

#[derive(Clone, Debug)]
pub struct Window {
    pub pid: u32,
    pub wid: usize,
    pub layer: i32,
    pub bounds: Rect,
}

pub fn hit_test(windows: &[Window], x: f64, y: f64) -> Option<&Window> {
    windows
        .iter()
        .find(|w| w.layer == 0 && w.bounds.contains(x, y))
}

pub fn covers(window: Rect, screen: Rect) -> bool {
    (window.x - screen.x).abs() <= 2.0
        && (window.y - screen.y).abs() <= 2.0
        && (window.w - screen.w).abs() <= 2.0
        && (window.h - screen.h).abs() <= 2.0
}

/// 作用域身份使用 bundle id；无 bundle 的命令行程序回退到可执行文件路径。
pub fn identity(
    _bundle: Option<String>,
    executable: Option<String>,
    id: String,
) -> (Option<String>, String) {
    (if id.is_empty() { executable } else { Some(id.clone()) }, id)
}

pub fn menu_point(x: f64, y: f64, primary_height: f64) -> (f64, f64) {
    (x, primary_height - y)
}

pub fn media_code(key: &str) -> Option<u16> {
    Some(match key.to_ascii_uppercase().as_str() {
        "PLAY" | "PLAY_PAUSE" | "MEDIA_PLAY" => 16,
        "NEXT" | "MEDIA_NEXT" => 17,
        "PREVIOUS" | "MEDIA_PREVIOUS" | "MEDIA_PREV" => 18,
        "MUTE" | "VOLUME_MUTE" => 7,
        "VOLUME_UP" | "SOUND_UP" => 0,
        "VOLUME_DOWN" | "SOUND_DOWN" => 1,
        "BRIGHTNESS_UP" => 2,
        "BRIGHTNESS_DOWN" => 3,
        _ => return None,
    })
}

pub fn media_data1(code: u16, down: bool) -> isize {
    ((code as isize) << 16) | ((if down { 0xA } else { 0xB }) << 8)
}

/// LaunchApp.args 使用 shell 风格引号分词，但不执行 shell 展开或替换。
pub fn arguments(text: &str) -> Option<Vec<String>> {
    let mut args = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut escape = false;
    let mut started = false;
    for ch in text.chars() {
        if escape {
            word.push(ch);
            escape = false;
        } else if ch == '\\' && quote != Some('\'') {
            escape = true;
            started = true;
        } else if quote == Some(ch) {
            quote = None;
        } else if quote.is_none() && matches!(ch, '\'' | '"') {
            quote = Some(ch);
            started = true;
        } else if quote.is_none() && ch.is_whitespace() {
            if started {
                args.push(std::mem::take(&mut word));
                started = false;
            }
        } else {
            word.push(ch);
            started = true;
        }
    }
    if escape || quote.is_some() {
        return None;
    }
    if started {
        args.push(word);
    }
    Some(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn window_selection_is_front_to_back_layer_zero_and_half_open() {
        let bounds = Rect {
            x: -100.0,
            y: 50.0,
            w: 100.0,
            h: 80.0,
        };
        let rows: Vec<_> = [9, 0, 0]
            .into_iter()
            .enumerate()
            .map(|(i, layer)| Window {
                pid: 42,
                wid: i,
                layer,
                bounds,
            })
            .collect();
        assert_eq!(hit_test(&rows, -50.0, 70.0).unwrap().wid, 1);
        assert!(hit_test(&rows, 0.0, 70.0).is_none());
        assert_eq!(hit_test(&rows, -100.0, 50.0).unwrap().pid, 42);
        assert!(covers(bounds, bounds));
        assert!(!covers(Rect { w: 90.0, ..bounds }, bounds));
    }
    #[test]
    fn bundle_identity_matches_existing_scopes() {
        let (exe, class) = identity(
            Some("/Applications/Google Chrome.app".into()),
            Some("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".into()),
            "com.google.Chrome".into(),
        );
        let exe = exe.unwrap();
        assert_eq!(class, "com.google.Chrome");
        assert_eq!(exe, "com.google.Chrome");
        assert!(mousetroke_core::matcher::exe_matches(
            "com.google.Chrome",
            &exe
        ));
        assert_eq!(
            identity(None, Some("/bin/tool".into()), String::new())
                .0
                .as_deref(),
            Some("/bin/tool")
        );
    }
    #[test]
    fn media_encoding_and_menu_coordinates() {
        assert_eq!(media_data1(16, true), 0x100A00);
        assert_eq!(media_data1(16, false), 0x100B00);
        for (name, code) in [
            ("PLAY", 16),
            ("NEXT", 17),
            ("PREVIOUS", 18),
            ("MUTE", 7),
            ("SOUND_UP", 0),
            ("SOUND_DOWN", 1),
            ("BRIGHTNESS_UP", 2),
            ("BRIGHTNESS_DOWN", 3),
        ] {
            assert_eq!(media_code(name), Some(code));
        }
        assert_eq!(media_code("unknown"), None);
        assert_eq!(menu_point(-200.0, -100.0, 900.0), (-200.0, 1000.0));
        assert_eq!(menu_point(100.0, 1200.0, 900.0), (100.0, -300.0));
    }
    #[test]
    fn variable_expansion_and_launch_arguments() {
        let values = Vars::from([("MS_PID".into(), "42".into())]);
        assert_eq!(
            vars::expand("/bin/kill -9 %MS_PID%", &values),
            "/bin/kill -9 42"
        );
        assert_eq!(
            arguments("one 'two three' \"\""),
            Some(vec!["one".into(), "two three".into(), "".into()])
        );
        assert!(arguments("'unterminated").is_none());
    }
}
