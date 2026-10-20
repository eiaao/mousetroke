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
        # 1.6.5 在 file::add 之前完成 background::create；其回调不携带文件路径。
        # 校验自动合成的 Retina TIFF，不再要求镜像内存在原始 PNG 目录。
        background = path.with_name(".background.tiff")
        with background.open("rb") as stream:
            if stream.read(4) not in (b"II*\x00", b"MM\x00*"):
                raise ValueError(str(background))


if __name__ == "__main__":
    settings, app, image = sys.argv[1:]
    build_dmg(image, "Mousetroke", settings, defines={"app": app, "art": str(Path(settings).resolve().with_name("background"))}, callback=verify_copy)
