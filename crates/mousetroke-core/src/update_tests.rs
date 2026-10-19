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
