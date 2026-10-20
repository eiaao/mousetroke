#!/usr/bin/env python3
"""发布流程和网站共用的双语更新日志解析器。"""
import argparse
from dataclasses import dataclass, field
from datetime import date
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parent.parent
LANGUAGES = ("English", "中文")


@dataclass
class Release:
    version: str
    date: str
    sections: dict[str, list[str]] = field(default_factory=dict)


def parse(text: str) -> list[Release]:
    releases = []
    current = None
    language = None
    previous = None
    for number, line in enumerate(text.splitlines(), 1):
        if line.startswith("## "):
            match = re.fullmatch(r"## (\d+\.\d+\.\d+) — (\d{4}-\d{2}-\d{2})", line)
            if not match:
                raise ValueError(f"line {number}: expected '## <version> — <YYYY-MM-DD>'")
            version, day = match.groups()
            date.fromisoformat(day)
            parts = tuple(map(int, version.split(".")))
            if previous is not None and parts >= previous:
                raise ValueError(f"line {number}: versions must be unique and descending")
            previous = parts
            current = Release(version, day)
            releases.append(current)
            language = None
        elif current is not None and line.startswith("### "):
            language = line[4:]
            if language not in LANGUAGES or language in current.sections:
                raise ValueError(f"line {number}: unknown or duplicate language section")
            current.sections[language] = []
        elif current is not None and line.strip():
            if language is None:
                raise ValueError(f"line {number}: bullet outside a language section")
            if line.startswith("- ") and line[2:].strip():
                current.sections[language].append(line[2:].strip())
            elif line.startswith("  ") and current.sections[language]:
                current.sections[language][-1] += " " + line.strip()
            else:
                raise ValueError(f"line {number}: expected a non-empty bullet")
    if not releases:
        raise ValueError("no releases found")
    for release in releases:
        for language in LANGUAGES:
            if not release.sections.get(language):
                raise ValueError(f"{release.version}: missing or empty {language} section")
    return releases


def load(path: Path = ROOT / "CHANGELOG.md") -> list[Release]:
    return parse(path.read_text(encoding="utf-8"))


def notes(releases: list[Release], version: str) -> str:
    release = next((release for release in releases if release.version == version), None)
    if release is None:
        raise ValueError(f"no changelog entry for {version}")
    return "\n\n".join("### " + language + "\n\n" + "\n".join(
        "- " + bullet for bullet in release.sections[language]) for language in LANGUAGES)


def workspace_version(manifest: str) -> str:
    # 只读取 workspace.package.version，与打包脚本一致；Python 3.9 无 tomllib。
    section = re.search(r"(?ms)^\[workspace\.package\][ \t]*(?:#[^\n]*)?\n(.*?)(?=^\[|\Z)", manifest)
    if section:
        version = re.search(r'''(?m)^version[ \t]*=[ \t]*(["'])([^"'\n]+)\1[ \t]*(?:#.*)?$''', section[1])
        if version:
            return version[2]
    raise ValueError("Cargo.toml: missing workspace.package.version")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    notes_parser = commands.add_parser("notes")
    notes_parser.add_argument("version")
    # 直接写 UTF-8 文件：经 PowerShell 重定向可能按控制台代码页转码，中文会乱
    notes_parser.add_argument("--out", type=Path)
    commands.add_parser("check")
    args = parser.parse_args()
    try:
        releases = load()
        if args.command == "notes":
            body = notes(releases, args.version)
            if args.out:
                args.out.write_text(body + "\n", encoding="utf-8")
            else:
                print(body)
        else:
            version = workspace_version((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
            notes(releases, version)
            print(f"OK: {len(releases)} releases; workspace {version} has an entry")
        return 0
    except (ValueError, OSError) as error:
        print(f"changelog: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    raise SystemExit(main())
