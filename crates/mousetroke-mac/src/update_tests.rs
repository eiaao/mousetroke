use super::*;
use mousetroke_core::update::{checksum_for, select_release_for};

#[test]
fn bundle_root_requires_exact_executable_layout() {
    for base in [
        "/Applications/Mousetroke.app",
        "/Users/test/Applications/Mouse stroke.app",
        "/tmp/a.app.bak/Custom.app",
    ] {
        assert_eq!(
            bundle_root(&Path::new(base).join("Contents/MacOS/mousetroke")).unwrap(),
            Path::new(base)
        );
    }
    for path in [
        "/tmp/mousetroke",
        "/Applications/Foo.app/elsewhere/mousetroke",
        "/Applications/Foo.app/Contents/MacOS/other",
        "/Applications/Foo.app.bak/Contents/MacOS/mousetroke",
        "/Applications/../Foo.app/Contents/MacOS/mousetroke",
    ] {
        assert!(bundle_root(Path::new(path)).is_err(), "{path}");
    }
}

#[test]
fn staging_names_stay_beside_the_running_bundle() {
    let paths = UpdatePaths::new(Path::new("/Users/test/Applications/Renamed.app")).unwrap();
    assert_eq!(
        paths.staging,
        Path::new("/Users/test/Applications/.Mousetroke-update.app")
    );
    assert_eq!(
        paths.old,
        Path::new("/Users/test/Applications/.Mousetroke-old.app")
    );
    assert!(UpdatePaths::new(&paths.staging).is_err());
    assert!(UpdatePaths::new(&paths.old).is_err());
}

#[test]
fn requirement_keeps_the_exact_certificate_pin_and_rejects_weaker_forms() {
    let requirement = format!(
        "identifier \"{BUNDLE_ID}\" and certificate root = H\"{}\"",
        "aB".repeat(20)
    );
    assert_eq!(
        designated_requirement(&format!(
            "Executable=/Applications/Mousetroke.app\r\n designated => {requirement}\r\n"
        ))
        .unwrap(),
        requirement
    );
    for text in [
        "".into(),
        "designated => identifier \"io.github.eiaao.mousetroke\"".into(),
        "designated => cdhash H\"1234\"".into(),
        format!("designated => {requirement}\ndesignated => {requirement}"),
        format!("designated => {requirement} or true"),
        format!(
            "designated => {}",
            requirement.replace(BUNDLE_ID, "evil.app")
        ),
        format!("designated => {}", requirement.replace("aB", "g0")),
        format!("designated => {requirement}\0"),
    ] {
        assert_eq!(
            designated_requirement(&text),
            Err(Error::Signature),
            "{text:?}"
        );
    }
}

#[test]
fn each_asset_redirect_must_match_the_source_allowlist() {
    let source = Source::production();
    for url in [
        "https://github.com/a",
        "https://objects.githubusercontent.com/a",
        "https://release-assets.githubusercontent.com/a?sig=abc",
    ] {
        assert!(redirect(&source, url, false).is_ok());
        assert!(redirect(&source, url, true).is_err());
    }
    assert!(redirect(&source, "https://api.github.com/a", true).is_ok());
    for url in [
        "http://github.com/a",
        "https://github.com.evil.test/a",
        "https://github.com:444/a",
        "https://user@github.com/a",
        "https://evil.test/a",
        "file:///tmp/a",
        "//github.com/a",
        "https://github.com/a\r\nX: y",
        "https://github.com/a#fragment",
    ] {
        assert!(redirect(&source, url, false).is_err(), "{url}");
    }
    let local = Source::local("http://127.0.0.1:8770").unwrap();
    assert!(redirect(&local, "http://127.0.0.1:8770/Mousetroke.dmg", false).is_ok());
    for url in [
        "http://127.0.0.1:8771/a",
        "http://127.0.0.1:87700/a",
        "http://localhost:8770/a",
        "https://github.com/a",
    ] {
        assert!(redirect(&local, url, false).is_err());
    }
}

#[test]
fn curl_trailer_handles_binary_newlines_and_caps_the_actual_body() {
    assert_eq!(
        response(b"\0\n200\n\xff\n200\n", 8).unwrap(),
        (&b"\0\n200\n\xff"[..], 200, "")
    );
    assert_eq!(
        response(b"\n302\nhttps://github.com/asset", 0).unwrap(),
        (&b""[..], 302, "https://github.com/asset")
    );
    for bytes in [
        &b"body\n200\n"[..],
        b"\n20\n",
        b"\nxyz\n",
        b"\n200",
        b"\n200\n\xff",
    ] {
        assert_eq!(response(bytes, 0), Err(Error::Network));
    }
}

#[test]
fn mixed_release_checksum_requires_one_exact_dmg_entry() {
    let hash = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    for marker in ["  ", " *"] {
        let text = format!(
            "\u{feff}{} *mousetroke.exe\r\n{}{marker}{PAYLOAD}\r\n",
            "f".repeat(64),
            hash.to_uppercase()
        );
        assert_eq!(checksum_for(&text, PAYLOAD).unwrap(), hash);
    }
    for text in [
        String::new(),
        format!("{hash}  ./{PAYLOAD}"),
        format!("invalid  {PAYLOAD}"),
        format!("{hash}  {PAYLOAD}\n{hash} *{PAYLOAD}"),
        format!("{hash} *mousetroke.exe"),
    ] {
        assert_eq!(checksum_for(&text, PAYLOAD), Err(Error::Checksum));
    }
    let release = r#"{"tag_name":"v999.0.0","draft":false,"prerelease":false,"html_url":"https://github.com/releases", "assets":[{"name":"Mousetroke.dmg","browser_download_url":"https://github.com/mac"},{"name":"mousetroke.exe","browser_download_url":"https://github.com/win"},{"name":"SHA256SUMS.txt","browser_download_url":"https://github.com/sums"}]}"#;
    assert_eq!(
        select_release_for(release.as_bytes(), &Source::production(), PAYLOAD)
            .unwrap()
            .exe,
        "https://github.com/mac"
    );
}

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "mousetroke-update-test-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn replacement_rolls_back_without_losing_either_bundle() {
    let dir = Directory::new();
    let paths = UpdatePaths::new(&dir.0.join("Custom.app")).unwrap();
    for (path, value) in [(&paths.bundle, "old"), (&paths.staging, "new")] {
        std::fs::create_dir(path).unwrap();
        std::fs::write(path.join("version"), value).unwrap();
    }
    paths.swap().unwrap();
    assert_eq!(
        std::fs::read_to_string(paths.bundle.join("version")).unwrap(),
        "new"
    );
    assert_eq!(
        std::fs::read_to_string(paths.old.join("version")).unwrap(),
        "old"
    );
    paths.rollback().unwrap();
    assert_eq!(
        std::fs::read_to_string(paths.bundle.join("version")).unwrap(),
        "old"
    );
    assert_eq!(
        std::fs::read_to_string(paths.staging.join("version")).unwrap(),
        "new"
    );
    assert!(paths.old.try_exists().is_ok_and(|exists| !exists));
}

#[test]
fn failed_second_rename_restores_original_and_never_overwrites_backup() {
    let dir = Directory::new();
    let paths = UpdatePaths::new(&dir.0.join("Mousetroke.app")).unwrap();
    std::fs::create_dir(&paths.bundle).unwrap();
    std::fs::write(paths.bundle.join("original"), "keep").unwrap();
    assert_eq!(paths.swap(), Err(Error::Replace));
    assert_eq!(
        std::fs::read_to_string(paths.bundle.join("original")).unwrap(),
        "keep"
    );
    std::fs::create_dir(&paths.old).unwrap();
    std::fs::write(paths.old.join("backup"), "keep too").unwrap();
    assert_eq!(paths.swap(), Err(Error::Busy));
    assert_eq!(
        std::fs::read_to_string(paths.old.join("backup")).unwrap(),
        "keep too"
    );
    assert_eq!(absent(&paths.old), Err(Error::Busy));
}

#[test]
fn failed_restore_keeps_new_bundle_available_and_trash_restore_uses_exact_path() {
    let dir = Directory::new();
    let paths = UpdatePaths::new(&dir.0.join("Mousetroke.app")).unwrap();
    std::fs::create_dir(&paths.bundle).unwrap();
    std::fs::write(paths.bundle.join("new"), "available").unwrap();
    assert_eq!(paths.rollback(), Err(Error::Replace));
    assert!(paths.bundle.join("new").is_file());
    assert!(!paths.staging.exists());
    let trashed = dir.0.join("Trash original.app");
    std::fs::create_dir(&trashed).unwrap();
    std::fs::write(trashed.join("old"), "restored").unwrap();
    paths.restore_from(&trashed).unwrap();
    assert!(paths.bundle.join("old").is_file());
    assert!(paths.staging.join("new").is_file());
}
