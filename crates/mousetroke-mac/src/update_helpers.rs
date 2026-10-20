//! 不依赖 Apple API 的更新协议、路径与替换事务，可在 Windows 回归。
use mousetroke_core::update::{Error, Source};
use std::path::{Component, Path, PathBuf};

pub(crate) const BUNDLE_ID: &str = "io.github.eiaao.mousetroke";
pub(crate) const PAYLOAD: &str = "Mousetroke.dmg";

pub(crate) fn bundle_root(exe: &Path) -> Result<&Path, Error> {
    if exe.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(Error::NotWritable);
    }
    let macos = exe
        .parent()
        .filter(|p| p.file_name().is_some_and(|s| s == "MacOS"));
    let contents = macos
        .and_then(Path::parent)
        .filter(|p| p.file_name().is_some_and(|s| s == "Contents"));
    contents
        .and_then(Path::parent)
        .filter(|p| {
            p.extension().is_some_and(|s| s == "app")
                && exe.file_name().is_some_and(|s| s == "mousetroke")
        })
        .ok_or(Error::NotWritable)
}

#[derive(Debug)]
pub(crate) struct UpdatePaths {
    pub bundle: PathBuf,
    pub staging: PathBuf,
    pub old: PathBuf,
}

impl UpdatePaths {
    pub fn new(bundle: &Path) -> Result<Self, Error> {
        let directory = bundle.parent().ok_or(Error::NotWritable)?;
        let staging = directory.join(".Mousetroke-update.app");
        let old = directory.join(".Mousetroke-old.app");
        if bundle == staging || bundle == old {
            return Err(Error::NotWritable);
        }
        Ok(Self {
            bundle: bundle.into(),
            staging,
            old,
        })
    }

    pub fn swap(&self) -> Result<(), Error> {
        // 不覆盖任何遗留备份，尤其不能递归删除用户的应用。
        absent(&self.old)?;
        std::fs::rename(&self.bundle, &self.old).map_err(|_| Error::Replace)?;
        if std::fs::rename(&self.staging, &self.bundle).is_err() {
            std::fs::rename(&self.old, &self.bundle).map_err(|_| Error::Replace)?;
            return Err(Error::Replace);
        }
        Ok(())
    }

    pub fn rollback(&self) -> Result<(), Error> {
        self.restore_from(&self.old)
    }

    pub fn restore_from(&self, previous: &Path) -> Result<(), Error> {
        absent(&self.staging)?;
        std::fs::rename(&self.bundle, &self.staging).map_err(|_| Error::Replace)?;
        if std::fs::rename(previous, &self.bundle).is_err() {
            let _ = std::fs::rename(&self.staging, &self.bundle);
            return Err(Error::Replace);
        }
        Ok(())
    }
}

pub(crate) fn absent(path: &Path) -> Result<(), Error> {
    match std::fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(Error::Busy),
    }
}

pub(crate) fn designated_requirement(output: &str) -> Result<String, Error> {
    let mut lines = output
        .lines()
        .filter_map(|line| line.trim().strip_prefix("designated => "));
    let requirement = lines.next().ok_or(Error::Signature)?.trim();
    // 本项目的自签名要求必须绑定实际证书指纹；拒绝 ad-hoc 的 cdhash 或仅有 identifier 的弱要求。
    let prefix = format!("identifier \"{BUNDLE_ID}\" and certificate root = H\"");
    let fingerprint = requirement
        .strip_prefix(&prefix)
        .and_then(|s| s.strip_suffix('"'))
        .ok_or(Error::Signature)?;
    if lines.next().is_some()
        || fingerprint.len() != 40
        || !fingerprint.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(Error::Signature);
    }
    Ok(requirement.into())
}

pub(crate) fn redirect(source: &Source, url: &str, api: bool) -> Result<String, Error> {
    if url.len() > 8192 || !source.allowed(url, api) {
        return Err(Error::Network);
    }
    Ok(url.into())
}

/// curl 的两个尾字段从后往前拆分，DMG 中的换行不会影响状态码解析。
pub(crate) fn response(data: &[u8], cap: usize) -> Result<(&[u8], u16, &str), Error> {
    let last = data
        .iter()
        .rposition(|&b| b == b'\n')
        .ok_or(Error::Network)?;
    let status_start = data[..last]
        .iter()
        .rposition(|&b| b == b'\n')
        .ok_or(Error::Network)?;
    let status = std::str::from_utf8(&data[status_start + 1..last]).map_err(|_| Error::Network)?;
    if status.len() != 3 || !status.bytes().all(|b| b.is_ascii_digit()) || status_start > cap {
        return Err(Error::Network);
    }
    let next = std::str::from_utf8(&data[last + 1..]).map_err(|_| Error::Network)?;
    Ok((
        &data[..status_start],
        status.parse().map_err(|_| Error::Network)?,
        next,
    ))
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
