#!/usr/bin/env python3
"""调试版更新回归服务：只监听回环地址，不写入发布文件，不执行重启。"""

import argparse
import hashlib
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exe", type=Path, required=True)
    parser.add_argument("--version", default="999.0.0")
    parser.add_argument("--port", type=int, default=8770)
    parser.add_argument("--bad-checksum", action="store_true")
    args = parser.parse_args()
    payload = args.exe.read_bytes()
    digest = "0" * 64 if args.bad_checksum else hashlib.sha256(payload).hexdigest()
    origin = f"http://127.0.0.1:{args.port}"
    release = {
        "tag_name": f"v{args.version}", "draft": False, "prerelease": False,
        "html_url": f"{origin}/releases/latest",
        "assets": [
            {"name": "mousetroke.exe", "browser_download_url": f"{origin}/mousetroke.exe"},
            {"name": "SHA256SUMS.txt", "browser_download_url": f"{origin}/SHA256SUMS.txt"},
        ],
    }
    bodies = {
        "/repos/eiaao/mousetroke/releases/latest": json.dumps(release).encode(),
        "/mousetroke.exe": payload,
        "/SHA256SUMS.txt": f"{digest} *mousetroke.exe\n".encode(),
    }

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
