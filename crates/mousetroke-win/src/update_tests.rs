use super::*;

#[test]
fn hashes_and_sums_support_standard_text_and_binary_lines() {
    let digest = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    assert_eq!(sha256(b"abc").unwrap(), digest);
    for marker in [" ", " *"] {
        assert_eq!(checksum_line(&format!("\u{feff}{}{}mousetroke.exe\r\n", digest.to_uppercase(), marker)).unwrap(), digest);
    }
    for text in [String::new(), format!("{digest}  ./mousetroke.exe"), "xyz  mousetroke.exe".into(), format!("{digest}  mousetroke.exe\n{digest} *mousetroke.exe")] {
        assert_eq!(checksum_line(&text), Err(Error::Checksum));
    }
}

fn sample(origin: &str) -> serde_json::Value {
    serde_json::json!({ "tag_name": "v1.2.3", "draft": false, "prerelease": false,
        "body": "### English\n- From the API\n### 中文\n- 来自 API",
        "html_url": format!("{origin}/releases/tag/v1.2.3"), "assets": [
            {"name":"other.exe", "browser_download_url":format!("{origin}/other")},
            {"name":"mousetroke.exe", "browser_download_url":format!("{origin}/exe")},
            {"name":"SHA256SUMS.txt", "browser_download_url":format!("{origin}/sums")}
        ] })
}

#[test]
fn release_requires_stable_version_and_both_exact_assets() {
    let source = Source::production();
    let valid = sample("https://github.com");
    let parse = |v: &serde_json::Value| select_release(&serde_json::to_vec(v).unwrap(), &source);
    let release = parse(&valid).unwrap();
    assert_eq!(release.latest, "1.2.3");
    assert_eq!(release_notes(&release.body, "en-US"), ["From the API"]);
    let mut no_body = valid.clone(); no_body["body"] = serde_json::Value::Null;
    assert!(release_notes(&parse(&no_body).unwrap().body, "en-US").is_empty());
    assert_eq!(release.exe, "https://github.com/exe");
    assert_eq!(release.sums, "https://github.com/sums");
    for field in ["draft", "prerelease"] { let mut v = valid.clone(); v[field] = true.into(); assert!(parse(&v).is_err()); }
    for tag in ["nightly", "v1.2.3-rc1"] { let mut v = valid.clone(); v["tag_name"] = tag.into(); assert!(parse(&v).is_err()); }
    let mut v = valid.clone(); v["assets"].as_array_mut().unwrap().pop(); assert!(parse(&v).is_err());
    let mut v = valid.clone(); v["assets"][1]["browser_download_url"] = "https://evil.test/a".into(); assert!(parse(&v).is_err());
    let mut v = valid.clone(); v["html_url"] = "http://github.com/a".into(); assert!(parse(&v).is_err());
    let mut v = valid.clone(); let duplicate = v["assets"][1].clone(); v["assets"].as_array_mut().unwrap().push(duplicate); assert!(parse(&v).is_err());
}

#[test]
fn environment_override_is_debug_only() {
    // 本模块其他测试显式传 Source，不读取环境；结束时恢复原值。
    let previous = std::env::var_os("MOUSETROKE_UPDATE_BASE");
    std::env::set_var("MOUSETROKE_UPDATE_BASE", "http://127.0.0.1:8770/fixture");
    let source = Source::configured();
    if let Some(value) = previous { std::env::set_var("MOUSETROKE_UPDATE_BASE", value); }
    else { std::env::remove_var("MOUSETROKE_UPDATE_BASE"); }
    if cfg!(debug_assertions) {
        assert_eq!(source.api, "http://127.0.0.1:8770/fixture/repos/eiaao/mousetroke/releases/latest");
        assert!(source.allowed("http://127.0.0.1:8770/exe", false));
    } else {
        assert_eq!(source.api, API);
        assert!(!source.allowed("http://127.0.0.1:8770/exe", false));
    }
}

#[test]
#[ignore = "本地 HTTP 下载、校验和替换集成测试；不重启程序"]
fn local_download_verify_replace_and_rollback() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let mut mixed = sample(&origin);
    mixed["assets"].as_array_mut().unwrap().push(serde_json::json!({
        "name": "Mousetroke.dmg", "browser_download_url": format!("{origin}/mac")
    }));
    let release_json = serde_json::to_vec(&mixed).unwrap();
    let data = b"test replacement payload";
    let sums = format!("{}  Mousetroke.dmg\n{} *mousetroke.exe\n", "a".repeat(64), sha256(data).unwrap()).into_bytes();
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    let server = std::thread::spawn(move || {
        while !flag.load(Ordering::Acquire) {
            if let Ok((mut socket, _)) = listener.accept() {
                socket.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                let mut buf = [0; 4096];
                let len = socket.read(&mut buf).unwrap();
                let req = String::from_utf8_lossy(&buf[..len]);
                let path = req.split_whitespace().nth(1).unwrap_or("");
                let (status, extra, body): (&str, &str, &[u8]) = match path {
                    "/repos/eiaao/mousetroke/releases/latest" => ("200 OK", "", &release_json),
                    "/sums" => ("200 OK", "", &sums),
                    "/bad-sums" => ("200 OK", "", b"0000000000000000000000000000000000000000000000000000000000000000  mousetroke.exe"),
                    "/exe" => ("200 OK", "", data),
                    "/redirect" => ("302 Found", "Location: /exe\r\n", b""),
                    "/escape" => ("302 Found", "Location: http://example.com/\r\n", b""),
                    _ => ("404 Not Found", "", b""),
                };
                write!(socket, "HTTP/1.1 {status}\r\n{extra}Content-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
                let _ = socket.write_all(body);
            } else { std::thread::sleep(Duration::from_millis(5)); }
        }
    });
    struct Server(Arc<AtomicBool>, Option<std::thread::JoinHandle<()>>);
    impl Drop for Server { fn drop(&mut self) { self.0.store(true, Ordering::Release); self.1.take().unwrap().join().unwrap(); } }
    let _server = Server(stop, Some(server));
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../target/update-test-{}-{}", std::process::id(), now()));
    std::fs::create_dir_all(&dir).unwrap();
    struct Dir(PathBuf);
    impl Drop for Dir { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }
    let _dir = Dir(dir.clone());
    let exe = dir.join("mousetroke.exe");
    std::fs::write(&exe, b"original").unwrap();
    let source = Source::local(&origin).unwrap();
    let mut release = fetch_release(&source).unwrap();
    let transaction = download_replace(&source, &release, &exe).unwrap();
    assert_eq!(std::fs::read(&exe).unwrap(), data);
    assert_eq!(std::fs::read(sidecar(&exe, ".old")).unwrap(), b"original");
    transaction.rollback().unwrap();
    assert_eq!(std::fs::read(&exe).unwrap(), b"original");
    release.sums = format!("{origin}/bad-sums");
    assert!(matches!(download_replace(&source, &release, &exe), Err(Error::Checksum)));
    assert!(!sidecar(&exe, ".new").exists());
    assert_eq!(std::fs::read(&exe).unwrap(), b"original");
    assert_eq!(http::get(&source, &format!("{origin}/redirect"), false, 100).unwrap(), data);
    assert!(http::get(&source, &format!("{origin}/escape"), false, 100).is_err());
    assert!(http::get(&source, &format!("{origin}/exe"), false, 2).is_err());
    let state_path = dir.join("update-state.json");
    let state = UpdateState { last_check: Some(now()), last_notified: Some("1.2.3".into()) };
    state.save(&state_path).unwrap();
    state.save(&state_path).unwrap();
    assert!(!UpdateState::load(&state_path).due(now()));
    assert_eq!(UpdateState::load(&state_path).last_notified.as_deref(), Some("1.2.3"));
    // 设置窗口单实例，不会并发更新；残留的半个 .new 来自中断的上次更新，应被覆盖而不是误报不可写。
    release.sums = format!("{origin}/sums");
    std::fs::write(sidecar(&exe, ".new"), b"stale").unwrap();
    let transaction = download_replace(&source, &release, &exe).unwrap();
    assert_eq!(std::fs::read(&exe).unwrap(), data);
    transaction.rollback().unwrap();
    assert_eq!(std::fs::read(&exe).unwrap(), b"original");
}

#[test]
fn mixed_mac_windows_release_selects_and_verifies_only_windows_payload() {
    let data = b"Windows executable payload";
    let expected = sha256(data).unwrap();
    let mac_hash = "a".repeat(64);
    for sums in [format!("{mac_hash}  Mousetroke.dmg\n{expected}  mousetroke.exe\n"),
        format!("{expected} *mousetroke.exe\n{mac_hash} *Mousetroke.dmg\n")] {
        assert_eq!(checksum_line(&sums).unwrap(), sha256(data).unwrap());
        assert_ne!(checksum_line(&sums).unwrap(), sha256(b"corrupt exe").unwrap());
    }
    let mut json = sample("https://github.com");
    json["assets"].as_array_mut().unwrap().insert(0, serde_json::json!({
        "name": "Mousetroke.dmg", "browser_download_url": "https://github.com/Mousetroke.dmg"
    }));
    let release = select_release(&serde_json::to_vec(&json).unwrap(), &Source::production()).unwrap();
    assert_eq!(release.exe, "https://github.com/exe");
    assert_eq!(release.sums, "https://github.com/sums");
}
