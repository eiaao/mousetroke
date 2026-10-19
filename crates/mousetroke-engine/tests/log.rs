use std::sync::Mutex;

static LINES: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[test]
fn platform_sink_receives_exact_text_and_is_registered_once() {
    mousetroke_engine::log::write("未注册时丢弃");
    mousetroke_engine::log::set_sink(|msg| LINES.lock().unwrap().push(msg.to_string()));
    mousetroke_engine::log::set_sink(|_| {});
    mousetroke_engine::log!("手势命中: {}（作用域 {}）", "关闭标签页", "所有程序");
    mousetroke_engine::log::write("保留\n原始文本");
    assert_eq!(*LINES.lock().unwrap(), ["手势命中: 关闭标签页（作用域 所有程序）", "保留\n原始文本"]);
}
