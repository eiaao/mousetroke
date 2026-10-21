"""用 Pillow 生成 DMG 的 1x/2x 背景；使用固定的品牌浅色主题颜色。

用法：python scripts/gen_dmg_background.py [--font 字体.ttf]
开发时需要 Pillow；发布和 Mac mini 打包直接使用已提交的 PNG，无需 Pillow。
"""
import argparse
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
BG = "#e9edf2"
SURFACE = "#ffffff"
INK = "#10151f"
MUTED = "#5b6576"
TRAIL = "#0098d9"


def render(scale, font_path):
    image = Image.new("RGB", (640 * scale, 400 * scale), BG)
    draw = ImageDraw.Draw(image)

    def text(y, value, size, color):
        font = ImageFont.truetype(str(font_path), round(size * scale))
        draw.text((320 * scale, y * scale), value, font=font, fill=color, anchor="mt")

    text(45, "Drag Mousetroke to Applications", 26, INK)
    draw.rounded_rectangle((32 * scale, 110 * scale, 608 * scale, 308 * scale), radius=24 * scale, fill=SURFACE)
    # 访达图标由 dmgbuild 放到 (160, 200) 和 (480, 200)，背景只画两者之间的箭头。
    # 箭身与实心箭头共用一条轮廓，避免两条粗线交叠时箭身伸出尖端。
    arrow = [(286, 197.5), (336, 197.5), (336, 184), (352, 200),
             (336, 216), (336, 202.5), (286, 202.5)]
    draw.polygon([(x * scale, y * scale) for x, y in arrow], fill=TRAIL)
    text(344, "First open blocked? System Settings → Privacy & Security → Open Anyway", 12, MUTED)
    # 固定逻辑尺寸，dmgbuild 用 tiffutil 将两张图合成 Retina TIFF。
    output = ROOT / "scripts/mac/background" / ("background@2x.png" if scale == 2 else "background.png")
    output.parent.mkdir(parents=True, exist_ok=True)
    image.save(output, dpi=(72 * scale, 72 * scale), optimize=True)
    print(output.relative_to(ROOT))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--font", type=Path)
    args = parser.parse_args()
    fonts = [args.font] if args.font else [
        Path("C:/Windows/Fonts/segoeui.ttf"),
        Path("/System/Library/Fonts/Supplemental/Arial.ttf"),
        Path("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"),
    ]
    font = next((path for path in fonts if path.is_file()), None)
    if font is None:
        parser.error("Specify --font with a TrueType font that includes the right arrow")
    for scale in (1, 2):
        render(scale, font)


if __name__ == "__main__":
    main()
