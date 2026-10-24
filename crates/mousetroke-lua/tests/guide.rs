//! Keep the public guides synchronized with the runtime and parser.
use mlua::{Function, Lua, Table, Value};
use mousetroke_core::keyseq::{self, Action, Key, KeySeq};
use mousetroke_lua::{install_ms_table, KeyPhase, LuaCtx, LuaPlatform};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

fn guides() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/guide")
}

fn read(lang: &str, slug: &str) -> String {
    std::fs::read_to_string(guides().join(lang).join(format!("{slug}.md"))).unwrap()
}

#[test]
fn action_library_is_linked_from_the_gesture_creation_guides() {
    for lang in ["en", "zh-CN"] {
        for slug in ["index", "actions", "getting-started", "gestures"] {
            assert!(read(lang, slug).contains("(action-library.md)"), "{lang}/{slug}.md");
        }
        let library = read(lang, "action-library");
        for expected in ["title:", "description:", "order:", "<!-- action-library:generated:start -->", "<!-- action-library:generated:end -->", "(actions.md)"] {
            assert!(library.contains(expected), "{lang}/action-library.md: {expected}");
        }
    }
}

#[derive(Default)]
struct Stub {
    mac: bool,
    clipboard: Mutex<Option<String>>,
    selection: String,
    effects: Mutex<Vec<String>>,
}

impl Stub {
    fn record(&self, effect: impl Into<String>) {
        self.effects.lock().unwrap().push(effect.into());
    }
}

impl LuaPlatform for Stub {
    fn log(&self, message: &str) {
        self.record(message);
    }
    fn cursor_pos(&self) -> (i32, i32) {
        (-120, 345)
    }
    fn sleep(&self, millis: u64) {
        self.record(format!("sleep:{millis}"));
    }
    fn send_keys(&self, seq: &KeySeq, _: KeyPhase) {
        for action in &seq.actions {
            if matches!(
                action,
                Action::Key {
                    key: Key::Vk(0x43),
                    ..
                }
            ) {
                *self.clipboard.lock().unwrap() = Some(self.selection.clone());
            }
        }
        self.record("keys");
    }
    fn get_clipboard_text(&self) -> Option<String> {
        self.clipboard.lock().unwrap().clone()
    }
    fn set_clipboard_text(&self, text: &str) {
        *self.clipboard.lock().unwrap() = Some(text.into());
        self.record("clipboard");
    }
    fn open_url(&self, url: &str) {
        self.record(format!("open:{url}"));
    }
    fn run_shell(&self, code: &str) {
        self.record(format!("cmd:{code}"));
    }
    fn launch_app(&self, file: &str, args: Option<&str>) {
        self.record(format!("run:{file}:{}", args.unwrap_or_default()));
    }
    fn set_paused(&self, paused: bool) {
        self.record(format!("paused:{paused}"));
    }
    fn show_toast(&self, message: String, _: u64) {
        self.record(format!("toast:{message}"));
    }
    fn set_topmost(&self, _: i64, top: bool) {
        assert!(!self.mac, "recipe must not use topmost on Mac");
        self.record(format!("top:{top}"));
    }
    fn window_state(&self, _: i64) -> Option<String> {
        Some("normal".into())
    }
    fn set_window_state(&self, _: i64, state: &str) {
        self.record(format!("state:{state}"));
    }
    fn adjust_brightness(&self, _: i32, _: i32, _: i32) -> Option<i64> {
        (!self.mac).then_some(55)
    }
}

#[test]
fn every_registered_ms_function_is_documented_in_both_languages() {
    let lua = Lua::new();
    let platform: Arc<dyn LuaPlatform> = Arc::new(Stub::default());
    install_ms_table(&lua, &LuaCtx::default(), &platform).unwrap();
    let ms: Table = lua.globals().get("ms").unwrap();
    let registered: BTreeSet<String> = ms
        .pairs::<String, Value>()
        .map(|pair| {
            let (name, value) = pair.unwrap();
            assert!(
                matches!(value, Value::Function(_)),
                "ms.{name} is not a function"
            );
            name
        })
        .collect();
    assert!(!registered.is_empty());
    for lang in ["en", "zh-CN"] {
        let doc = read(lang, "lua-api");
        let documented: BTreeSet<String> = doc
            .lines()
            .filter_map(|line| line.strip_prefix("### `ms.")?.strip_suffix('`'))
            .map(str::to_owned)
            .collect();
        assert_eq!(registered, documented, "{lang}/lua-api.md API coverage");
    }
    println!(
        "Lua API: {} registered, documented in both languages",
        registered.len()
    );
}

#[test]
fn every_named_key_is_documented_in_both_languages() {
    let source = include_str!("../../mousetroke-core/src/keyseq.rs");
    let lookup = source
        .split("fn lookup_vk(")
        .nth(1)
        .unwrap()
        .split("#[cfg(test)]")
        .next()
        .unwrap();
    let special = source
        .split("fn resolve_special(")
        .nth(1)
        .unwrap()
        .split("struct BadChar")
        .next()
        .unwrap();
    let mut names: BTreeSet<String> = lookup
        .split('"')
        .skip(1)
        .step_by(2)
        .chain(special.split('"').skip(1).step_by(2))
        .filter(|s| {
            !s.is_empty()
                && s.bytes()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_')
        })
        .map(str::to_owned)
        .collect();
    // Probe the entire u16 range used by lookup_vk; no duplicated F-key upper bound.
    for number in 0..=u16::MAX {
        for prefix in ['F', 'N'] {
            let name = format!("{prefix}{number}");
            if keyseq::parse(&format!("{{{name}}}"), false).is_ok() {
                names.insert(name);
            }
        }
    }
    assert!(!names.is_empty());
    for lang in ["en", "zh-CN"] {
        let doc = read(lang, "key-sequences");
        for name in &names {
            let argument = match name.as_str() {
                "SLEEP" => " 1",
                "MOUSE_WHEEL" | "MOUSE_HWHEEL" => " 120",
                _ => "",
            };
            assert!(
                keyseq::parse(&format!("{{{name}{argument}}}"), false).is_ok(),
                "{name}"
            );
            assert!(
                doc.contains(&format!("`{{{name}}}`")),
                "{lang} missing {{{name}}}"
            );
        }
    }
    println!("Key names: {} documented in both languages", names.len());
}

fn lua_blocks(markdown: &str) -> Vec<(usize, String)> {
    let mut blocks = Vec::new();
    let mut current: Option<(usize, String)> = None;
    for (number, line) in markdown.lines().enumerate() {
        if line == "```lua" {
            assert!(current.is_none());
            current = Some((number + 1, String::new()));
        } else if line == "```" {
            if let Some(block) = current.take() {
                blocks.push(block);
            }
        } else if let Some((_, code)) = &mut current {
            code.push_str(line);
            code.push('\n');
        }
    }
    assert!(current.is_none(), "unclosed Lua fence");
    blocks
}

#[test]
fn all_guide_lua_examples_compile_in_the_actual_runtime() {
    let lua = Lua::new();
    let mut count = 0;
    for lang in ["en", "zh-CN"] {
        for file in std::fs::read_dir(guides().join(lang)).unwrap() {
            let path = file.unwrap().path();
            if path.extension().is_some_and(|ext| ext == "md") {
                let doc = std::fs::read_to_string(&path).unwrap();
                for (line, code) in lua_blocks(&doc) {
                    lua.load(&code)
                        .into_function()
                        .unwrap_or_else(|error| panic!("{}:{line}: {error}", path.display()));
                    count += 1;
                }
            }
        }
    }
    assert!(count > 0);
    println!("Lua examples: {count} compiled");
}

#[test]
fn recipes_execute_with_stubbed_platforms_and_selection_fallbacks() {
    for lang in ["en", "zh-CN"] {
        for (line, code) in lua_blocks(&read(lang, "lua-examples")) {
            for mac in [false, true] {
                for (saved, selection) in [(None, ""), (Some("saved text"), "a b&中文")] {
                    let lua = Lua::new();
                    lua.set_memory_limit(8 * 1024 * 1024).unwrap();
                    let package: Table = lua.globals().get("package").unwrap();
                    package
                        .set("config", if mac { "/\n" } else { "\\\n" })
                        .unwrap();
                    let stub = Arc::new(Stub {
                        mac,
                        clipboard: Mutex::new(saved.map(str::to_owned)),
                        selection: selection.into(),
                        ..Stub::default()
                    });
                    let platform: Arc<dyn LuaPlatform> = stub.clone();
                    install_ms_table(&lua, &LuaCtx::default(), &platform).unwrap();
                    lua.load(&code).exec().unwrap();
                    let entry: Function = lua.globals().get("ms_action").unwrap();
                    let pos = lua.create_table().unwrap();
                    pos.set("x", -120).unwrap();
                    pos.set("y", 345).unwrap();
                    entry
                        .call::<()>((
                            pos,
                            42,
                            123,
                            if mac {
                                "com.apple.Safari"
                            } else {
                                "firefox.exe"
                            },
                            "Example",
                            "UnderCursor",
                        ))
                        .unwrap_or_else(|error| {
                            panic!("{lang}/lua-examples.md:{line} mac={mac}: {error}")
                        });
                    let effects = stub.effects.lock().unwrap();
                    assert!(!effects.is_empty(), "recipe had no observable effect");
                    if code.contains("local saved =") {
                        assert_eq!(
                            stub.get_clipboard_text().as_deref(),
                            Some(saved.unwrap_or("")),
                            "clipboard restore at {lang}:{line}"
                        );
                    }
                    if code.contains("local function url_encode") && !selection.is_empty() {
                        assert!(effects.iter().any(|e| e
                            == "open:https://www.google.com/search?q=a%20b%26%E4%B8%AD%E6%96%87"));
                    }
                }
            }
        }
    }
}

#[test]
fn documented_standard_libraries_match_lua_new() {
    let lua = Lua::new();
    lua.load(r#"
        assert(_VERSION == 'Lua 5.4')
        for _, name in ipairs({'coroutine', 'table', 'io', 'os', 'string', 'utf8', 'math', 'package'}) do
            assert(type(_G[name]) == 'table', name)
        end
        assert(type(os.date) == 'function')
        assert(debug == nil)
        assert(not pcall(package.loadlib, 'missing', 'missing'))
    "#).exec().unwrap();
}
