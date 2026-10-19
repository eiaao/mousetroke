// 测试桩：模拟 Tauri 后端命令（仅用于浏览器内交互测试）
// 用法：python -m http.server 8765（目录内放 ui\dist 的内容 + 本文件 + config.json，并在 index.html 里先加载 mock.js），然后 node ui.mjs http://localhost:8765/ <截图目录>
(function () {
  const log = (window.__calls = []);
  function record(raw, grid) {
    if (!raw.length) return [];
    let anchor = raw[0], segs = [];
    for (const [x, y] of raw.slice(1)) {
      const dx = x - anchor[0], dy = y - anchor[1];
      if (Math.hypot(dx, dy) < grid) continue;
      const sx = Math.round(dx / grid), sy = Math.round(dy / grid);
      if (!sx && !sy) continue;
      const l = segs[segs.length - 1];
      if (l && Math.sign(l.x) === Math.sign(sx) && Math.sign(l.y) === Math.sign(sy)) { l.x += sx; l.y += sy; } else segs.push({ x: sx, y: sy });
      anchor = [x, y];
    }
    const t = Math.tan(Math.PI / 8), dirs = [];
    for (const s of segs) {
      const ax = Math.abs(s.x), ay = Math.abs(s.y);
      const d = ay < ax * t ? [Math.sign(s.x), 0] : ax < ay * t ? [0, Math.sign(s.y)] : [Math.sign(s.x), Math.sign(s.y)];
      const last = dirs[dirs.length - 1];
      if (!last || last[0] !== d[0] || last[1] !== d[1]) dirs.push(d);
    }
    if (!dirs.length) return [];
    const pts = [[0, 0]];
    for (const [dx, dy] of dirs) { const p = pts[pts.length - 1]; pts.push([p[0] + dx * 50, p[1] + dy * 50]); }
    const mx = Math.min(...pts.map((p) => p[0])), my = Math.min(...pts.map((p) => p[1]));
    return pts.flatMap(([x, y]) => [x - mx, y - my]);
  }
  // 形状命令的简易替身，仅供浏览器冒烟；产品中的采样与比较均由 Rust 核心完成。
  function shapeSample(raw) {
    const length = raw.slice(1).reduce((n, p, i) => n + Math.hypot(p[0] - raw[i][0], p[1] - raw[i][1]), 0);
    if (raw.length < 2 || length < 1) throw { key: "ui.shape.too_short", params: {} };
    const pts = Array.from({ length: 64 }, (_, i) => raw[Math.round(i * (raw.length - 1) / 63)]);
    const xs = pts.map((p) => p[0]), ys = pts.map((p) => p[1]);
    const span = Math.max(Math.max(...xs) - Math.min(...xs), Math.max(...ys) - Math.min(...ys));
    const cx = xs.reduce((a, b) => a + b, 0) / 64, cy = ys.reduce((a, b) => a + b, 0) / 64;
    return pts.flatMap(([x, y]) => [Math.round((x - cx) * 1000 / span), Math.round((y - cy) * 1000 / span)]);
  }
  function classifyStroke(raw) {
    if (raw.length < 2) return "trace";
    const xs = raw.map((p) => p[0]), ys = raw.map((p) => p[1]);
    const diagonal = Math.hypot(Math.max(...xs) - Math.min(...xs), Math.max(...ys) - Math.min(...ys));
    const simplify = (points) => {
      const a = points[0], b = points.at(-1), dx = b[0] - a[0], dy = b[1] - a[1], length = dx * dx + dy * dy;
      let far = 0, split = 0;
      for (let i = 1; i < points.length - 1; i++) {
        const p = points[i], t = length ? Math.max(0, Math.min(1, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / length)) : 0;
        const distance = Math.hypot(p[0] - a[0] - t * dx, p[1] - a[1] - t * dy);
        if (distance > far) { far = distance; split = i; }
      }
      return far > diagonal * 0.06 ? [...simplify(points.slice(0, split + 1)).slice(0, -1), ...simplify(points.slice(split))] : [a, b];
    };
    const path = simplify(raw);
    return path.length <= 5 && [0.08, 0.10, 0.12].some((grid) => record(raw, Math.max(1, diagonal * grid)).length <= 10) ? "trace" : "shape";
  }
  window.__listeners = {};
  // 测试用：触发 Tauri 事件（拖放等）
  window.__emit = (event, payload) => (window.__listeners[event] ?? []).forEach((h) => h({ event, payload, id: 1 }));
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "main" }, currentWebview: { windowLabel: "main", label: "main" } },
    transformCallback: (cb) => cb,
    async invoke(cmd, args) {
      log.push([cmd, args]);
      switch (cmd) {
        case "init_state": {
          // ?lang=en-US 时用英文版测试配置（用户数据也是英文，才能断言整页无中文）；
          // resolvedLang / systemLangs 模拟后端按系统语言解析的结果
          const q = new URLSearchParams(location.search);
          const lang = q.get("lang") ?? "zh-CN";
          const config = await (await fetch(q.get("cfg") ?? (lang === "zh-CN" ? "config.json" : "config-en.json"))).json();
          return { configPath: "C:\\Users\\me\\AppData\\Roaming\\mousetroke\\config.json", config, configExists: true, loadError: null,
            engine: { running: true, autostart: false, exeFound: true },
            systemLangs: [lang === "qps-ploc" ? "en-US" : lang], resolvedLang: lang };
        }
        case "take_about_request": { const about = window.__aboutRequest ?? false; window.__aboutRequest = false; return about; }
        case "check_update": {
          const outcome = window.__updateOutcome ?? new URLSearchParams(location.search).get("update") ?? "current";
          await new Promise((r) => setTimeout(r, 100));
          if (outcome === "error") throw { key: "update.error.network", params: {} };
          const zh = (new URLSearchParams(location.search).get("lang") ?? "zh-CN").startsWith("zh");
          const notes = zh ? ["形状手势：画 G、C、O、S、6 这类曲线来执行动作。同一个形状录 2–5 次，Mousetroke 会记住你的画法。", "录制时自动区分直线和曲线：直的录成轨迹，弯的录成形状；判断错了点一下就能切换。", "识别阈值用真实手绘样本校准：画得随意、有点抖的曲线也能认出来，直线手势不会被误认成形状。", "乱画不再无限持续：方向变化超过 100 次会自动取消并给个提示，不执行任何动作。", "检查更新：默认每天自动检查一次（可在偏好设置里关闭），也可以在“关于”里手动检查。一键更新：自动下载、校验 SHA-256、替换并重启。", "“关于”页加入主页、源代码和反馈入口。"] : ["Shape gestures: draw a curve such as G, C, O, S or 6 to run an action. Record the same shape 2–5 times and Mousetroke learns how you draw it.", "The recorder tells lines from curves by itself: a straight stroke becomes a line gesture, a curve becomes a shape. One click switches it if it guessed wrong.", "Recognition is tuned on real hand-drawn strokes: loose, wobbly curves are accepted, while straight gestures are never mistaken for shapes.", "Endless scribbling no longer keeps the gesture going: after 100 direction changes it is cancelled with a short message, and nothing runs.", "Check for updates: automatically once a day (can be turned off in Preferences) or from About. Updating takes one click: the new version is downloaded, its checksum verified, and Mousetroke restarts itself.", "About now links to the website, the source code and feedback."];
          return { current: "0.1.0", latest: outcome === "newer" ? "0.2.0" : "0.1.0", newer: outcome === "newer", notes: outcome === "newer" ? notes : [], notes_url: "https://github.com/eiaao/mousetroke/releases/latest" };
        }
        case "install_update": {
          const outcome = window.__installOutcome ?? new URLSearchParams(location.search).get("install") ?? "success";
          if (outcome === "pending") return new Promise(() => {});
          if (outcome !== "success") throw { key: `update.error.${outcome}`, params: {} };
          return null;
        }
        case "get_engine_status": return { running: true, autostart: false, exeFound: true };
        case "validate_keys": return /\{[^}]*$/.test(args.keys) ? { key: "keyseq.error.near", params: { pos: String(args.keys.length), reason: "keyseq.error.missing_brace" } } : null;
        case "record_trace": return record(args.points, args.grid);
        case "classify_stroke": return classifyStroke(args.points);
        case "shape_sample": return shapeSample(args.points);
        case "shape_compare": return window.__shapeCompareScore ?? (args.a.length === 128 && args.b.length === 128 ? Math.max(0, 1 - Math.sqrt(args.a.reduce((sum, x, i) => sum + (x - args.b[i]) ** 2, 0) / 128) / 500) : 0);
        case "shape_limits": {
          const raw = args.sample?.length === 128 ? Array.from({ length: 64 }, (_, i) => [args.sample[i * 2], args.sample[i * 2 + 1]]) : [];
          return { accept: classifyStroke(raw) === "shape" ? 0.687 : 0.801, margin: 0.04 };
        }
        case "app_icons": return Object.fromEntries(args.exes.map((e) => [e, /chrome/i.test(e) ? "data:image/svg+xml," + encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><circle cx="16" cy="16" r="15" fill="#db4437"/><circle cx="16" cy="16" r="7" fill="#4285f4" stroke="#fff" stroke-width="2"/></svg>') : null]));
        case "running_apps": return [{ exe: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe", title: "新标签页 - Google Chrome" }, { exe: "C:\\Windows\\explorer.exe", title: "下载" }, { exe: "C:\\Users\\me\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe", title: "main.rs - mousetroke" }];
        case "save_config": window.__saved = args.config; return null;
        case "open_path":
          if (["log", "config_dir", "homepage", "github", "issues", "releases"].includes(args.what)) return null;
          throw { key: "error.unknown_location", params: { what: args.what } };
        case "plugin:event|listen": (window.__listeners[args.event] ??= []).push(args.handler); return window.__listeners[args.event].length;
        case "resolve_app":
          if (/\.exe$/i.test(args.path)) return args.path;
          if (/\.lnk$/i.test(args.path)) return "C:\\Program Files\\Notepad++\\notepad++.exe";
          throw { key: "error.not_a_program", params: { file: args.path.split("\\").pop() } };
        case "preset_apps": return [
          { exe: "mstsc.exe", name_key: "preset.app.mstsc", name: "远程桌面连接", category: "远程与虚拟机", category_key: "preset.category.remote", path: "C:\\Windows\\System32\\mstsc.exe" },
          { exe: "ToDesk.exe", name_key: "preset.app.todesk", name: "ToDesk", category: "远程与虚拟机", category_key: "preset.category.remote", path: null },
          { exe: "blender.exe", name_key: "preset.app.blender", name: "Blender", category: "3D 与设计", category_key: "preset.category.design", path: "C:\\Program Files\\Blender Foundation\\Blender 4.2\\blender.exe" },
          { exe: "acad.exe", name_key: "preset.app.acad", name: "AutoCAD", category: "3D 与设计", category_key: "preset.category.design", path: null },
        ];
        case "app_info": return { version: "0.1.0", exePath: "D:\\Mousetroke\\mousetroke.exe", configPath: "C:\\Users\\me\\AppData\\Roaming\\mousetroke\\config.json", logPath: "C:\\Users\\me\\AppData\\Roaming\\mousetroke\\mousetroke.log" };
        case "pick_window": return { exe: "C:\\Program Files\\Mozilla Firefox\\firefox.exe", title: "Firefox" };
        default: return null;
      }
    },
  };
})();
