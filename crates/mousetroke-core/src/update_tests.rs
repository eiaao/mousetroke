use super::*;

#[test]
fn notes_select_language_and_fall_back_only_to_english() {
    let body = "Intro\n### English\n- First\n  continued\n- Second\n\n### 中文\n- 第一项\n### Other\n- Ignore";
    for language in ["zh-CN", "zh-TW", "ZH-hans"] {
        assert_eq!(release_notes(body, language), ["第一项"]);
    }
    for language in ["en-US", "de-DE", "qps-ploc"] {
        assert_eq!(release_notes(body, language), ["First continued", "Second"]);
    }
    assert_eq!(
        release_notes("### English\n- Fallback", "zh-CN"),
        ["Fallback"]
    );
    assert_eq!(
        release_notes("### 中文\n- <b></b>\n### English\n- Fallback", "zh-CN"),
        ["Fallback"]
    );
    assert!(release_notes("### 中文\n- 中文", "en-US").is_empty());
    assert!(release_notes("No sections", "zh-CN").is_empty());
}

#[test]
fn notes_strip_markdown_html_and_cap_unicode_text() {
    let body = "### English\n- **Bold**, _emphasis_, ~~old~~ and `code`: [a link](https://example.com/a_(b)) and [reference][id].\n- <b>Text</b><script>alert(1)</script><style>hidden</style><!-- secret --> stays.\n- ![Alt](image.png) and \\*literal\\*\n- <img src=x>";
    assert_eq!(
        release_notes(body, "en"),
        [
            "Bold, emphasis, old and code: a link and reference.",
            "Text stays.",
            "Alt and *literal*"
        ]
    );
    let body = format!(
        "### English\n{}",
        (0..20)
            .map(|_| format!("- {}\n", "中".repeat(400)))
            .collect::<String>()
    );
    let notes = release_notes(&body, "en-US");
    assert_eq!(notes.len(), 12);
    assert!(notes
        .iter()
        .all(|note| note.chars().count() == 300 && note.ends_with('…')));
}

#[test]
fn versions_are_numeric_and_malformed_tags_are_rejected() {
    let tags = ["0.1.0", "v0.1.1", "0.2.0", "v1.0.0", "1.10.0"];
    for pair in tags.windows(2) {
        assert!(version(pair[0]).unwrap() < version(pair[1]).unwrap());
    }
    assert_eq!(version("v0.1.0"), version("0.1.0"));
    for tag in [
        "",
        "v",
        "1",
        "1.2",
        "1.2.3.4",
        "1.2.x",
        "1.2.3-rc1",
        "1.2.3+build",
        " 1.2.3",
        "vv1.2.3",
        "+1.2.3",
        "1.2.18446744073709551616",
    ] {
        assert_eq!(version(tag), None, "{tag}");
    }
}

#[test]
fn marker_accepts_only_prefixed_versions_and_one_optional_line_ending() {
    for ending in ["", "\n", "\r\n"] {
        assert_eq!(
            marker_version(format!("v1.2.3{ending}").as_bytes()),
            Some([1, 2, 3])
        );
    }
    for body in [
        "",
        "1.2.3",
        "v",
        "vv1.2.3",
        " v1.2.3",
        "v1.2.3 ",
        "v1.2.3\t",
        "v1.2.3\r",
        "v1.2.3\n\n",
        "v1.2.3\r\n\r\n",
        "v1.2.3\nv1.2.3",
        "\nv1.2.3",
        "v1.2.3\n ",
        "v1.2.3\0",
        "v1.2",
        "v1.2.3.4",
        "v1.2.x",
        "v1.2.3-rc1",
        "v1.2.3+build",
        "v+1.2.3",
        "v1.2.18446744073709551616",
        "\u{feff}v1.2.3",
        "v１.2.3",
    ] {
        assert_eq!(marker_version(body.as_bytes()), None, "{body:?}");
    }
    assert_eq!(marker_version(b"v1.2.\xff"), None);
    let maximum = format!("v{0}.{0}.{0}", u64::MAX);
    assert_eq!(marker_version(maximum.as_bytes()), Some([u64::MAX; 3]));
    let at_cap = format!("{maximum}\n");
    assert_eq!(at_cap.len(), MARKER_CAP);
    assert_eq!(marker_version(at_cap.as_bytes()), Some([u64::MAX; 3]));
    let over_cap = format!("{maximum}\r\n");
    assert_eq!(over_cap.len(), MARKER_CAP + 1);
    assert_eq!(marker_version(over_cap.as_bytes()), None);
    assert_eq!(
        marker_version(format!("v{}.2.3", "0".repeat(64)).as_bytes()),
        None
    );
}

#[test]
fn marker_url_uses_release_download_or_local_origin_without_changing_api() {
    let source = Source::production();
    assert_eq!(source.api, API);
    assert_eq!(
        source.marker_url(),
        "https://github.com/eiaao/mousetroke/releases/latest/download/latest.txt"
    );
    assert!(source.allowed(&source.marker_url(), false));
    for base in ["http://127.0.0.1:8770", "http://127.0.0.1:8770/fixture/"] {
        let source = Source::local(base).unwrap();
        assert_eq!(source.marker_url(), "http://127.0.0.1:8770/latest.txt");
        assert_eq!(
            source.api,
            format!(
                "{}/repos/eiaao/mousetroke/releases/latest",
                base.trim_end_matches('/')
            )
        );
        assert!(source.allowed(&source.marker_url(), false));
    }
}

fn api_check(latest: &str) -> Check {
    Check::from_release(
        Release {
            latest: latest.into(),
            notes_url: "https://github.com/eiaao/mousetroke/releases/latest".into(),
            body: "### English\n- API notes".into(),
            exe: "https://github.com/payload".into(),
            sums: "https://github.com/SHA256SUMS.txt".into(),
        },
        Some("https://github.com/payload".into()),
    )
}

#[test]
fn automatic_marker_never_changes_api_result_or_request_order() {
    use std::cell::RefCell;
    for latest in ["0.0.0", "999.0.0"] {
        for (marker, diagnostic) in [
            (Ok(format!("v{latest}\n").into_bytes()), None),
            (Ok(b"v1.2.3".to_vec()), Some("与 API 不一致")),
            (Ok(b"invalid".to_vec()), Some("无效")),
            (Ok(vec![b'x'; MARKER_CAP + 1]), Some("无效")),
            (Err(Error::Network), Some("请求失败")),
        ] {
            let requests = RefCell::new(Vec::new());
            let logs = RefCell::new(Vec::new());
            let result = automatic_check(
                || {
                    requests.borrow_mut().push("marker");
                    marker
                },
                || {
                    requests.borrow_mut().push("api");
                    Ok(api_check(latest))
                },
                |message| logs.borrow_mut().push(message.to_string()),
            )
            .unwrap();
            assert_eq!(*requests.borrow(), ["marker", "api"]);
            assert_eq!(
                serde_json::to_value(&result).unwrap(),
                serde_json::to_value(api_check(latest)).unwrap()
            );
            assert_eq!(result.newer, latest == "999.0.0");
            let logs = logs.borrow();
            match diagnostic {
                Some(text) => {
                    assert_eq!(logs.len(), 1);
                    assert!(logs[0].contains(text));
                }
                None => assert!(logs.is_empty()),
            }
        }
    }
}

#[test]
fn automatic_api_failure_is_returned_after_every_marker_outcome() {
    use std::cell::RefCell;
    for marker in [
        Ok(b"v999.0.0".to_vec()),
        Ok(b"invalid".to_vec()),
        Err(Error::Network),
    ] {
        let requests = RefCell::new(Vec::new());
        let result = automatic_check(
            || {
                requests.borrow_mut().push("marker");
                marker
            },
            || {
                requests.borrow_mut().push("api");
                Err(Error::Network)
            },
            |_| {},
        );
        assert_eq!(*requests.borrow(), ["marker", "api"]);
        assert!(matches!(result, Err(Error::Network)));
    }
}

#[test]
fn download_hosts_are_exact_https_and_local_override_is_loopback_only() {
    let source = Source::production();
    for host in [
        "github.com",
        "objects.githubusercontent.com",
        "release-assets.githubusercontent.com",
        "GITHUB.COM",
    ] {
        assert!(source.allowed(&format!("https://{host}/a?signature=test"), false));
    }
    for url in [
        "http://github.com/a",
        "https://github.com.evil.test/a",
        "https://evil.test/github.com/a",
        "https://github.com@evil.test/a",
        "https://evil.test@github.com/a",
        "https://github.com:444/a",
        "https://github.com\\evil/a",
        "https://github.com/%0a#a",
        "https://github.com/a\r\nb",
        "file:///tmp/a",
        "http://127.0.0.1:8080/a",
        "https://api.github.com/a",
    ] {
        assert!(!source.allowed(url, false), "{url}");
    }
    assert!(source.allowed(API, true));
    for base in [
        "http://localhost:8080",
        "http://127.0.0.2:8080",
        "https://127.0.0.1:8080",
        "http://127.0.0.1",
        "http://127.0.0.1:80@evil.test",
    ] {
        assert!(Source::local(base).is_none());
    }
    let local = Source::local("http://127.0.0.1:8080/test").unwrap();
    assert!(local.allowed("http://127.0.0.1:8080/test/exe", false));
    assert!(!local.allowed("http://127.0.0.1:80801/exe", false));
    assert!(!local.allowed("https://github.com/exe", false));
}

#[test]
fn state_waits_twenty_four_hours_and_handles_clock_rollback() {
    let mut state = UpdateState::default();
    assert!(state.due(0));
    state.last_check = Some(DAY);
    assert!(!state.due(DAY - 1));
    assert!(!state.due(DAY));
    assert!(!state.due(2 * DAY - 1));
    assert!(state.due(2 * DAY));
    state.last_notified = Some("1.2.3".into());
    let copy: UpdateState = serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
    assert_eq!(copy.last_notified, state.last_notified);
}

#[test]
fn mac_release_requires_exact_dmg_and_sums_and_rejects_untrusted_urls() {
    let mut value = serde_json::json!({"tag_name":"v1.2.3", "draft":false, "prerelease":false,
        "html_url":"https://github.com/eiaao/mousetroke/releases/tag/v1.2.3", "assets":[
        {"name":"mousetroke.exe", "browser_download_url":"https://github.com/exe"},
        {"name":"Mousetroke.dmg", "browser_download_url":"https://github.com/mac"},
        {"name":"SHA256SUMS.txt", "browser_download_url":"https://github.com/sums"}]});
    let parse = |v: &serde_json::Value| {
        select_release_for(
            &serde_json::to_vec(v).unwrap(),
            &Source::production(),
            "Mousetroke.dmg",
        )
    };
    assert_eq!(parse(&value).unwrap().exe, "https://github.com/mac");
    for field in ["draft", "prerelease"] {
        let mut bad = value.clone();
        bad[field] = true.into();
        assert!(parse(&bad).is_err());
    }
    let mut bad = value.clone();
    let duplicate = bad["assets"][1].clone();
    bad["assets"].as_array_mut().unwrap().push(duplicate);
    assert!(parse(&bad).is_err());
    let mut bad = value.clone();
    bad["assets"].as_array_mut().unwrap().remove(1);
    assert!(parse(&bad).is_err());
    value["assets"][1]["browser_download_url"] = "https://github.com.evil.test/mac".into();
    assert!(parse(&value).is_err());
    let hash = "b".repeat(64);
    assert_eq!(
        checksum_for(
            &format!(
                "{}  mousetroke.exe\n{hash} *Mousetroke.dmg\n",
                "a".repeat(64)
            ),
            "Mousetroke.dmg"
        )
        .unwrap(),
        hash
    );
    assert!(checksum_for(
        &format!("{hash}  Mousetroke.dmg\n{hash} *Mousetroke.dmg"),
        "Mousetroke.dmg"
    )
    .is_err());
}
