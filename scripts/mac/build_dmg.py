"""调用 dmgbuild，并校验镜像内副本，防止其忽略 ditto 失败后继续生成镜像。"""
from pathlib import Path
import subprocess
import sys

from dmgbuild import build_dmg


def verify_copy(event):
    if event.get("type") != "operation::finished" or event.get("operation") != "file::add":
        return
    path = Path(event["file"])
    if path.suffix == ".app":
        subprocess.run(["codesign", "--verify", "--strict", "--all-architectures", str(path)], check=True)
    elif path.name == ".background":
        for name in ("background.png", "background@2x.png"):
            # 上游 ditto 返回码未检查；缺图时也必须终止打包。
            with (path / name).open("rb") as stream:
                if stream.read(8) != b"\x89PNG\r\n\x1a\n":
                    raise ValueError(str(path / name))


if __name__ == "__main__":
    settings, app, image = sys.argv[1:]
    build_dmg(image, "Mousetroke", settings, defines={"app": app, "art": str(Path(settings).resolve().with_name("background"))}, callback=verify_copy)
