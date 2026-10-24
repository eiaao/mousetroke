//! 内置的"建议禁用手势"程序清单：这些程序自己要用右键/中键拖动（远程操作、视图旋转/平移），
//! 手势会和它们抢鼠标键。全屏游戏已由全屏自动暂停覆盖，不在此列。
//!
//! 显示名与类别都是词条：`preset.app.<exe 小写、空格换下划线>` 与 `preset.category.*`。
//! 产品名（ToDesk、Blender…）在两种语言里通常一样，但"远程桌面连接"这类是系统组件的
//! 本地化名称，仍然进词条。

use crate::i18n::keys;

pub mod actions;

/// 建议禁用的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetCategory {
    /// 远程桌面与虚拟机。
    Remote,
    /// 3D 建模与设计。
    Design,
    /// 游戏引擎编辑器。
    GameDev,
}

impl PresetCategory {
    /// 类别显示名的词条键。
    pub fn key(self) -> &'static str {
        match self {
            Self::Remote => keys::PRESET_CATEGORY_REMOTE,
            Self::Design => keys::PRESET_CATEGORY_DESIGN,
            Self::GameDev => keys::PRESET_CATEGORY_GAMEDEV,
        }
    }
}

/// (exe 文件名, 显示名词条键, 类别)
pub const DISABLE_SUGGESTIONS: &[(&str, &str, PresetCategory)] = &[
    ("mstsc.exe", keys::PRESET_APP_MSTSC, PresetCategory::Remote),
    ("ToDesk.exe", keys::PRESET_APP_TODESK, PresetCategory::Remote),
    ("SunloginClient.exe", keys::PRESET_APP_SUNLOGINCLIENT, PresetCategory::Remote),
    ("AnyDesk.exe", keys::PRESET_APP_ANYDESK, PresetCategory::Remote),
    ("TeamViewer.exe", keys::PRESET_APP_TEAMVIEWER, PresetCategory::Remote),
    ("rustdesk.exe", keys::PRESET_APP_RUSTDESK, PresetCategory::Remote),
    ("parsecd.exe", keys::PRESET_APP_PARSECD, PresetCategory::Remote),
    ("vmware.exe", keys::PRESET_APP_VMWARE, PresetCategory::Remote),
    ("vmconnect.exe", keys::PRESET_APP_VMCONNECT, PresetCategory::Remote),
    ("VirtualBoxVM.exe", keys::PRESET_APP_VIRTUALBOXVM, PresetCategory::Remote),
    ("blender.exe", keys::PRESET_APP_BLENDER, PresetCategory::Design),
    ("acad.exe", keys::PRESET_APP_ACAD, PresetCategory::Design),
    ("SLDWORKS.exe", keys::PRESET_APP_SLDWORKS, PresetCategory::Design),
    ("3dsmax.exe", keys::PRESET_APP_3DSMAX, PresetCategory::Design),
    ("maya.exe", keys::PRESET_APP_MAYA, PresetCategory::Design),
    ("SketchUp.exe", keys::PRESET_APP_SKETCHUP, PresetCategory::Design),
    ("Rhino.exe", keys::PRESET_APP_RHINO, PresetCategory::Design),
    ("Cinema 4D.exe", keys::PRESET_APP_CINEMA_4D, PresetCategory::Design),
    ("UnrealEditor.exe", keys::PRESET_APP_UNREALEDITOR, PresetCategory::GameDev),
    ("Unity.exe", keys::PRESET_APP_UNITY, PresetCategory::GameDev),
];

/// macOS 的应用身份均为 bundle id，避免安装位置或语言影响匹配。
pub const MAC_DISABLE_SUGGESTIONS: &[(&str, &str, PresetCategory)] = &[
    ("com.parallels.desktop.console", keys::PRESET_APP_PARALLELS, PresetCategory::Remote),
    ("com.vmware.fusion", keys::PRESET_APP_FUSION, PresetCategory::Remote),
    ("com.microsoft.rdc.macos", keys::PRESET_APP_WINDOWS_APP, PresetCategory::Remote),
    ("com.apple.ScreenSharing", keys::PRESET_APP_SCREEN_SHARING, PresetCategory::Remote),
    ("org.virtualbox.app.VirtualBoxVM", keys::PRESET_APP_VIRTUALBOXVM, PresetCategory::Remote),
    ("org.blenderfoundation.blender", keys::PRESET_APP_BLENDER, PresetCategory::Design),
    ("net.maxon.cinema4d", keys::PRESET_APP_CINEMA_4D, PresetCategory::Design),
    ("com.autodesk.Maya", keys::PRESET_APP_MAYA, PresetCategory::Design),
    ("com.unity3d.UnityEditor", keys::PRESET_APP_UNITY, PresetCategory::GameDev),
    ("com.epicgames.UnrealEditor", keys::PRESET_APP_UNREALEDITOR, PresetCategory::GameDev),
];

pub fn disable_suggestions() -> &'static [(&'static str, &'static str, PresetCategory)] {
    if cfg!(target_os = "macos") { MAC_DISABLE_SUGGESTIONS } else { DISABLE_SUGGESTIONS }
}

/// 程序（exe 名或完整路径）在建议清单里时，返回它显示名的词条键（preset.app.*）。
/// 添加程序时写入作用域的 `name_key`，显示随语言切换。
pub fn name_key(exe: &str) -> Option<&'static str> {
    let file = crate::matcher::file_name(exe).to_ascii_lowercase();
    disable_suggestions()
        .iter()
        .find(|(e, _, _)| e.to_ascii_lowercase() == file)
        .map(|(_, k, _)| *k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_name_keys_match_catalog() {
        // 键名生成规则必须和 build.rs / i18n 词条一致，这里抽查几个
        if cfg!(target_os = "macos") {
            assert_eq!(name_key("com.vmware.fusion"), Some(keys::PRESET_APP_FUSION));
            assert_eq!(name_key("ORG.BLENDERFOUNDATION.BLENDER"), Some(keys::PRESET_APP_BLENDER));
        } else {
            assert_eq!(name_key("C:\\Program Files\\Cinema 4D.exe"), Some(keys::PRESET_APP_CINEMA_4D));
            assert_eq!(name_key("MSTSC.EXE"), Some(keys::PRESET_APP_MSTSC));
        }
        assert_eq!(name_key("chrome.exe"), None);
        // 每个键都真实存在于词条库（不断言具体译文：当前语言是进程全局，别的测试会切换它）
        for (_, k, cat) in DISABLE_SUGGESTIONS.iter().chain(MAC_DISABLE_SUGGESTIONS) {
            assert!(crate::i18n::try_t(k).is_some(), "缺少词条 {k}");
            assert!(crate::i18n::try_t(cat.key()).is_some(), "缺少词条 {}", cat.key());
        }
    }
}
