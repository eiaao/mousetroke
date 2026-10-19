"""验证共享解析器拒绝会造成发布或站点缺文案的输入。"""
import unittest
from changelog import notes, parse

ENTRY = "## 0.1.1 — 2026-10-19\n\n### English\n\n- A change\n\n### 中文\n\n- 一项更新\n"


class ChangelogTests(unittest.TestCase):
    def test_notes_and_wrapped_bullets(self):
        releases = parse("# Changelog\n\n" + ENTRY.replace("A change", "A change\n  continued"))
        self.assertEqual(notes(releases, "0.1.1"), "### English\n\n- A change continued\n\n### 中文\n\n- 一项更新")
        with self.assertRaises(ValueError):
            notes(releases, "9.9.9")

    def test_rejects_missing_empty_duplicate_sections_and_invalid_dates(self):
        for text in ["", ENTRY.split("### 中文")[0], ENTRY.replace("- 一项更新", ""),
                     ENTRY + "\n### 中文\n- 重复", ENTRY.replace("2026-10-19", "2026-02-30"),
                     ENTRY.replace("0.1.1 —", "v0.1.1 —"), ENTRY.replace("- A change", "A change")]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                parse(text)

    def test_versions_are_numeric_unique_and_descending(self):
        parse(ENTRY.replace("0.1.1", "0.10.0") + ENTRY.replace("0.1.1", "0.2.0"))
        for tail in [ENTRY, ENTRY.replace("0.1.1", "0.2.0")]:
            with self.assertRaises(ValueError):
                parse(ENTRY + tail)


if __name__ == "__main__":
    unittest.main()
