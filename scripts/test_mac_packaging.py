"""用隔离目录和 Apple 工具替身验证打包失败时不会留下可发布的旧产物。"""
import os
from pathlib import Path
import re
import runpy
import shutil
import subprocess
import tempfile
import textwrap
import unittest
from unittest.mock import Mock, patch
import types
import struct
import sys
import contextlib
import hashlib
import io
import json


ROOT = Path(__file__).resolve().parents[1]
BASH = (shutil.which("bash", path="C:/Program Files/Git/bin")
        if os.name == "nt" else "/bin/bash" if Path("/bin/bash").is_file() else shutil.which("bash"))

MOCK_TOOL = r"""#!/bin/bash
set -euo pipefail
tool=$(basename "$0") || exit 1
printf '%s' "$tool" >> "$TEST_LOG"
printf ' <%s>' "$@" >> "$TEST_LOG"
printf '\n' >> "$TEST_LOG"
case "$tool" in
    cargo)
        test ! -e "$PWD/target/mac/Mousetroke.app" || exit 1
        [[ "${FAIL_CARGO:-0}" != 1 ]] || exit 1
        target=
        while [[ $# -gt 0 ]]; do
            if [[ "$1" == --target ]]; then target="$2/"; shift; fi
            shift
        done
        mkdir -p "$CARGO_TARGET_DIR/${target}release" || exit 1
        printf 'new binary\n' > "$CARGO_TARGET_DIR/${target}release/mousetroke"
        ;;
    lipo)
        [[ "${FAIL_LIPO:-0}" != 1 ]] || exit 1
        if [[ " $* " == *' -create '* ]]; then
            [[ $# -eq 5 && "$3" == -create && "$4" == -output ]] || exit 1
            { test -f "$1" && test -f "$2"; } || exit 1
            printf 'universal binary\n' > "$5"
        elif [[ $# -eq 2 && "$2" == -archs ]]; then
            test -f "$1" || exit 1
            [[ "${FAIL_ARCHS:-0}" != 1 ]] || exit 1
            printf '%s\n' "${MOCK_SLICES-arm64 x86_64}"
        else
            # macOS 27 CommandLineTools 每次 -verify_arch 只接受一个架构。
            if [[ $# -ne 3 || "$2" != -verify_arch ]]; then
                printf 'lipo: -verify_arch requires exactly one input file\n' >&2
                exit 1
            fi
            [[ "$3" == arm64 || "$3" == x86_64 ]] || exit 1
            test -f "$1" || exit 1
            slices=" ${MOCK_SLICES-arm64 x86_64} "
            [[ "$slices" == *" $3 "* ]] || exit 1
        fi
        ;;
    codesign)
        case "$1" in
            --verify) [[ "${FAIL_VERIFY:-0}" != 1 ]] || exit 1 ;;
            --display)
                [[ "${FAIL_DISPLAY:-0}" != 1 ]] || exit 1
                architecture=
                while [[ $# -gt 0 ]]; do
                    if [[ "$1" == --architecture ]]; then architecture="$2"; shift; fi
                    shift
                done
                if [[ "${MOCK_ADHOC_ARCH:-}" == all || "${MOCK_ADHOC_ARCH:-}" == "$architecture" ]]; then
                    printf 'Signature=adhoc\n' >&2
                else
                    printf 'Authority=Fixture Certificate\n' >&2
                fi
                ;;
            --force)
                path="${!#}"
                [[ "$path" != *"${FAIL_SIGN_SUFFIX:-.never}" ]] || exit 1
                ;;
            *) exit 2 ;;
        esac
        ;;
    security)
        [[ "$1" != "${FAIL_SECURITY:-.never}" ]] || exit 1
        case "$1" in
            find-identity) printf '1) %s "Fixture"\n' "${MOCK_FINGERPRINT-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA}" ;;
            find-certificate) printf 'SHA-1 hash: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n' ;;
            list-keychains)
                if [[ $# -eq 3 ]]; then printf '%s\n' "${MOCK_KEYCHAINS:-}"; fi
                ;;
        esac
        ;;
    openssl)
        [[ "$1" != "${FAIL_OPENSSL:-.never}" ]] || exit 1
        if [[ "$1" == rand ]]; then printf 'fixture-password\n'; fi
        ;;
    git)
        if [[ "${!#}" == --git-common-dir ]]; then
            printf '%s/.git\n' "$PWD"
        else
            # 隔离证书测试模拟仓库外的临时目录；不查询真实 worktree。
            exit 1
        fi
        ;;
    base64)
        [[ "${FAIL_BASE64:-0}" != 1 ]] || exit 1
        cat || exit 1
        ;;
    plutil) [[ "${FAIL_PLUTIL:-0}" != 1 ]] || exit 1 ;;
    ditto)
        [[ "${FAIL_DITTO:-0}" != 1 ]] || exit 1
        cp -R "$1" "$2" || exit 1
        ;;
    ln)
        # Windows 的 Git Bash 无法链接到不存在的 macOS /Applications。
        [[ $# -eq 3 && "$1" == -s && "$2" == /Applications ]] || exit 1
        printf '/Applications\n' > "$3"
        ;;
    python3)
        [[ $# -eq 4 && "$1" == */build_dmg.py && "$2" == */dmg_settings.py ]] || exit 1
        test -f "$2" || exit 1
        test -f "$(dirname "$2")/background/background.png" || exit 1
        test -f "$(dirname "$2")/background/background@2x.png" || exit 1
        test -f "$3/Contents/MacOS/mousetroke" || exit 1
        hdiutil create "$4" || exit 1
        [[ "${FAIL_DMGBUILD:-0}" != 1 ]] || exit 1
        ;;
    hdiutil)
        if [[ "$1" == create ]]; then
            image="$2"
            [[ "$image" == *.dmg && ! -e "$image" ]] || exit 1
            printf 'fresh DMG\n' > "$image"
            [[ "${FAIL_CREATE:-0}" != 1 ]] || exit 1
        else
            [[ "$1" == verify && "${FAIL_IMAGE_VERIFY:-0}" != 1 ]] || exit 1
        fi
        ;;
    *) exit 2 ;;
esac
"""


# 有意只识别常见的不兼容语法；真正的 3.2 运行和 bash -n 仍是最终校验。
BASH4_FEATURES = {
    "associative/global arrays or namerefs": r"\b(?:declare|local|typeset)\s+[^;\n]*-[a-zA-Z]*[Agn][a-zA-Z]*(?:\s|$)",
    "mapfile/readarray/coproc": r"\b(?:mapfile|readarray|coproc)\b",
    "case conversion": r"\$\{[^}\n]*(?:,|\^)[^}\n]*\}",
    "new redirections/case fallthrough": r"\|&|&>>|;;&|;&",
    "negative array indices": r"\b\w+\[\s*-\d",
    "dynamic file descriptors": r"\{[A-Za-z_]\w*\}[<>]",
    "variable transformations": r"\$\{[^}\n]*@[A-Za-z][^}\n]*\}",
    "variable existence test": r"\[\[\s*(?:!\s*)?-(?:v|R)\s",
    "wait -n / read -N": r"\b(?:wait\s+-[a-zA-Z]*n|read\s+-[a-zA-Z]*N)\b",
    "new shopt options": r"\bshopt\s+[^;\n]*\b(?:globstar|lastpipe|direxpand|localvar_inherit|localvar_unset)\b",
    "masked command substitution": r"\b(?:local|declare|typeset|export|readonly)\s+[^;\n]*=\$\(",
    "unguarded standalone predicate": r"^\s*(?:\[\[.*\]\]|\(\(.*\)\)|test\s+(?:(?!\|\||&&|;).)+|\[\s+.*\])\s*$",
}


def shell_compatibility_issues(source):
    # 注释不是可执行代码；避免文档里的特性名称造成误报。
    source = "\n".join(line for line in source.splitlines() if not line.lstrip().startswith("#"))
    return [name for name, pattern in BASH4_FEATURES.items() if re.search(pattern, source, re.MULTILINE)]


def workflow_bash_blocks():
    """读取本仓库的简单 run 标量，不引入 PyYAML；排除 Windows 的 PowerShell/CMD。"""
    for path in sorted((ROOT / ".github/workflows").glob("*.y*ml")):
        source = path.read_text(encoding="utf-8").split("\njobs:\n", 1)[1]
        jobs = re.split(r"(?m)^  [\w-]+:\s*\n", source)[1:]
        for job in jobs:
            if re.search(r"runs-on:\s*windows", job):
                continue
            macos = bool(re.search(r"runs-on:\s*macos", job))
            lines = job.splitlines()
            step = ""
            for number, line in enumerate(lines):
                if line.lstrip().startswith("- name:"):
                    step = line.strip()[7:].strip()
                match = re.match(r"(\s+)run:\s*(.*)", line)
                if not match:
                    continue
                indent, value = match.groups()
                if not value:  # defaults.run 配置，不是 step 的 run 标量。
                    continue
                if value in ("|", ">", "|-", ">-"):
                    block = []
                    for following in lines[number + 1:]:
                        if following.strip() and len(following) - len(following.lstrip()) <= len(indent):
                            break
                        block.append(following)
                    value = textwrap.dedent("\n".join(block))
                value = re.sub(r"\$\{\{.*?\}\}", "workflow-value", value)
                yield f"{path.name}: {step}", value, macos


class DmgLayoutTests(unittest.TestCase):
    def test_layout_loads_like_dmgbuild_and_includes_retina_art(self):
        # dmgbuild 用 exec 加载设置，不提供 __file__。
        settings = {"defines": {"app": "/tmp/staged/Mousetroke.app", "art": str(ROOT / "scripts/mac/background")}}
        exec(compile((ROOT / "scripts/mac/dmg_settings.py").read_text(encoding="utf-8"), "dmg_settings.py", "exec"), settings)
        self.assertEqual(settings["window_rect"][1], (640, 400))
        self.assertEqual(settings["icon_locations"], {"Mousetroke.app": (160, 200), "Applications": (480, 200)})
        self.assertEqual(settings["symlinks"], {"Applications": "/Applications"})
        self.assertEqual(settings["files"], ["/tmp/staged/Mousetroke.app"])
        self.assertEqual(settings["background"], str(ROOT / "scripts/mac/background") + "/background.png")
        self.assertEqual(settings["icon_size"], 128)
        self.assertEqual(settings["format"], "UDZO")
        for key in ["show_toolbar", "show_sidebar", "show_status_bar", "show_pathbar", "show_tab_view"]:
            self.assertFalse(settings[key], key)
        for suffix, size in [("", (640, 400)), ("@2x", (1280, 800))]:
            data = (ROOT / f"scripts/mac/background/background{suffix}.png").read_bytes()
            self.assertEqual(data[:8], b"\x89PNG\r\n\x1a\n")
            self.assertEqual(struct.unpack(">II", data[16:24]), size)

    def test_image_copy_is_verified_even_if_dmgbuild_ignores_ditto_status(self):
        module = types.ModuleType("dmgbuild")
        module.build_dmg = Mock()
        with patch.dict(sys.modules, {"dmgbuild": module}):
            source = runpy.run_path(str(ROOT / "scripts/mac/build_dmg.py"))
        with tempfile.TemporaryDirectory() as directory:
            app = Path(directory) / "Mousetroke.app"
            background = app.with_name(".background.tiff")
            event = {"type": "operation::finished", "operation": "file::add", "file": str(app)}
            for header in (b"II*\x00", b"MM\x00*"):
                background.write_bytes(header)
                with patch("subprocess.run") as run:
                    source["verify_copy"](event)
                    self.assertEqual(run.call_args.args[0], ["codesign", "--verify", "--strict", "--all-architectures", str(app)])
                    self.assertTrue(run.call_args.kwargs["check"])
            with patch("subprocess.run", side_effect=subprocess.CalledProcessError(1, "codesign")):
                with self.assertRaises(subprocess.CalledProcessError):
                    source["verify_copy"](event)
            # dmgbuild 自动生成的 TIFF 缺失或损坏时也必须终止打包。
            with patch("subprocess.run"):
                background.write_bytes(b"invalid TIFF")
                with self.assertRaises(ValueError):
                    source["verify_copy"](event)
                background.unlink()
                with self.assertRaises(FileNotFoundError):
                    source["verify_copy"](event)


class MacShellCompatibilityTests(unittest.TestCase):
    def test_compatibility_guard_detects_bash4_and_masked_failures(self):
        for source in ["declare -A items", "local -n reference=items", "mapfile -t items",
                       "readarray items", "coproc command", "echo ${value,,}", "echo ${value^^}",
                       "a |& b", "a &>> log", "case x in x) a ;& esac", "echo ${items[-1]}",
                       "exec {fd}>file", "echo ${value@Q}", "[[ -v value ]]", "wait -n",
                       "shopt -s globstar", "local value=$(false)", "[[ 1 == 2 ]]", "(( count++ ))"]:
            with self.subTest(source=source):
                self.assertTrue(shell_compatibility_issues(source), source)

    def test_mac_sources_avoid_bash4_and_unguarded_checks(self):
        sources = [(path.name, path.read_text(encoding="utf-8"))
                   for path in sorted((ROOT / "scripts/mac").glob("*.sh"))]
        sources += [(name, code) for name, code, macos in workflow_bash_blocks() if macos]
        sources.append(("MOCK_TOOL", MOCK_TOOL))
        for name, code in sources:
            with self.subTest(source=name):
                self.assertEqual(shell_compatibility_issues(code), [], name)

    @unittest.skipUnless(BASH, "Bash is required")
    def test_shell_and_workflow_bash_syntax(self):
        sources = [(path.name, path.read_text(encoding="utf-8"))
                   for path in sorted((ROOT / "scripts/mac").glob("*.sh"))]
        sources += [(name, code) for name, code, _ in workflow_bash_blocks()]
        sources.append(("MOCK_TOOL", MOCK_TOOL))
        for name, code in sources:
            with self.subTest(source=name):
                result = subprocess.run([BASH, "--noprofile", "--norc", "-n"], input=code,
                                        text=True, capture_output=True, timeout=30)
                self.assertEqual(result.returncode, 0, f"{name}: {result.stderr}")


@unittest.skipUnless(BASH, "Bash is required (Git Bash on Windows)")
class MacPackagingTests(unittest.TestCase):
    def setUp(self):
        # 临时内容仅写入当前 worktree 的忽略目录；路径包含空格。
        (ROOT / "target").mkdir(exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix="mac packaging ", dir=ROOT / "target")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        shutil.copytree(ROOT / "scripts/mac", self.root / "scripts/mac")
        shutil.copyfile(ROOT / "Cargo.toml", self.root / "Cargo.toml")
        icons = self.root / "crates/mousetroke-settings/icons"
        icons.mkdir(parents=True)
        shutil.copyfile(ROOT / "crates/mousetroke-settings/icons/icon.icns", icons / "icon.icns")
        (self.root / "i18n").mkdir()
        shutil.copyfile(ROOT / "i18n/en-US.json", self.root / "i18n/en-US.json")
        commands = self.root / "commands"
        commands.mkdir()
        for name in ["cargo", "lipo", "codesign", "security", "openssl", "git", "base64", "plutil", "ditto", "ln", "hdiutil", "python3"]:
            executable = commands / name
            with executable.open("w", encoding="utf-8", newline="\n") as stream:
                stream.write(MOCK_TOOL)
            executable.chmod(0o755)
        self.bundle = self.root / "target/mac/Mousetroke.app"
        self.image = self.root / "target/mac/Mousetroke.dmg"
        self.log = self.root / "commands.log"
        self.env = {
            **os.environ,
            "TMPDIR": self.root.as_posix(),
            "LC_ALL": "C",
            "TEST_LOG": self.log.as_posix(),
            "CARGO_TARGET_DIR": (self.root / "build").as_posix(),
            "CARGO_BUILD_JOBS": "4",
            "MOUSETROKE_SIGN_ID": "Fixture",
            "MOUSETROKE_SIGN_KEYCHAIN": "",
            "MOUSETROKE_REQUIRE_SIGNING": "0",
            # 不读取测试机器的真实开发证书/钥匙串。
            "HOME": (self.root / "home").as_posix(),
        }
        self.seed_outputs()

    def seed_outputs(self):
        (self.bundle / "Contents/MacOS").mkdir(parents=True, exist_ok=True)
        (self.bundle / "Contents/MacOS/mousetroke").write_text("old binary")
        (self.bundle / "stale-resource").write_text("old resource")
        self.image.write_text("old DMG")
        self.log.write_text("")

    def run_script(self, name, *arguments, success=True, input=None, **environment):
        return self.run_shell('exec "$BASH" --noprofile --norc "$@"',
                              (self.root / f"scripts/mac/{name}.sh").as_posix(), *arguments,
                              success=success, input=input, **environment)

    def run_shell(self, source, *arguments, success=True, input=None, **environment):
        result = subprocess.run(
            [BASH, "--noprofile", "--norc", "-c",
             'export PATH="$PWD/commands:$PATH";\n' + source,
             "mac-packaging-test", *arguments],
            cwd=self.root, env={**self.env, **environment}, input=input,
            capture_output=True, text=True, timeout=30,
        )
        if success:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    def workflow_step(self, name):
        return next(code for label, code, _ in workflow_bash_blocks()
                    if label == f"release.yml: {name}")

    def test_universal_bundle_replaces_old_resources_and_uses_apple_lipo_syntax(self):
        self.run_script("bundle", "--universal")
        self.assertFalse((self.bundle / "stale-resource").exists())
        self.assertEqual((self.bundle / "Contents/MacOS/mousetroke").read_text(), "universal binary\n")
        self.assert_universal_verification()

    def assert_universal_verification(self):
        operations = [line.split("> ", 1)[1] for line in self.log.read_text().splitlines()
                      if line.startswith("lipo <") and " <-create>" not in line]
        self.assertEqual(operations, ["<-verify_arch> <arm64>", "<-verify_arch> <x86_64>", "<-archs>"])

    def test_lipo_fake_rejects_multiple_architectures(self):
        binary = (self.bundle / "Contents/MacOS/mousetroke").as_posix()
        for arguments in [(binary, "-verify_arch", "arm64", "x86_64"),
                          ("-verify_arch", "arm64", "x86_64", binary)]:
            with self.subTest(arguments=arguments):
                result = subprocess.run(
                    [BASH, (self.root / "commands/lipo").as_posix(), *arguments],
                    cwd=self.root, env=self.env, capture_output=True, text=True, timeout=30,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("-verify_arch requires exactly one input file", result.stderr)

    def test_bundle_rejects_missing_extra_or_duplicate_architectures(self):
        for slices in ["arm64", "x86_64", "arm64 x86_64 arm64e", "arm64 x86_64 x86_64", ""]:
            with self.subTest(slices=slices):
                self.seed_outputs()
                self.run_script("bundle", "--universal", success=False, MOCK_SLICES=slices)
                self.assertFalse(self.bundle.exists())
                self.assertNotIn("codesign", self.log.read_text())

    def test_bundle_and_dmg_accept_reversed_architecture_order(self):
        for name, arguments in [("bundle", ("--universal",)), ("dmg", ())]:
            with self.subTest(script=name):
                self.seed_outputs()
                self.run_script(name, *arguments, MOCK_SLICES="x86_64 arm64")
                self.assert_universal_verification()

    def test_architecture_listing_failure_stops_bundle_and_dmg(self):
        for name, arguments, output in [("bundle", ("--universal",), self.bundle), ("dmg", (), self.image)]:
            with self.subTest(script=name):
                self.seed_outputs()
                self.run_script(name, *arguments, success=False, FAIL_ARCHS="1")
                self.assert_universal_verification()
                self.assertFalse(output.exists())
                self.assertNotIn("codesign", self.log.read_text())
                self.assertNotIn("hdiutil", self.log.read_text())

    def test_failed_bundle_removes_old_or_partial_app(self):
        for failure, stage in [
            (dict(FAIL_CARGO="1"), "cargo <build>"),
            (dict(FAIL_LIPO="1"), "lipo <"),
            (dict(FAIL_SIGN_SUFFIX=".app"), "codesign <--force>"),
        ]:
            with self.subTest(failure=failure):
                self.seed_outputs()
                self.run_script("bundle", "--universal", success=False, **failure)
                self.assertFalse(self.bundle.exists())
                self.assertIn(stage, self.log.read_text())

    def test_plist_preview_preserves_existing_bundle(self):
        self.assertIn("<plist", self.run_script("bundle", "--print-plist").stdout)
        self.assertTrue((self.bundle / "stale-resource").exists())
        self.assertEqual(self.log.read_text(), "")

    def test_plist_preview_and_validation_propagate_failures(self):
        self.run_script("bundle", "--universal", success=False, FAIL_PLUTIL="1")
        self.assertFalse(self.bundle.exists())
        self.assertNotIn("codesign", self.log.read_text())
        self.seed_outputs()
        (self.root / "scripts/mac/Info.plist.in").unlink()
        self.run_script("bundle", "--print-plist", success=False)
        self.assertTrue((self.bundle / "stale-resource").exists())

    def test_signing_query_failures_and_malformed_fingerprints_stop_packaging(self):
        for name, arguments, output in [("bundle", ("--universal",), self.bundle), ("dmg", (), self.image)]:
            for failure in [dict(FAIL_SECURITY="find-identity"), dict(MOCK_FINGERPRINT="invalid")]:
                with self.subTest(script=name, failure=failure):
                    self.seed_outputs()
                    self.run_script(name, *arguments, success=False, **failure)
                    self.assertFalse(output.exists())
                    self.assertNotIn("codesign <--force>", self.log.read_text())
                    self.assertNotIn("hdiutil <verify>", self.log.read_text())
                    if name == "bundle":
                        self.assertNotIn("hdiutil", self.log.read_text())

    def test_signing_function_propagates_failures_when_called_in_a_condition(self):
        source = '''set -euo pipefail
root=$PWD
source "$root/scripts/mac/messages.sh" || exit 1
source "$root/scripts/mac/sign.sh" || exit 1
if sign_mac "$root/target/mac/Mousetroke.app"; then exit 0; else exit 1; fi
'''
        for failure in [dict(FAIL_SIGN_SUFFIX=".app"), dict(FAIL_VERIFY="1"),
                        dict(FAIL_SECURITY="find-identity"), dict(MOCK_FINGERPRINT="invalid")]:
            with self.subTest(failure=failure):
                self.seed_outputs()
                self.run_shell(source, success=False, **failure)
                if "FAIL_SIGN_SUFFIX" in failure:
                    self.assertNotIn("codesign <--verify>", self.log.read_text())

    def test_message_function_propagates_missing_file_or_key_in_a_condition(self):
        source = '''set -euo pipefail
root=$PWD
source "$root/scripts/mac/messages.sh" || exit 1
if message=$(mac_message mac.bundle.done); then exit 0; else exit 1; fi
'''
        catalog = self.root / "i18n/en-US.json"
        catalog.write_text("{}", encoding="utf-8")
        self.run_shell(source, success=False)
        catalog.unlink()
        self.run_shell(source, success=False)

    def test_development_signing_stops_on_unlock_and_certificate_query_failure(self):
        home = self.root / "home"
        (home / "Library/Keychains").mkdir(parents=True)
        (home / "Library/Keychains/mousetroke-dev.keychain-db").touch()
        (home / ".config/mousetroke-dev").mkdir(parents=True)
        (home / ".config/mousetroke-dev/keychain-password").write_text("fixture-password")
        for operation in ["unlock-keychain", "find-certificate"]:
            with self.subTest(operation=operation):
                self.seed_outputs()
                self.run_script("bundle", "--universal", success=False,
                                MOUSETROKE_SIGN_ID="", FAIL_SECURITY=operation)
                self.assertFalse(self.bundle.exists())
                self.assertNotIn("codesign", self.log.read_text())

    def test_dev_certificate_preserves_keychain_list_and_accepts_empty_list(self):
        for keychains in ["", '    "existing keychain"', '    "existing keychain"\n']:
            with self.subTest(keychains=keychains):
                self.log.write_text("")
                self.run_script("make-dev-cert", MOCK_KEYCHAINS=keychains)
                last = self.log.read_text().splitlines()[-1]
                self.assertIn("security <list-keychains> <-d> <user> <-s>", last)
                if keychains:
                    self.assertIn("<existing keychain>", last)

    def test_dev_certificate_query_and_generation_failures_stop_before_success(self):
        for failure in [dict(FAIL_SECURITY="list-keychains"), dict(FAIL_OPENSSL="rand"),
                        dict(FAIL_OPENSSL="req"), dict(FAIL_OPENSSL="pkcs12")]:
            with self.subTest(failure=failure):
                self.log.write_text("")
                self.run_script("make-dev-cert", success=False, **failure)
                self.assertNotIn("security <list-keychains> <-d> <user> <-s>", self.log.read_text())

    def test_release_certificate_rejects_unreadable_empty_or_mismatched_password(self):
        (self.root / ".git").mkdir()
        # 输出位于 worktree 的忽略目录，但在模拟的证书脚本仓库之外。
        with tempfile.TemporaryDirectory(prefix="cert output ", dir=ROOT / "target") as output:
            certificate = (Path(output) / "release.p12").as_posix()
            for password in ["", "\n", "one\ntwo\n", "one\n"]:
                with self.subTest(password=password):
                    self.run_script("make-release-cert", certificate, success=False, input=password)
                    self.assertNotIn("openssl", self.log.read_text())
            self.run_script("make-release-cert", certificate, success=False,
                            input="one\none\n", FAIL_OPENSSL="req")
            self.assertNotIn("openssl <pkcs12>", self.log.read_text())

    def test_release_workflow_import_stops_on_tool_or_decode_failure(self):
        runner = self.root / "runner"
        runner.mkdir()
        github_env = runner / "github-env"
        for failure in [dict(FAIL_OPENSSL="rand"), dict(FAIL_SECURITY="list-keychains"),
                        dict(FAIL_BASE64="1"), dict(FAIL_SECURITY="create-keychain"),
                        dict(FAIL_SECURITY="import"), dict(FAIL_SECURITY="set-key-partition-list")]:
            with self.subTest(failure=failure):
                self.log.write_text("")
                self.run_shell(self.workflow_step("Import stable release identity"), success=False,
                               RUNNER_TEMP=runner.as_posix(), GITHUB_ENV=github_env.as_posix(),
                               MAC_SIGN_P12_BASE64="fixture", MAC_SIGN_P12_PASSWORD="fixture", **failure)
                self.assertNotIn("security <list-keychains> <-d> <user> <-s>", self.log.read_text())
                self.assertFalse(github_env.exists())

    def test_release_workflow_import_exports_identity_only_after_success(self):
        runner = self.root / "runner"
        runner.mkdir()
        github_env = runner / "github-env"
        self.run_shell(self.workflow_step("Import stable release identity"),
                       RUNNER_TEMP=runner.as_posix(), GITHUB_ENV=github_env.as_posix(),
                       MAC_SIGN_P12_BASE64="fixture", MAC_SIGN_P12_PASSWORD="fixture")
        self.assertIn("MOUSETROKE_SIGN_ID=Mousetroke Self-Signed", github_env.read_text())
        self.assertIn("security <import>", self.log.read_text())
        self.assertFalse((runner / "mousetroke-release.p12").exists())

    def test_release_workflow_cleanup_handles_empty_lists_and_preserves_failures(self):
        runner = self.root / "runner"
        runner.mkdir()
        for keychains, failure in [("", {}), ('    "existing keychain"\n', {}),
                                   ("", dict(FAIL_SECURITY="list-keychains")),
                                   ("", dict(FAIL_SECURITY="delete-keychain"))]:
            with self.subTest(keychains=keychains, failure=failure):
                (runner / "mousetroke-keychains.txt").write_text(keychains, encoding="utf-8")
                (runner / "mousetroke-release.keychain-db").touch()
                (runner / "mousetroke-release.p12").touch()
                self.run_shell(self.workflow_step("Delete temporary signing material"),
                               success=not failure, RUNNER_TEMP=runner.as_posix(), **failure)
                self.assertFalse((runner / "mousetroke-keychains.txt").exists())
                self.assertFalse((runner / "mousetroke-release.p12").exists())
                self.assertIn("security <delete-keychain>", self.log.read_text())

    def test_failed_bundle_cannot_feed_stale_app_into_dmg(self):
        self.run_script("bundle", "--universal", success=False, FAIL_CARGO="1")
        self.run_script("dmg", success=False)
        self.assertFalse(self.bundle.exists())
        self.assertFalse(self.image.exists())
        self.assertNotIn("hdiutil", self.log.read_text())

    def test_missing_app_removes_old_image_before_failing(self):
        shutil.rmtree(self.bundle)
        self.run_script("dmg", success=False)
        self.assertFalse(self.image.exists())
        self.assertNotIn("hdiutil", self.log.read_text())

    def test_dmg_rejects_missing_slice_or_invalid_signature_before_creation(self):
        for failure in [dict(MOCK_SLICES="arm64"), dict(MOCK_SLICES="x86_64"), dict(FAIL_VERIFY="1")]:
            with self.subTest(failure=failure):
                self.seed_outputs()
                self.run_script("dmg", success=False, **failure)
                self.assertFalse(self.image.exists())
                self.assertNotIn("hdiutil", self.log.read_text())

    def test_dmg_rejects_extra_duplicate_or_empty_architectures_before_creation(self):
        for slices in ["arm64 x86_64 arm64e", "arm64 x86_64 arm64", ""]:
            with self.subTest(slices=slices):
                self.seed_outputs()
                self.run_script("dmg", success=False, MOCK_SLICES=slices)
                self.assertFalse(self.image.exists())
                self.assertNotIn("codesign", self.log.read_text())
                self.assertNotIn("hdiutil", self.log.read_text())

    def test_release_rejects_adhoc_signature_in_either_slice(self):
        for architecture in ["arm64", "x86_64", "all"]:
            with self.subTest(architecture=architecture):
                self.seed_outputs()
                self.run_script("dmg", success=False, MOUSETROKE_REQUIRE_SIGNING="1", MOCK_ADHOC_ARCH=architecture)
                self.assertFalse(self.image.exists())
                self.assertNotIn("hdiutil", self.log.read_text())

    def test_release_rejects_unreadable_signature_metadata(self):
        self.run_script("dmg", success=False, MOUSETROKE_REQUIRE_SIGNING="1", FAIL_DISPLAY="1")
        self.assertFalse(self.image.exists())
        self.assertNotIn("hdiutil", self.log.read_text())

    def test_universal_adhoc_smoke_and_certificate_release_create_fresh_image(self):
        for environment in [
            dict(MOUSETROKE_SIGN_ID="-", MOCK_ADHOC_ARCH="all"),
            dict(MOUSETROKE_REQUIRE_SIGNING="1"),
        ]:
            with self.subTest(environment=environment):
                self.seed_outputs()
                self.run_script("dmg", **environment)
                self.assertEqual(self.image.read_text(), "fresh DMG\n")
                self.assertIn("hdiutil <verify>", self.log.read_text())
                self.assert_universal_verification()

    def test_failed_image_creation_signing_or_verification_removes_partial_image(self):
        for failure, stage in [
            (dict(FAIL_DITTO="1"), "ditto <"),
            (dict(FAIL_CREATE="1"), "hdiutil <create>"),
            (dict(FAIL_DMGBUILD="1"), "python3 <"),
            (dict(FAIL_SIGN_SUFFIX=".dmg"), "codesign <--force>"),
            (dict(FAIL_IMAGE_VERIFY="1"), "hdiutil <verify>"),
        ]:
            with self.subTest(failure=failure):
                self.seed_outputs()
                self.run_script("dmg", success=False, **failure)
                self.assertFalse(self.image.exists())
                self.assertIn(stage, self.log.read_text())


class UpdateFixtureTests(unittest.TestCase):
    def fixture(self, *options):
        with patch.object(sys, "argv", ["update-test-server.py", *map(str, options)]), \
                patch("http.server.ThreadingHTTPServer") as server, \
                contextlib.redirect_stdout(io.StringIO()):
            server.return_value.serve_forever.side_effect = KeyboardInterrupt
            runpy.run_path(str(ROOT / "scripts/update-test-server.py"), run_name="__main__")
        self.assertEqual(server.call_args.args[0], ("127.0.0.1", 8770))
        handler = server.call_args.args[1]

        def get(path):
            request = types.SimpleNamespace(path=path, wfile=io.BytesIO(),
                                            send_response=Mock(), send_header=Mock(), end_headers=Mock())
            handler.do_GET(request)
            request.send_response.assert_called_once_with(200)
            body = request.wfile.getvalue()
            request.send_header.assert_called_once_with("Content-Length", str(len(body)))
            return body
        return get

    def test_windows_mac_and_mixed_fixtures_serve_exact_payloads_and_checksums(self):
        with tempfile.TemporaryDirectory() as directory:
            exe, dmg = Path(directory) / "input.exe", Path(directory) / "input.dmg"
            exe.write_bytes(b"Windows fixture")
            dmg.write_bytes(b"Mac fixture")
            for options, assets in [(["--exe", exe], {"mousetroke.exe": exe}),
                                    (["--dmg", dmg], {"Mousetroke.dmg": dmg}),
                                    (["--exe", exe, "--dmg", dmg], {"mousetroke.exe": exe, "Mousetroke.dmg": dmg})]:
                with self.subTest(options=options):
                    get = self.fixture(*options, "--version", "1.2.3")
                    release = json.loads(get("/repos/eiaao/mousetroke/releases/latest"))
                    self.assertEqual(release["tag_name"], "v1.2.3")
                    self.assertEqual({a["name"] for a in release["assets"]}, set(assets) | {"SHA256SUMS.txt"})
                    sums = get("/SHA256SUMS.txt").decode()
                    for name, source in assets.items():
                        payload = source.read_bytes()
                        self.assertEqual(get("/" + name), payload)
                        self.assertIn(f"{hashlib.sha256(payload).hexdigest()} *{name}\n", sums)
                    bad = self.fixture(*options, "--bad-checksum")("/SHA256SUMS.txt").decode()
                    self.assertTrue(all(line.startswith("0" * 64) for line in bad.splitlines()))

    def test_fixture_requires_a_payload(self):
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as error:
            self.fixture()
        self.assertEqual(error.exception.code, 2)


if __name__ == "__main__":
    unittest.main()
