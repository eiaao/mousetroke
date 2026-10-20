"""生成 Mousetroke 图标（与托盘图标同一字形：圆角方块 + 白色手势折线）。

用法: python scripts/gen_icon.py <输出.ico|输出.icns>
.icns 按 macOS 图标规范四周留白（1024 画布里主体 824），与系统其他应用图标大小一致。
纯标准库实现（zlib + struct），不依赖 PIL。
"""
import math, struct, sys, zlib

PATH = [(7.0, 11.0), (13.5, 21.0), (19.0, 13.0), (25.0, 21.0)]  # 32px 设计稿坐标
BG = (0.0, 0.62, 0.95)


def seg_dist(p, a, b):
    abx, aby = b[0] - a[0], b[1] - a[1]
    t = max(0.0, min(1.0, ((p[0] - a[0]) * abx + (p[1] - a[1]) * aby) / (abx * abx + aby * aby)))
    qx, qy = a[0] + abx * t, a[1] + aby * t
    return math.hypot(p[0] - qx, p[1] - qy)


def render(n, inset=0.0):
    # inset：四周留白占边长的比例；圆角方块与折线都在留白内的主体里绘制
    o = n * inset
    body = n - 2 * o
    k = body / 32.0
    r = 7.0 * k
    path = [(o + x * k, o + y * k) for x, y in PATH]
    half = 1.8 * k
    rows = []
    for y in range(n):
        row = bytearray([0])  # filter: none
        for x in range(n):
            # 4x 超采样
            acc = [0.0, 0.0, 0.0, 0.0]
            for sy in range(4):
                for sx in range(4):
                    fx, fy = x + (sx + 0.5) / 4, y + (sy + 0.5) / 4
                    if not (o <= fx <= n - o and o <= fy <= n - o):
                        continue
                    cx, cy = min(max(fx, o + r), n - o - r), min(max(fy, o + r), n - o - r)
                    inside = math.hypot(fx - cx, fy - cy) <= r
                    if not inside:
                        continue
                    d = min(seg_dist((fx, fy), path[i], path[i + 1]) for i in range(len(path) - 1))
                    c = (1.0, 1.0, 1.0) if d <= half else BG
                    acc[0] += c[0]; acc[1] += c[1]; acc[2] += c[2]; acc[3] += 1
            a = acc[3] / 16
            if a > 0:
                rgb = [int(round(acc[i] / acc[3] * 255)) for i in range(3)]
            else:
                rgb = [0, 0, 0]
            row += bytes(rgb + [int(round(a * 255))])
        rows.append(bytes(row))
    return b"".join(rows)


def png(n, inset=0.0):
    raw = render(n, inset)
    def chunk(t, d):
        c = struct.pack(">I", len(d)) + t + d
        return c + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", n, n, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def ico(sizes):
    imgs = [png(s) for s in sizes]
    out = struct.pack("<HHH", 0, 1, len(imgs))
    off = 6 + 16 * len(imgs)
    for s, data in zip(sizes, imgs):
        out += struct.pack("<BBBBHHII", s % 256, s % 256, 0, 0, 1, 32, len(data), off)
        off += len(data)
    return out + b"".join(imgs)


# macOS 图标：每种 OSType 存一张 PNG（macOS 11 起都支持 PNG 数据）
ICNS_TYPES = [(b"icp4", 16), (b"icp5", 32), (b"icp6", 64), (b"ic07", 128), (b"ic08", 256),
              (b"ic09", 512), (b"ic10", 1024), (b"ic11", 32), (b"ic12", 64), (b"ic13", 256), (b"ic14", 512)]
MAC_INSET = 100 / 1024


def icns():
    cache = {}
    body = b""
    for kind, size in ICNS_TYPES:
        if size not in cache:
            cache[size] = png(size, MAC_INSET)
        body += kind + struct.pack(">I", 8 + len(cache[size])) + cache[size]
    return b"icns" + struct.pack(">I", 8 + len(body)) + body


if __name__ == "__main__":
    dst = sys.argv[1]
    with open(dst, "wb") as f:
        f.write(icns() if dst.endswith(".icns") else ico([16, 24, 32, 48, 64, 128, 256]))
    print("written", dst)
