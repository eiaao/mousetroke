//! 两平台在工作线程/动作队列消费时调用，菜单选中的动作也走同一个入口。
use mousetroke_core::{caps::Platform, i18n::{self, Lang}, presets::actions, schema::Action};

pub fn prepare_action(action: Action, platform: Platform, ui_lang: Lang) -> Option<Action> {
    prepare_action_with_feedback(action, platform, ui_lang, |key| crate::push_toast(i18n::t(key).into(), 1600))
}

fn prepare_action_with_feedback(action: Action, platform: Platform, ui_lang: Lang, feedback: impl FnOnce(&'static str)) -> Option<Action> {
    match action {
        Action::Preset { id, params } => match actions::expand(&id, platform, &params, ui_lang) {
            Ok(action) => Some(action),
            Err(error) => {
                crate::log!("跳过预置动作 {id}: {error:?}");
                feedback(error.reason_key());
                None
            }
        },
        Action::Unknown(raw) => {
            crate::log!("跳过较新版本的动作: type={}, id={}", raw.get("type").and_then(|v| v.as_str()).unwrap_or("?"), raw.get("id").and_then(|v| v.as_str()).unwrap_or("?"));
            feedback("preset.status.unknown");
            None
        }
        action => Some(action),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_expands_supported_presets_and_skips_unavailable_actions() {
        // 验证同一分发逻辑，不向其他状态机测试共用的全局 Toast 队列写入。
        let prepare_action = |action, platform, lang| prepare_action_with_feedback(action, platform, lang, |_| {});
        let preset = |id: &str| Action::Preset { id: id.into(), params: Default::default() };
        for (platform, keys) in [(Platform::Windows, "^c"), (Platform::Macos, "#c")] {
            assert!(matches!(prepare_action(preset("edit.copy"), platform, Lang::EnUs), Some(Action::KeySequence { keys: actual, .. }) if actual == keys));
            for id in ["future.preset", "window.snap_left", "mousetroke.cheatsheet"] {
                assert!(prepare_action(preset(id), platform, Lang::EnUs).is_none());
            }
            assert!(prepare_action(Action::Unknown(Default::default()), platform, Lang::EnUs).is_none());
        }
        assert!(prepare_action(preset("window.toggle_topmost"), Platform::Macos, Lang::EnUs).is_none());
        let menu = Action::Menu { columns: 1, items: vec![mousetroke_core::schema::MenuItem { label: "Copy".into(), action: preset("edit.copy") }] };
        let Some(Action::Menu { items, .. }) = prepare_action(menu, Platform::Macos, Lang::EnUs) else { panic!() };
        assert!(matches!(prepare_action(items[0].action.clone(), Platform::Macos, Lang::EnUs), Some(Action::KeySequence { keys, .. }) if keys == "#c"));
    }
}
