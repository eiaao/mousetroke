//! bundle id / 路径 → 系统图标 PNG；缓存限制为 256 项，只缓存成功结果。
use objc2::{rc::Retained, AllocAnyThread};
use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSWorkspace};
use objc2_foundation::{NSDictionary, NSString, NSURL};
use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
};

thread_local! { static CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new()); }

pub fn resolve_exe(identity: &str) -> Option<PathBuf> {
    if Path::new(identity).is_absolute() {
        return Path::new(identity).exists().then(|| identity.into());
    }
    NSWorkspace::sharedWorkspace()
        .URLForApplicationWithBundleIdentifier(&NSString::from_str(identity))?
        .path()
        .map(|p| PathBuf::from(p.to_string()))
}

pub fn icon_data_url(identity: &str) -> Option<String> {
    if let Some(hit) = CACHE.with(|c| c.borrow().get(identity).cloned()) {
        return Some(hit);
    }
    let path = resolve_exe(identity)?;
    let image =
        NSWorkspace::sharedWorkspace().iconForFile(&NSString::from_str(&path.to_string_lossy()));
    let tiff = image.TIFFRepresentation()?;
    let bitmap = NSBitmapImageRep::initWithData(NSBitmapImageRep::alloc(), &tiff)?;
    // 安全：空 properties 字典符合 PNG 编码接口要求，bitmap 来自有效 TIFF。
    let png = unsafe {
        bitmap.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())
    }?;
    let encoded =
        png.base64EncodedStringWithOptions(objc2_foundation::NSDataBase64EncodingOptions(0));
    let url = format!("data:image/png;base64,{encoded}");
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.len() >= 256 {
            cache.clear();
        }
        cache.insert(identity.into(), url.clone());
    });
    Some(url)
}

pub fn bundle_url(path: &Path) -> Retained<NSURL> {
    NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()))
}
