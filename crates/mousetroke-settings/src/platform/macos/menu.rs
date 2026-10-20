//! 覆盖 Tauri 默认英文菜单，保留 WKWebView 原生文本编辑与应用快捷键。
use crate::{keys, t};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem as Item, Submenu},
    Manager,
};

pub fn install(app: &tauri::App) {
    refresh(app.handle());
    app.on_menu_event(|app, event| {
        if event.id().as_ref() == "settings-about" {
            crate::show_window(app, true);
        }
    });
}

pub fn refresh(app: &tauri::AppHandle) {
    // 菜单创建失败不影响界面；系统默认菜单仍可工作。
    let _ = build(app).and_then(|menu| app.set_menu(menu).map(|_| ()));
}
fn build(app: &tauri::AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let about = MenuItem::with_id(
        app,
        "settings-about",
        t(keys::MAC_MENU_ABOUT),
        true,
        None::<&str>,
    )?;
    let sep = Item::separator(app)?;
    let hide = Item::hide(app, Some(t(keys::MAC_MENU_HIDE)))?;
    let hide_others = Item::hide_others(app, Some(t(keys::MAC_MENU_HIDE_OTHERS)))?;
    let show_all = Item::show_all(app, Some(t(keys::MAC_MENU_SHOW_ALL)))?;
    let quit = Item::quit(app, Some(t(keys::TRAY_MENU_QUIT)))?;
    let app_menu = Submenu::with_items(
        app,
        "Mousetroke",
        true,
        &[
            &about,
            &sep,
            &hide,
            &hide_others,
            &show_all,
            &Item::separator(app)?,
            &quit,
        ],
    )?;
    let undo = Item::undo(app, Some(t(keys::COMMON_UNDO)))?;
    let redo = Item::redo(app, Some(t(keys::COMMON_REDO)))?;
    let cut = Item::cut(app, Some(t(keys::MAC_MENU_CUT)))?;
    let copy = Item::copy(app, Some(t(keys::MAC_MENU_COPY)))?;
    let paste = Item::paste(app, Some(t(keys::MAC_MENU_PASTE)))?;
    let all = Item::select_all(app, Some(t(keys::MAC_MENU_SELECT_ALL)))?;
    let edit = Submenu::with_items(
        app,
        t(keys::MAC_MENU_EDIT),
        true,
        &[
            &undo,
            &redo,
            &Item::separator(app)?,
            &cut,
            &copy,
            &paste,
            &all,
        ],
    )?;
    Menu::with_items(app, &[&app_menu, &edit])
}
