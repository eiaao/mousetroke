"""用隔离目录和 Apple 工具替身验证打包失败时不会留下可发布的旧产物。"""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
BASH = (shutil.which("bash", path="C:/Program Files/Git/bin")
        if os.name == "nt" else shutil.which("bash"))

MOCK_TOOL = r"""#!/bin/bash
set -euo pipefail
tool=$(basename "$0")
printf '%s' "$tool" >> "$TEST_LOG"
printf ' <%s>' "$@" >> "$TEST_LOG"
printf '\n' >> "$TEST_LOG"
case "$tool" in
    cargo)
        test ! -e "$PWD/target/mac/Mousetroke.app"
        [[ "${FAIL_CARGO:-0}" != 1 ]] || exit 1
        target=
        while [[ $# -gt 0 ]]; do
            if [[ "$1" == --target ]]; then target="$2/"; shift; fi
            shift
        done
        mkdir -p "$CARGO_TARGET_DIR/${target}release"
        printf 'new binary\n' > "$CARGO_TARGET_DIR/${target}release/mousetroke"
        ;;
    lipo)
        [[ "${FAIL_LIPO:-0}" != 1 ]] || exit 1
        if [[ " $* " == *' -create '* ]]; then
            [[ $# -eq 5 && "$3" == -create && "$4" == -output ]]
            test -f "$1" && test -f "$2"
            printf 'universal binary\n' > "$5"
        elif [[ $# -eq 2 && "$2" == -archs ]]; then
            test -f "$1"
            [[ "${FAIL_ARCHS:-0}" != 1 ]] || exit 1
            printf '%s\n' "${MOCK_SLICES-arm64 x86_64}"
        else
            # macOS 27 CommandLineTools 每次 -verify_arch 只接受一个架构。
            if [[ $# -ne 3 || "$2" != -verify_arch ]]; then
                printf 'lipo: -verify_arch requires exactly one input file\n' >&2
                exit 1
            fi
            [[ "$3" == arm64 || "$3" == x86_64 ]]
            test -f "$1"
            slices=" ${MOCK_SLICES-arm64 x86_64} "
            [[ "$slices" == *" $3 "* ]]
        fi
        ;;
    codesign)
        case "$1" in
            --verify) [[ "${FAIL_VERIFY:-0}" != 1 ]] ;;
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
                [[ "$path" != *"${FAIL_SIGN_SUFFIX:-.never}" ]]
                ;;
            *) exit 2 ;;
        esac
        ;;
    security) printf '1) AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA "Fixture"\n' ;;
    plutil) : ;;
    ditto) cp -R "$1" "$2" ;;
    ln)
        # Windows 的 Git Bash 无法链接到不存在的 macOS /Applications。
        [[ $# -eq 3 && "$1" == -s && "$2" == /Applications ]]
        printf '/Applications\n' > "$3"
        ;;
    hdiutil)
        if [[ "$1" == create ]]; then
            image="$2"
            [[ "$image" == *.dmg && ! -e "$image" ]]
            printf 'fresh DMG\n' > "$image"
            [[ "${FAIL_CREATE:-0}" != 1 ]]
        else
            [[ "$1" == verify && "${FAIL_IMAGE_VERIFY:-0}" != 1 ]]
        fi
        ;;
    *) exit 2 ;;
esac
"""


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
        (self.root / "i18n").mkdir()
        shutil.copyfile(ROOT / "i18n/en-US.json", self.root / "i18n/en-US.json")
        commands = self.root / "commands"
        commands.mkdir()
        for name in ["cargo", "lipo", "codesign", "security", "plutil", "ditto", "ln", "hdiutil"]:
            executable = commands / name
            executable.write_text(MOCK_TOOL, encoding="utf-8", newline="\n")
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
        }
        self.seed_outputs()

    def seed_outputs(self):
        (self.bundle / "Contents/MacOS").mkdir(parents=True, exist_ok=True)
        (self.bundle / "Contents/MacOS/mousetroke").write_text("old binary")
        (self.bundle / "stale-resource").write_text("old resource")
        self.image.write_text("old DMG")
        self.log.write_text("")

    def run_script(self, name, *arguments, success=True, **environment):
        result = subprocess.run(
            [BASH, "--noprofile", "--norc", "-c",
             'export PATH="$PWD/commands:$PATH"; exec "$BASH" --noprofile --norc "$@"',
             "mac-packaging-test", (self.root / f"scripts/mac/{name}.sh").as_posix(), *arguments],
            cwd=self.root, env={**self.env, **environment}, capture_output=True, text=True, timeout=30,
        )
        if success:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

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
            (dict(FAIL_CREATE="1"), "hdiutil <create>"),
            (dict(FAIL_SIGN_SUFFIX=".dmg"), "codesign <--force>"),
            (dict(FAIL_IMAGE_VERIFY="1"), "hdiutil <verify>"),
        ]:
            with self.subTest(failure=failure):
                self.seed_outputs()
                self.run_script("dmg", success=False, **failure)
                self.assertFalse(self.image.exists())
                self.assertIn(stage, self.log.read_text())


if __name__ == "__main__":
    unittest.main()
