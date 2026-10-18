#!/usr/bin/env python3
"""禁止在源码里写死中文（docs/I18N-DESIGN.md §7.2）。

扫描 crates/**/src/**/*.rs 与 ui/src/**/*.{ts,tsx} 中的字符串字面量与 JSX 文本，
出现 CJK 字符就报告（注释不算）。只用标准库。

白名单：
- ``log!(…)`` 宏的参数（运行日志不翻译）；
- 测试：``*_tests.rs``、``tests/`` 目录、``#[cfg(test)]`` 之后的代码；
- names.rs 里迁移用的中文对照；
- 行内有 ``i18n:allow`` 标记的（就地豁免，须写明理由）。

用法：
  python scripts/i18n_lint.py            # 报告（P0–P4 阶段）
  python scripts/i18n_lint.py --check    # 有违规时退出码 1（P5 起接入 cargo test / npm build）
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# CJK 区段：CJK 统一表意文字、CJK 标点、全角形式
def is_cjk(ch: str) -> bool:
    cp = ord(ch)
    return (
        0x4E00 <= cp <= 0x9FFF
        or 0x3000 <= cp <= 0x303F
        or 0xFF00 <= cp <= 0xFFEF
    )


def has_cjk(s: str) -> bool:
    return any(is_cjk(c) for c in s)


class Violation:
    def __init__(self, path: Path, line: int, snippet: str):
        self.path, self.line, self.snippet = path, line, snippet

    def __str__(self):
        rel = self.path.relative_to(ROOT)
        return f"{rel}:{self.line}: {self.snippet.strip()[:90]}"


# ---------------------------------------------------------------- Rust 扫描

def scan_rust(path: Path, text: str, allowed_lines: set[int]) -> list[Violation]:
    out: list[Violation] = []
    i, n = 0, len(text)
    line = 1
    state = "code"  # code | line_comment | block_comment | string | raw
    raw_hashes = 0
    buf_start = None

    def advance():
        nonlocal i, line
        if text[i] == "\n":
            line += 1
        i += 1

    macro_depth = 0  # 在 log!(…) 的参数里时 > 0（跳过一切，包括字符串）

    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if macro_depth > 0 and state == "code":
            # log!(…) 参数里跳过一切；字符串内的括号不能参与深度计数
            if c == '"':
                j = i + 1
                while j < n and text[j] != '"':
                    j += 2 if text[j] == "\\" else 1
                line += text.count("\n", i, min(j + 1, n))
                i = j + 1
                continue
            if c == "(":
                macro_depth += 1
            elif c == ")":
                macro_depth -= 1
            advance()
            continue
        if state == "code":
            if c == "/" and nxt == "/":
                state = "line_comment"
                advance(); advance()
            elif c == "/" and nxt == "*":
                state = "block_comment"
                advance(); advance()
            elif c == '"':
                state = "string"
                buf_start = i + 1
                advance()
            elif c == "r" and nxt in ('"', "#"):
                # 原始字符串 r"…" / r#"…"#
                j = i + 1
                hashes = 0
                while j < n and text[j] == "#":
                    hashes += 1
                    j += 1
                if j < n and text[j] == '"':
                    state = "raw"
                    raw_hashes = hashes
                    buf_start = j + 1
                    # 越过 r、# 和开头的引号；只越过 r 的话，r"…" 的开头引号会被当成结尾
                    while i <= j:
                        advance()
                else:
                    advance()  # 普通标识符 r 开头
            elif c == "'":
                # 字符字面量 'x' / '\n'；生命周期 'a 不会闭合
                j = i + 1
                if j < n and text[j] == "\\":
                    j += 2
                elif j < n:
                    j += 1
                if j < n and text[j] == "'":
                    advance()
                    advance()
                    advance()
                else:
                    advance()
            elif c == "!" and i > 0 and _ident_before(text, i) == "log":
                # log!( — 进入宏参数跳过模式（等 '('）
                advance()
                if i < n and text[i] == "(":
                    macro_depth = 1
                    advance()
            else:
                advance()
        elif state == "line_comment":
            if c == "\n":
                state = "code"
            advance()
        elif state == "block_comment":
            if c == "*" and nxt == "/":
                state = "code"
                advance(); advance()
            else:
                advance()
        elif state == "string":
            if c == "\\":
                advance(); advance()
            elif c == '"':
                content = text[buf_start:i]
                if has_cjk(content) and line not in allowed_lines:
                    out.append(Violation(path, line, content))
                state = "code"
                advance()
            else:
                advance()
        elif state == "raw":
            if c == '"' and text.startswith("#" * raw_hashes, i + 1):
                content = text[buf_start:i]
                if has_cjk(content) and line not in allowed_lines:
                    out.append(Violation(path, line, content))
                state = "code"
                for _ in range(1 + raw_hashes):
                    advance()
            else:
                advance()
    return out


def _ident_before(text: str, pos: int) -> str:
    """pos 前面紧邻的标识符（用于识别 `log!`；不含 `crate::` 前缀也行）。"""
    j = pos
    while j > 0 and (text[j - 1].isalnum() or text[j - 1] == "_"):
        j -= 1
    return text[j:pos]


# ---------------------------------------------------------------- TS/TSX 扫描

def scan_ts(path: Path, text: str, allowed_lines: set[int]) -> list[Violation]:
    out: list[Violation] = []
    i, n = 0, len(text)
    line = 1
    state = "code"  # code | line_comment | block_comment | single | double | template
    buf_start = None

    def advance():
        nonlocal i, line
        if text[i] == "\n":
            line += 1
        i += 1

    def flag(content: str):
        if has_cjk(content) and line not in allowed_lines:
            out.append(Violation(path, line, content))

    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if state == "code":
            if c == "/" and nxt == "/":
                state = "line_comment"
                advance(); advance()
            elif c == "/" and nxt == "*":
                state = "block_comment"
                advance(); advance()
            elif c == "'":
                state = "single"
                buf_start = i + 1
                advance()
            elif c == '"':
                state = "double"
                buf_start = i + 1
                advance()
            elif c == "`":
                state = "template"
                buf_start = i + 1
                advance()
            else:
                if is_cjk(c) and line not in allowed_lines:
                    # JSX 文本（不在字符串里的中文；标识符不可能是 CJK）
                    out.append(Violation(path, line, text[i : i + 1] + "…（JSX 文本）"))
                advance()
        elif state == "line_comment":
            if c == "\n":
                state = "code"
            advance()
        elif state == "block_comment":
            if c == "*" and nxt == "/":
                state = "code"
                advance(); advance()
            else:
                advance()
        elif state in ("single", "double", "template"):
            close = {"single": "'", "double": '"', "template": "`"}[state]
            if c == "\\":
                advance(); advance()
            elif c == close:
                flag(text[buf_start:i])
                state = "code"
                advance()
            else:
                advance()
    return out


# ---------------------------------------------------------------- 汇总

def allowed_lines_of(text: str) -> set[int]:
    """i18n:allow 行内标记豁免该行；i18n:allow-begin … i18n:allow-end 豁免整块。"""
    allowed = {i + 1 for i, l in enumerate(text.splitlines()) if "i18n:allow" in l}
    lines = text.splitlines()
    begin = None
    for i, l in enumerate(lines):
        if "i18n:allow-begin" in l:
            begin = i + 1
        elif "i18n:allow-end" in l and begin is not None:
            allowed.update(range(begin, i + 2))
            begin = None
    return allowed


def files_to_scan():
    for pat in ("crates/**/src/**/*.rs",):
        for p in ROOT.glob(pat):
            yield p
    for p in (ROOT / "crates/mousetroke-settings/ui/src").rglob("*"):
        if p.suffix in (".ts", ".tsx"):
            yield p


def is_test_file(p: Path) -> bool:
    s = str(p).replace("\\", "/")
    return s.endswith("_tests.rs") or "/tests/" in s


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    violations: list[Violation] = []
    for p in sorted(files_to_scan()):
        if is_test_file(p):
            continue
        text = p.read_text(encoding="utf-8")
        allowed = allowed_lines_of(text)
        if p.suffix == ".rs":
            if p.name == "names.rs":
                continue  # 迁移用中英文对照表（词条键以外的存量映射）
            m = re.search(r"^#\[cfg\(test\)\]", text, re.M)
            head = text[: m.start()] if m else text  # #[cfg(test)] 之后是测试
            violations += scan_rust(p, head, allowed)
        else:
            violations += scan_ts(p, text, allowed)

    if violations:
        print(f"i18n lint：{len(violations)} 处写死的中文（应改为词条，见 i18n/README.md）")
        for v in violations:
            print(f"  {v}")
    else:
        print("i18n lint：通过")
    if "--check" in sys.argv and violations:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
