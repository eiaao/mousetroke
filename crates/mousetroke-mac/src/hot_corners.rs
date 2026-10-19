//! 系统触发角只读检查；禁用作用域和绑定不参与冲突提示。
use mousetroke_core::{
    edge,
    schema::{Config, Step},
};

pub const CORNERS: [(&str, u8); 4] = [
    ("wvous-tl-corner", edge::LEFT_TOP),
    ("wvous-tr-corner", edge::RIGHT_TOP),
    ("wvous-bl-corner", edge::LEFT_BOTTOM),
    ("wvous-br-corner", edge::RIGHT_BOTTOM),
];

fn conflicts(config: &Config, values: [i64; 4]) -> Vec<u8> {
    CORNERS
        .iter()
        .zip(values)
        .filter_map(|((_, corner), value)| {
            (value > 1 && config.scopes.iter().filter(|s| !s.disabled).any(|s| {
                s.bindings.iter().any(|b| {
                    !b.disabled
                        && matches!(b.pattern.first(), Some(Step::Edge { edge }) if edge == corner)
                })
            }))
            .then_some(*corner)
        })
        .collect()
}

#[cfg(target_os = "macos")]
pub fn check(config: &Config) {
    use crate::ax::{self, Cf, Raw};
    use mousetroke_core::i18n::{self, keys};
    use objc2_core_foundation::CFString;
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFPreferencesCopyAppValue(key: Raw, application: Raw) -> Raw;
        fn CFPreferencesAppSynchronize(application: Raw) -> bool;
    }
    let app = CFString::from_str("com.apple.dock");
    let app_raw = (&*app as *const CFString).cast();
    // 安全：有效 CFString，刷新本进程缓存以便配置重载时读到最新偏好；不设置任何键。
    unsafe { CFPreferencesAppSynchronize(app_raw) };
    let values = CORNERS.map(|(key, _)| {
        let key = CFString::from_str(key);
        // 安全：Copy 返回拥有的 CF 对象，Cf 负责释放；缺失/非数字按未配置处理。
        unsafe {
            Cf::owned(CFPreferencesCopyAppValue(
                (&*key as *const CFString).cast(),
                app_raw,
            ))
        }
        .and_then(|v| ax::number(v.raw()))
        .unwrap_or(0)
    });
    let corners = conflicts(config, values);
    for corner in &corners {
        crate::log!("系统触发角冲突：{}；保留双方设置", edge::name(*corner));
    }
    if !corners.is_empty() {
        let names = corners
            .iter()
            .filter_map(|e| i18n::try_t(&format!("edge.{e}")))
            .collect::<Vec<_>>()
            .join(", ");
        mousetroke_engine::push_toast(
            i18n::tf(keys::MAC_HOT_CORNER_CONFLICT, &[("corner", &names)]),
            2200,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dock_keys_map_to_shared_corner_bits() {
        assert_eq!(
            CORNERS,
            [
                ("wvous-tl-corner", 9),
                ("wvous-tr-corner", 3),
                ("wvous-bl-corner", 12),
                ("wvous-br-corner", 6)
            ]
        );
    }
    #[test]
    fn only_enabled_bindings_and_system_actions_conflict() {
        let mut config = Config::from_json(include_str!(
            "../../mousetroke-core/assets/default-config-macos.json"
        ))
        .unwrap();
        assert!(conflicts(&config, [0, 1, 0, 1]).is_empty());
        assert_eq!(
            conflicts(&config, [2; 4]),
            [edge::LEFT_TOP, edge::LEFT_BOTTOM, edge::RIGHT_BOTTOM]
        );
        config.scopes[0]
            .bindings
            .iter_mut()
            .for_each(|b| b.disabled = true);
        assert!(conflicts(&config, [2; 4]).is_empty());
        config.scopes[0]
            .bindings
            .iter_mut()
            .for_each(|b| b.disabled = false);
        assert_eq!(conflicts(&config, [2; 4]).len(), 4);
        config.scopes[0].disabled = true;
        assert!(conflicts(&config, [2; 4]).is_empty());
    }
}
