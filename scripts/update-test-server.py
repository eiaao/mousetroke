#!/usr/bin/env python3
"""调试版更新回归服务：只监听回环地址，不写入发布文件，不执行重启。"""

import argparse
import hashlib
import json
import os
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exe", type=Path)
    parser.add_argument("--dmg", type=Path)
    parser.add_argument("--version", default="999.0.0")
    parser.add_argument("--port", type=int, default=8770)
    parser.add_argument("--bad-checksum", action="store_true")
    args = parser.parse_args()
    if not args.exe and not args.dmg:
        catalogs = Path(__file__).resolve().parents[1] / "i18n"
        language = os.environ.get("MOUSETROKE_BUILD_LANG", "en-US")
        if language not in {path.stem for path in catalogs.glob("*.json")}:
            language = "en-US"
        messages = json.loads((catalogs / f"{language}.json").read_text(encoding="utf-8"))
        parser.error(messages["update.fixture.payload_required"])
    origin = f"http://127.0.0.1:{args.port}"
    release = {
        "tag_name": f"v{args.version}", "draft": False, "prerelease": False,
        "html_url": f"{origin}/releases/latest",
        "assets": [{"name": "SHA256SUMS.txt", "browser_download_url": f"{origin}/SHA256SUMS.txt"}],
    }
    bodies = {}
    sums = []
    for path, name in [(args.exe, "mousetroke.exe"), (args.dmg, "Mousetroke.dmg")]:
        if path is None:
            continue
        payload = path.read_bytes()
        digest = "0" * 64 if args.bad_checksum else hashlib.sha256(payload).hexdigest()
        release["assets"].append({"name": name, "browser_download_url": f"{origin}/{name}"})
        bodies[f"/{name}"] = payload
        sums.append(f"{digest} *{name}\n")
    bodies["/repos/eiaao/mousetroke/releases/latest"] = json.dumps(release).encode()
    bodies["/SHA256SUMS.txt"] = "".join(sums).encode()

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            body = bodies.get(self.path)
            self.send_response(200 if body is not None else 404)
            self.send_header("Content-Length", str(len(body or b"")))
            self.end_headers()
            self.wfile.write(body or b"")

        def log_message(self, *_):
            # 不记录请求路径，避免养成输出下载签名参数的习惯。
            print("update fixture: request completed", flush=True)

    print(f"Update fixture ready on 127.0.0.1:{args.port}, version {args.version}", flush=True)
    try:
        ThreadingHTTPServer(("127.0.0.1", args.port), Handler).serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
