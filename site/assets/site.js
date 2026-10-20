// Mousetroke 产品页交互。所有文案都在 HTML 里（#demo-text 与各元素），脚本只负责行为，
// 这样中英两页共用一份脚本，搜索引擎和不跑脚本的读者也能看到完整内容。
(() => {
  "use strict";
  const $ = (s, r = document) => r.querySelector(s);
  const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));
  const T = JSON.parse($("#demo-text")?.textContent || "{}");
  const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const isWin = /Win/.test(navigator.platform || navigator.userAgent);
  const css = (name) => getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
  const docY = (el) => el.getBoundingClientRect().top + scrollY;
  const store = {
    get(k) { try { return localStorage.getItem(k); } catch { return null; } },
    set(k, v) { try { localStorage.setItem(k, v); } catch { /* 隐私模式等 */ } },
  };
  // 平台文案与下载共用 os-win/os-mac；默认显示 Windows，Mac 在 HTML 中 hidden。
  // iPad 也报 MacIntel，用触屏点数排除；不跑脚本时主按钮是 Windows 版，菜单列出全部。
  const isMac = (navigator.userAgentData?.platform === "macOS" || /Mac/.test(navigator.platform || "")) && navigator.maxTouchPoints < 2;
  const os = isMac ? "os-mac" : "os-win";
  $$(".os-win, .os-mac").forEach((el) => { el.hidden = !el.classList.contains(os); });
  // ▾ 菜单反向显示其他系统；无脚本时两种下载都可从菜单访问。
  $$(".split-menu a").forEach((a) => { a.hidden = a.classList.contains(os); });
  // 鼠标设备悬停即展开，移出整个组合按钮后稍等再收起；触屏仍是点按展开
  if (matchMedia("(hover: hover)").matches) {
    $$(".split").forEach((split) => {
      const d = $(".split-more", split);
      let timer = 0;
      d.addEventListener("pointerenter", () => { clearTimeout(timer); d.open = true; });
      split.addEventListener("pointerenter", () => clearTimeout(timer));
      split.addEventListener("pointerleave", () => { timer = setTimeout(() => { d.open = false; }, 200); });
    });
  }
  $$(".split-more").forEach((d) => d.addEventListener("toggle", () => {
    if (!d.open) return;
    const close = (e) => {
      if (e.type === "keydown" ? e.key === "Escape" : !d.contains(e.target)) {
        d.open = false;
        removeEventListener("pointerdown", close);
        removeEventListener("keydown", close);
      }
    };
    addEventListener("pointerdown", close);
    addEventListener("keydown", close);
  }));

  // 在语言菜单里手动选过语言后记住选择，首页不再按浏览器语言自动跳转（见 _worker.js）
  $$(".langdrop a[hreflang]").forEach((a) => a.addEventListener("click", () => {
    document.cookie = `mt_lang=${encodeURIComponent(a.hreflang)}; path=/; max-age=31536000; SameSite=Lax`;
  }));
  // 锚点滚动由脚本接管（自定义缓动），关掉 CSS 的 smooth 避免逐帧叠加
  document.documentElement.style.scrollBehavior = "auto";

  // ---------- 方向编码（与引擎同一套：P 为 0/50/100 的折线坐标，y 向下） ----------
  const ARROWS = ["→", "↘", "↓", "↙", "←", "↖", "↑", "↗"];
  /** 画的方向：正方向各占 60°，斜方向只占 30°，减少手抖出来的斜线。 */
  function dirOf(dx, dy) {
    const a = ((Math.atan2(dy, dx) * 180) / Math.PI + 360) % 360;
    let d = Math.round(a / 45) % 8;
    if (d % 2 === 1 && Math.abs(a - d * 45) > 15) d = (a < d * 45 ? d - 1 : d + 1) % 8;
    return d;
  }
  function patternDirs(p) {
    const n = p.split(",").map(Number);
    const out = [];
    for (let i = 2; i + 1 < n.length; i += 2) {
      const d = dirOf(n[i] - n[i - 2], n[i + 1] - n[i - 1]);
      if (out[out.length - 1] !== d) out.push(d);
    }
    return out;
  }
  const keyOf = (dirs) => dirs.map((d) => ARROWS[d]).join("");
  /** 去掉夹在两个相邻正方向之间的斜线（↓↘→ 视为 ↓→）。 */
  function cleaned(dirs) {
    const near = (a, b) => (a - b + 8) % 8 === 1 || (b - a + 8) % 8 === 1;
    const kept = dirs.filter((d, i) => !(d % 2 === 1 && i > 0 && i < dirs.length - 1 && near(dirs[i - 1], d) && near(dirs[i + 1], d)));
    return kept.filter((d, i) => kept[i - 1] !== d);
  }
  const lookup = (map, dirs) => map.get(keyOf(dirs)) || map.get(keyOf(cleaned(dirs)));

  // ---------- 手势表里的小图 ----------
  function glyphSVG(p) {
    const n = p.split(",").map(Number);
    let raw = [];
    for (let i = 0; i + 1 < n.length; i += 2) raw.push([n[i], n[i + 1]]);
    // 原路折返（←→、↑↓）会画在同一条线上：折返处向侧边错开一点，画成回形针
    const pts0 = [raw[0]];
    let off = [0, 0];
    for (let i = 1; i < raw.length; i++) {
      const d = [raw[i][0] - raw[i - 1][0], raw[i][1] - raw[i - 1][1]];
      const pd = i > 1 ? [raw[i - 1][0] - raw[i - 2][0], raw[i - 1][1] - raw[i - 2][1]] : null;
      if (pd && d[0] * pd[0] + d[1] * pd[1] < 0 && d[0] * pd[1] - d[1] * pd[0] === 0) {
        const len = Math.hypot(pd[0], pd[1]);
        off = [off[0] + (-pd[1] / len) * 16, off[1] + (pd[0] / len) * 16];
        pts0.push([raw[i - 1][0] + off[0], raw[i - 1][1] + off[1]]);
      }
      pts0.push([raw[i][0] + off[0], raw[i][1] + off[1]]);
    }
    const xs = pts0.map((q) => q[0]), ys = pts0.map((q) => q[1]);
    const minX = Math.min(...xs), minY = Math.min(...ys);
    const span = Math.max(Math.max(...xs) - minX, Math.max(...ys) - minY, 1);
    const s = 24 / span;
    const ox = 3 + (24 - (Math.max(...xs) - minX) * s) / 2, oy = 3 + (24 - (Math.max(...ys) - minY) * s) / 2;
    const pts = pts0.map((q) => [ox + (q[0] - minX) * s, oy + (q[1] - minY) * s]);
    const d = pts.map((q, i) => (i ? "L" : "M") + q[0].toFixed(1) + " " + q[1].toFixed(1)).join(" ");
    // 末端箭头：终点 → 左翼 → 终点 → 右翼，接在主线后面一笔画完。
    // 不能用 M 另起子路径：pathLength 归一后，描线动画在子路径之间是断开的。
    const [a, b] = [pts[pts.length - 2], pts[pts.length - 1]];
    const ang = Math.atan2(b[1] - a[1], b[0] - a[0]);
    const h = (k) => [b[0] - 6 * Math.cos(ang + k), b[1] - 6 * Math.sin(ang + k)];
    const [l, r] = [h(0.6), h(-0.6)];
    return `<svg viewBox="0 0 30 30" aria-hidden="true"><circle cx="${pts[0][0]}" cy="${pts[0][1]}" r="2.6"/><path pathLength="1" d="${d} L${l[0].toFixed(1)} ${l[1].toFixed(1)} L${b[0].toFixed(1)} ${b[1].toFixed(1)} L${r[0].toFixed(1)} ${r[1].toFixed(1)}"/></svg>`;
  }
  $$(".sheet li[data-p]").forEach((li) => { $(".glyph", li).innerHTML = glyphSVG(li.dataset.p); });

  // 首屏演示能识别的手势（只取右键、无组合键的那些）：方向串 → { act, name }
  const GESTURES = new Map();
  $$(".sheet li[data-act]").forEach((li) => {
    const k = keyOf(patternDirs(li.dataset.p));
    if (!GESTURES.has(k)) GESTURES.set(k, { act: li.dataset.act, name: $(".nm", li).firstChild.textContent.trim() });
  });

  // =====================================================================
  // 轨迹：捕获 → 识别 → 画线 → 胶囊提示。首屏画板与全页手势共用。
  // =====================================================================
  class Trail {
    constructor(canvas, pill, resolve, { fixed = false } = {}) {
      this.canvas = canvas; this.ctx = canvas.getContext("2d");
      this.pill = pill; this.arrows = $(".arrows", pill); this.name = $(".name", pill);
      this.resolve = resolve; this.fixed = fixed;
      this.pts = []; this.dirs = []; this.capturing = false; this.fade = 0; this.raf = 0;
      this.resize();
      if (fixed) addEventListener("resize", () => this.resize());
      else new ResizeObserver(() => this.resize()).observe(canvas.parentElement);
    }
    resize() {
      const r = this.fixed ? { width: innerWidth, height: innerHeight } : this.canvas.parentElement.getBoundingClientRect();
      const dpr = Math.min(devicePixelRatio || 1, 2);
      this.W = r.width; this.H = r.height;
      this.canvas.width = Math.round(this.W * dpr); this.canvas.height = Math.round(this.H * dpr);
      this.ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    }
    begin(x, y) {
      cancelAnimationFrame(this.raf);
      this.capturing = true; this.moved = false; this.fade = 0; this.matched = null;
      this.pts = [[x, y]]; this.dirs = []; this.anchor = [x, y]; this.start = [x, y];
    }
    add(x, y) {
      if (!this.capturing) return;
      const last = this.pts[this.pts.length - 1];
      if (Math.hypot(x - last[0], y - last[1]) < 1.5) return;
      this.pts.push([x, y]);
      if (!this.moved && Math.hypot(x - this.start[0], y - this.start[1]) > 6) this.moved = true;
      if (Math.hypot(x - this.anchor[0], y - this.anchor[1]) >= 26) {
        const d = dirOf(x - this.anchor[0], y - this.anchor[1]);
        if (this.dirs[this.dirs.length - 1] !== d) this.dirs.push(d);
        this.anchor = [x, y];
        this.matched = this.resolve(this.dirs);
        this.showPill();
      }
      this.draw();
    }
    showPill() {
      if (!this.dirs.length) return this.pill.classList.remove("on");
      this.arrows.textContent = keyOf(cleaned(this.dirs));
      this.name.textContent = this.matched ? this.matched.name : T.miss;
      this.pill.classList.toggle("miss", !this.matched);
      this.pill.classList.add("on");
    }
    /** 结束捕获，返回识别结果（没画或没认出来是 null）。 */
    end() {
      if (!this.capturing) return null;
      this.capturing = false;
      const m = this.moved && this.dirs.length ? this.matched : null;
      this.draw();
      setTimeout(() => this.pill.classList.remove("on"), m ? 650 : 400);
      const t0 = performance.now();
      const step = (t) => {
        this.fade = Math.min(1, (t - t0) / (reduce ? 1 : 360));
        this.draw();
        if (this.fade < 1) this.raf = requestAnimationFrame(step);
        else { this.pts = []; this.ctx.clearRect(0, 0, this.W, this.H); }
      };
      this.raf = requestAnimationFrame(step);
      return m;
    }
    draw() {
      const { ctx, pts } = this;
      ctx.clearRect(0, 0, this.W, this.H);
      if (pts.length < 2) return;
      ctx.globalAlpha = 1 - this.fade;
      ctx.strokeStyle = this.matched && !this.capturing ? css("--ok") : css("--trail");
      ctx.lineWidth = 5; ctx.lineCap = "round"; ctx.lineJoin = "round";
      ctx.shadowColor = ctx.strokeStyle; ctx.shadowBlur = 8;
      ctx.beginPath();
      ctx.moveTo(pts[0][0], pts[0][1]);
      for (let i = 1; i < pts.length - 1; i++) {
        ctx.quadraticCurveTo(pts[i][0], pts[i][1], (pts[i][0] + pts[i + 1][0]) / 2, (pts[i][1] + pts[i + 1][1]) / 2);
      }
      const l = pts[pts.length - 1];
      ctx.lineTo(l[0], l[1]);
      ctx.stroke();
      ctx.shadowBlur = 0; ctx.globalAlpha = 1;
    }
  }

  // =====================================================================
  // 平滑滚动 + 章节
  // =====================================================================
  let scrollRaf = 0;
  const stopScroll = () => cancelAnimationFrame(scrollRaf);
  ["wheel", "touchstart", "keydown"].forEach((ev) => addEventListener(ev, stopScroll, { passive: true }));
  function scrollToY(y, ms = 950) {
    stopScroll();
    y = clamp(y, 0, document.documentElement.scrollHeight - innerHeight);
    const y0 = scrollY, dy = y - y0;
    if (reduce || Math.abs(dy) < 2) return scrollTo(0, y);
    ms = Math.min(1400, ms + Math.abs(dy) * 0.05);
    const t0 = performance.now();
    const ease = (k) => (k < 0.5 ? 8 * k ** 4 : 1 - (-2 * k + 2) ** 4 / 2);
    const step = (t) => {
      const k = Math.min(1, (t - t0) / ms);
      scrollTo(0, y0 + dy * ease(k));
      if (k < 1) scrollRaf = requestAnimationFrame(step);
    };
    scrollRaf = requestAnimationFrame(step);
  }
  const hero = $(".hero"), get = $(".get");
  const SECTIONS = [hero, ...$$("section.sec"), get].filter(Boolean);
  /** 每个章节"切到"时停的位置：标题露在顶栏下方一点。 */
  const stopAt = (sec) => (sec === hero ? 0 : sec === get ? docY(sec) : docY($("h2", sec)) - 110);
  const anchorTarget = (el) => (SECTIONS.includes(el) ? stopAt(el) : docY(el) - 72);
  document.addEventListener("click", (e) => {
    const a = e.target.closest('a[href^="#"]');
    if (!a || a.getAttribute("href").length < 2) return;
    const el = document.getElementById(a.getAttribute("href").slice(1));
    if (!el) return;
    e.preventDefault();
    scrollToY(anchorTarget(el));
    history.replaceState(null, "", a.getAttribute("href"));
    if (el.id === "main") el.focus?.();
  });
  function goSection(dir) {
    const now = scrollY, stops = SECTIONS.map(stopAt);
    if (dir === "top") return scrollToY(0);
    if (dir === "bottom") return scrollToY(document.documentElement.scrollHeight);
    if (dir > 0) { const y = stops.find((s) => s > now + 8); scrollToY(y ?? document.documentElement.scrollHeight); }
    else { const ys = stops.filter((s) => s < now - 8); scrollToY(ys.length ? ys[ys.length - 1] : 0); }
  }

  // 顶栏阴影 + 进度条 + 右侧章节导轨
  const top = $(".top");
  const bar = document.createElement("div");
  bar.className = "progress";
  document.body.appendChild(bar);
  const dotsNav = document.createElement("nav");
  dotsNav.setAttribute("aria-label", T.dots || "");
  const dots = document.createElement("ul");
  dots.className = "dots";
  SECTIONS.forEach((sec) => {
    const li = document.createElement("li");
    const a = document.createElement("a");
    a.href = "#" + (sec.id || "main");
    const label = sec === hero ? T.heroDot : $("h2", sec).textContent.trim();
    a.innerHTML = `<span></span><i></i>`;
    a.firstChild.textContent = label;
    a.setAttribute("aria-label", label);
    if (sec === hero) a.addEventListener("click", (e) => { e.preventDefault(); e.stopPropagation(); scrollToY(0); });
    li.appendChild(a); dots.appendChild(li);
  });
  dotsNav.appendChild(dots);
  document.body.appendChild(dotsNav);
  const dotLinks = $$("a", dots);

  // =====================================================================
  // 贯穿全页的轨迹线：滚动时向下"画"，经过章节标题时分叉指向它
  // =====================================================================
  const NS = "http://www.w3.org/2000/svg";
  const spine = document.createElementNS(NS, "svg");
  spine.classList.add("spine");
  spine.setAttribute("aria-hidden", "true");
  document.body.prepend(spine);
  let geo = null;
  function layoutSpine() {
    const wrap = $("section.sec .wrap");
    if (!wrap || innerWidth < 1240) { geo = null; spine.innerHTML = ""; return; }
    const headX = wrap.getBoundingClientRect().left + scrollX + parseFloat(getComputedStyle(wrap).paddingLeft);
    const g = Math.round(headX - 58);
    if (g < 24) { geo = null; spine.innerHTML = ""; return; }
    const y0 = docY(hero) + hero.offsetHeight - 30;
    const y1 = get ? docY(get) - 40 : document.documentElement.scrollHeight - 200;
    const H = document.documentElement.scrollHeight;
    spine.setAttribute("height", H);
    spine.setAttribute("viewBox", `0 0 ${innerWidth} ${H}`);
    spine.style.height = H + "px";
    const branches = $$("section.sec h2").map((h) => {
      const fs = parseFloat(getComputedStyle(h).fontSize);
      const y = docY(h) + fs * 0.55;
      const x2 = headX - 14;
      return { y, d: `M${g} ${y - 34} C${g} ${y - 8} ${g + 6} ${y} ${g + 28} ${y} L${x2} ${y} L${x2 - 8} ${y - 7} L${x2} ${y} L${x2 - 8} ${y + 7}` };
    });
    spine.innerHTML =
      `<path class="rail" d="M${g} ${y0} L${g} ${y1}"/><path class="ink" d=""/>` +
      branches.map((b) => `<path class="br" pathLength="1" d="${b.d}"/><circle class="node" cx="${g}" cy="${b.y - 34}" r="5"/>`).join("") +
      `<circle class="tip" r="6" cx="${g}" cy="${y0}"/>`;
    geo = { g, y0, y1, branches, ink: $(".ink", spine), tip: $(".tip", spine), brs: $$(".br", spine), nodes: $$(".node", spine) };
    onScroll();
  }

  function onScroll() {
    const H = document.documentElement.scrollHeight - innerHeight;
    top?.classList.toggle("scrolled", scrollY > 8);
    bar.style.setProperty("--p", H > 0 ? (scrollY / H).toFixed(4) : 0);
    // 当前章节
    const probe = scrollY + innerHeight * 0.4;
    let cur = 0;
    SECTIONS.forEach((s, i) => { if (docY(s) <= probe) cur = i; });
    dotLinks.forEach((a, i) => a.classList.toggle("cur", i === cur));
    dots.classList.toggle("on-dark", SECTIONS[cur] === get);
    // 轨迹线
    if (geo) {
      const tipY = clamp(scrollY + innerHeight * 0.62, geo.y0, geo.y1);
      geo.ink.setAttribute("d", `M${geo.g} ${geo.y0} L${geo.g} ${tipY}`);
      geo.tip.setAttribute("cy", tipY);
      geo.tip.style.opacity = tipY > geo.y0 + 2 ? 1 : 0;
      geo.branches.forEach((b, i) => {
        const on = tipY >= b.y - 34;
        geo.brs[i].classList.toggle("on", on);
        geo.nodes[i].classList.toggle("on", on);
      });
    }
    // 全页手势提示：离开首屏后出现
    if (hint && !hintDone) hint.classList.toggle("on", scrollY > hero.offsetHeight * 0.6 && scrollY < H - 200);
  }
  let scrollQueued = false;
  addEventListener("scroll", () => {
    if (scrollQueued) return;
    scrollQueued = true;
    requestAnimationFrame(() => { scrollQueued = false; onScroll(); });
  }, { passive: true });

  // =====================================================================
  // 出场动画
  // =====================================================================
  // 首屏标题逐词（中文逐字）升起
  const h1 = $(".hero h1");
  if (h1 && !reduce) {
    h1.setAttribute("aria-label", h1.textContent.replace(/\s+/g, " ").trim());
    const CJK = "　-〿＀-￯一-鿿";
    const re = new RegExp(`[${CJK}]|[^\\s${CJK}]+`, "g"), latin = new RegExp(`^[^${CJK}]`);
    let i = 0;
    const out = [];
    h1.childNodes.forEach((n) => {
      if (n.nodeType === 3) {
        (n.textContent.match(re) || []).forEach((tok) => out.push(`<span class="wm" aria-hidden="true"><span class="w" style="--i:${i++}">${tok}</span></span>${latin.test(tok) ? " " : ""}`));
      } else out.push(n.outerHTML);
    });
    h1.innerHTML = out.join("").replace(/\s+$/, "");
  }
  // 章节标题：遮罩上滑
  $$("section.sec h2, .get h2").forEach((h) => { h.innerHTML = `<span class="ln"><span>${h.innerHTML}</span></span>`; });
  const mark = (els, cls = "", stagger = true) => els.forEach((el, i) => {
    el.classList.add("rv"); if (cls) el.classList.add(cls);
    if (stagger) el.style.setProperty("--i", i);
  });
  mark($$(".sec-head p"));
  mark($$(".note"));
  $$(".sheet > div").forEach((col, c) => {
    mark([$("h3", col)]);
    $$("li", col).forEach((li, i) => { li.classList.add("rv"); li.style.setProperty("--i", i + c * 2); });
  });
  mark($$(".rules li"), "from-r");
  mark($$(".monitor"), "zoom");
  mark($$(".acts > div"));
  mark($$("#apps"), "from-l", false);
  mark($$("#apps .row"));
  mark($$(".points li"), "from-r");
  mark($$(".figs > div"));
  mark($$("#proc"), "zoom", false);
  mark($$(".faq details"));
  mark($$(".get .wrap > div"));
  const revealIO = new IntersectionObserver((es) => es.forEach((e) => {
    if (!e.isIntersecting) return;
    e.target.classList.add("in");
    revealIO.unobserve(e.target);
    e.target.dispatchEvent(new CustomEvent("revealed"));
  }), { threshold: 0.12, rootMargin: "0px 0px -6% 0px" });
  $$(".rv, section.sec h2, .get, .get h2").forEach((el) => revealIO.observe(el));

  // 数字从 0 数上来
  $$(".figs dd").forEach((dd) => {
    const t = dd.firstChild;
    if (!t || t.nodeType !== 3 || reduce) return;
    const to = parseFloat(t.textContent), dec = (t.textContent.split(".")[1] || "").length;
    t.textContent = (0).toFixed(dec);
    dd.parentElement.addEventListener("revealed", () => {
      const t0 = performance.now();
      const step = (now) => {
        const k = Math.min(1, (now - t0) / 1400);
        t.textContent = (to * (1 - (1 - k) ** 4)).toFixed(dec);
        if (k < 1) requestAnimationFrame(step);
      };
      requestAnimationFrame(step);
    }, { once: true });
  });

  // 下载区的大笔画
  if (get) {
    const big = document.createElementNS(NS, "svg");
    big.classList.add("big");
    big.setAttribute("viewBox", "0 0 1000 600");
    big.setAttribute("aria-hidden", "true");
    big.innerHTML = '<circle cx="300" cy="70" r="22"/><path pathLength="1" d="M300 70 C 298 200, 300 330, 312 400 C 322 452, 360 470, 450 468 L 900 462 L 830 400 L 900 462 L 830 524"/>';
    get.prepend(big);
  }

  // =====================================================================
  // 首屏画板
  // =====================================================================
  const stage = $("#stage");
  if (stage) initStage(stage);

  function initStage(stage) {
    const toastEl = $(".toast", stage);
    const browser = $("#browser");
    const trail = new Trail($("canvas", stage), $(".gpill", stage), (dirs) => lookup(GESTURES, dirs));

    // --- 模拟浏览器状态 ---
    let uid = 0;
    const mkTab = (hist, i = hist.length - 1) => ({ id: ++uid, hist, i });
    const fresh = () => [mkTab(T.pages.map((p) => p.slice()), 1), mkTab([T.tabs[0]]), mkTab([T.tabs[1]])];
    let tabs = fresh(), cur = 0, topmost = false;
    const tabsEl = $(".tabs", browser), urlEl = $(".url", browser), viewEl = $(".view", browser);
    const hue = (s) => [...s].reduce((h, c) => (h * 31 + c.charCodeAt(0)) % 360, 7);
    function pageEl(page) {
      const el = document.createElement("div");
      el.className = "page";
      const h = hue(page[1]);
      const bars = Array.from({ length: 22 }, (_, i) => `<div class="bar" style="width:${55 + ((h + i * 37) % 45)}%"></div>${i % 6 === 2 ? `<div class="blk" style="background:hsl(${(h + i * 20) % 360} 55% 62% / .35)"></div>` : ""}`).join("");
      el.innerHTML = `<div class="scroller"><h3></h3>${bars}</div>`;
      $("h3", el).textContent = page[0];
      return el;
    }
    function renderTabs() {
      const have = new Map($$(".tab:not(.out)", tabsEl).map((e) => [+e.dataset.id, e]));
      tabs.forEach((t, i) => {
        let e = have.get(t.id);
        if (!e) { e = document.createElement("div"); e.className = "tab"; e.dataset.id = t.id; tabsEl.appendChild(e); }
        have.delete(t.id);
        e.textContent = t.hist[t.i][0];
        e.classList.toggle("cur", i === cur);
      });
      have.forEach((e) => { e.classList.add("out"); setTimeout(() => e.remove(), 260); });
    }
    function renderPage(from) {
      const t = tabs[cur], page = t.hist[t.i];
      urlEl.textContent = page[1];
      const old = $$(".page", viewEl);
      const el = pageEl(page);
      if (from && !reduce) {
        el.classList.add(from === "l" ? "enter-l" : "enter-r");
        viewEl.appendChild(el);
        el.getBoundingClientRect();
        el.classList.remove("enter-l", "enter-r");
        old.forEach((o) => { o.classList.add(from === "l" ? "leave-r" : "leave-l"); setTimeout(() => o.remove(), 420); });
      } else {
        old.forEach((o) => o.remove());
        viewEl.appendChild(el);
      }
    }
    const render = (from) => { renderTabs(); renderPage(from); };
    render();

    let toastTimer;
    function toast(msg) {
      toastEl.textContent = msg;
      toastEl.classList.add("on");
      clearTimeout(toastTimer);
      toastTimer = setTimeout(() => toastEl.classList.remove("on"), 1400);
    }
    const flash = (n) => { const i = $(`.nb i[data-n="${n}"]`, browser); i.classList.add("flash"); setTimeout(() => i.classList.remove("flash"), 500); };
    const scroller = () => $(".page:last-child .scroller", viewEl);
    const ACTIONS = {
      back() { const t = tabs[cur]; flash("back"); if (t.i > 0) { t.i--; render("l"); } else toast(T.noBack); },
      forward() { const t = tabs[cur]; flash("fwd"); if (t.i < t.hist.length - 1) { t.i++; render("r"); } else toast(T.noFwd); },
      newtab() { tabs.push(mkTab([T.newTab])); cur = tabs.length - 1; render("r"); },
      closetab() {
        if (tabs.length === 1) return ACTIONS.closewin();
        tabs.splice(cur, 1); cur = Math.min(cur, tabs.length - 1); render();
      },
      refresh() { flash("reload"); urlEl.classList.remove("loading"); urlEl.getBoundingClientRect(); urlEl.classList.add("loading"); renderPage(); },
      prevtab() { cur = (cur - 1 + tabs.length) % tabs.length; render(); },
      nexttab() { cur = (cur + 1) % tabs.length; render(); },
      top() { const s = scroller(); if (s) s.style.transform = ""; },
      bottom() { const s = scroller(); if (s) s.style.transform = `translateY(${-(s.scrollHeight - viewEl.clientHeight + 30)}px)`; },
      minimize() {
        browser.classList.add("min");
        const app = $(".taskbar .app", stage);
        setTimeout(() => app.classList.add("bounce"), 350);
        setTimeout(() => { app.classList.remove("bounce"); browser.classList.remove("min"); }, 1500);
      },
      maximize() { browser.classList.toggle("max"); },
      closewin() {
        browser.classList.add("closed");
        toast(T.reopen);
        setTimeout(() => { tabs = fresh(); cur = 0; tabsEl.innerHTML = ""; render(); browser.classList.remove("closed", "max"); }, 1300);
      },
      topmost() { topmost = !topmost; toast(topmost ? T.topmost : T.untop); },
      copy: () => toast(T.copied), paste: () => toast(T.pasted), search: () => toast(T.searched), enter: () => toast(T.entered),
      backspace: () => toast(T.backspace), delete: () => toast(T.deleted), esc: () => toast(T.esc), snip: () => toast(T.snip),
    };

    let lastGestureAt = 0, ghost = false;
    const chips = $$(".try button");
    function finish() {
      const moved = trail.moved;
      const m = trail.end();
      if (moved) lastGestureAt = performance.now();
      if (m) {
        ACTIONS[m.act]?.();
        const k = keyOf(cleaned(trail.dirs));
        chips.forEach((c) => c.classList.toggle("hit", keyOf(patternDirs(c.dataset.play)) === k));
      }
    }
    const local = (e) => { const r = stage.getBoundingClientRect(); return [e.clientX - r.left, e.clientY - r.top]; };
    stage.addEventListener("pointerdown", (e) => {
      if (ghost || (e.pointerType === "mouse" && e.button !== 0 && e.button !== 2)) return;
      stage.setPointerCapture(e.pointerId);
      if (e.button === 0) e.preventDefault();
      stage.classList.add("used");
      chips.forEach((c) => c.classList.remove("hit"));
      trail.begin(...local(e));
    });
    stage.addEventListener("pointermove", (e) => {
      if (ghost) return;
      if (trail.capturing) return trail.add(...local(e));
      // 没在画的时候，窗口随鼠标轻微反向漂移，做出一点纵深
      if (reduce || e.pointerType !== "mouse") return;
      const [x, y] = local(e);
      stage.style.setProperty("--px", ((0.5 - x / trail.W) * 14).toFixed(1) + "px");
      stage.style.setProperty("--py", ((0.5 - y / trail.H) * 10).toFixed(1) + "px");
    });
    stage.addEventListener("pointerleave", () => { stage.style.setProperty("--px", "0px"); stage.style.setProperty("--py", "0px"); });
    stage.addEventListener("pointerup", () => { if (!ghost) finish(); });
    stage.addEventListener("pointercancel", () => { if (!ghost) finish(); });
    // 和真程序一样：只点不画，右键菜单照常弹出（Windows 上菜单在松开时才出现，所以能判断）。
    stage.addEventListener("contextmenu", (e) => {
      if (!isWin || trail.capturing || performance.now() - lastGestureAt < 500) e.preventDefault();
    });

    // --- 按钮回放：沿手势折线合成移动 ---
    function play(p) {
      if (trail.capturing || ghost) return;
      ghost = true;
      chips.forEach((c) => c.classList.remove("hit"));
      const n = p.split(",").map(Number);
      const xs = n.filter((_, i) => i % 2 === 0), ys = n.filter((_, i) => i % 2 === 1);
      const { W, H } = trail;
      const unit = Math.min(W, H) * 0.26 / 50;
      const cx = (Math.min(...xs) + Math.max(...xs)) / 2, cy = (Math.min(...ys) + Math.max(...ys)) / 2;
      const P = xs.map((x, i) => [W / 2 + (x - cx) * unit, H * 0.48 + (ys[i] - cy) * unit]);
      const seg = [];
      for (let i = 1; i < P.length; i++) seg.push(Math.hypot(P[i][0] - P[i - 1][0], P[i][1] - P[i - 1][1]));
      const total = seg.reduce((a, b) => a + b, 0);
      const dur = reduce ? 1 : Math.max(380, total * 2.4);
      trail.begin(P[0][0], P[0][1]);
      const t0 = performance.now();
      const step = (t) => {
        let k = Math.min(1, (t - t0) / dur);
        k = 1 - Math.pow(1 - k, 1.6);
        let dist = k * total, i = 0;
        while (i < seg.length - 1 && dist > seg[i]) { dist -= seg[i]; i++; }
        const f = seg[i] ? Math.min(1, dist / seg[i]) : 1;
        const j = Math.sin(t / 37) * 0.8; // 一点手抖，看起来像人画的
        trail.add(P[i][0] + (P[i + 1][0] - P[i][0]) * f + j, P[i][1] + (P[i + 1][1] - P[i][1]) * f - j);
        if (k < 1) requestAnimationFrame(step);
        else { trail.add(P[P.length - 1][0] + 0.01, P[P.length - 1][1]); finish(); ghost = false; }
      };
      requestAnimationFrame(step);
    }
    chips.forEach((c) => c.addEventListener("click", () => play(c.dataset.play)));

    // --- 落地动画结束后自己画一次 ←，演示怎么用 ---
    if (!reduce) {
      let shown = false;
      const io = new IntersectionObserver((es) => {
        if (shown || !es[0].isIntersecting) return;
        shown = true; io.disconnect();
        setTimeout(() => { if (!stage.classList.contains("used")) play("50,0,0,0"); }, 1700);
      }, { threshold: 0.6 });
      io.observe(stage);
    }
  }

  // =====================================================================
  // 全页手势：在页面任意处按住右键画，↓ 下一屏 / ↑ 上一屏 / ←↑ 顶部 / ←↓ 底部
  // 只在 Windows 上启用：别的系统右键菜单在按下时就弹出，没法和手势共存。
  // =====================================================================
  let hint = null, hintDone = store.get("mt-page-hint") === "done";
  if (isWin && !reduce && matchMedia("(hover: hover)").matches) {
    const canvas = document.createElement("canvas");
    canvas.className = "pagecanvas";
    const pill = document.createElement("div");
    pill.className = "gpill ppill";
    pill.setAttribute("aria-hidden", "true");
    pill.innerHTML = '<span class="arrows"></span><span class="name"></span>';
    document.body.append(canvas, pill);
    const PAGE = new Map([
      ["↓", { go: 1, name: T.pageNext }], ["↑", { go: -1, name: T.pagePrev }],
      ["←↑", { go: "top", name: T.pageTop }], ["←↓", { go: "bottom", name: T.pageBottom }],
    ]);
    const trail = new Trail(canvas, pill, (dirs) => lookup(PAGE, dirs), { fixed: true });
    let lastAt = 0;
    document.addEventListener("pointerdown", (e) => {
      if (e.pointerType !== "mouse" || e.button !== 2 || e.target.closest("#stage, #screen, input, textarea")) return;
      trail.begin(e.clientX, e.clientY);
    });
    addEventListener("pointermove", (e) => { if (trail.capturing) trail.add(e.clientX, e.clientY); });
    addEventListener("pointerup", (e) => {
      if (!trail.capturing || e.button !== 2) return;
      const moved = trail.moved;
      const m = trail.end();
      if (moved) lastAt = performance.now();
      if (m) {
        goSection(m.go);
        if (hint) { hintDone = true; hint.classList.remove("on"); store.set("mt-page-hint", "done"); }
      }
    });
    document.addEventListener("contextmenu", (e) => {
      if (trail.capturing || performance.now() - lastAt < 500) e.preventDefault();
    });

    if (!hintDone) {
      hint = document.createElement("div");
      hint.className = "phint";
      hint.setAttribute("role", "note");
      hint.innerHTML = `<span>${T.pageHint}</span><button type="button">×</button>`;
      const x = $("button", hint);
      x.setAttribute("aria-label", T.hintClose);
      x.addEventListener("click", () => { hintDone = true; hint.classList.remove("on"); store.set("mt-page-hint", "done"); });
      document.body.appendChild(hint);
    }
  }

  // =====================================================================
  // 屏幕边角
  // =====================================================================
  const screen = $("#screen");
  if (screen) initEdges(screen);

  function initEdges(screen) {
    const label = $(".label", screen);
    const cursor = document.createElement("div");
    cursor.className = "cur";
    cursor.innerHTML = '<svg viewBox="0 0 14 20" width="14" height="20"><path d="M1 1v16l4.2-4 2.8 6.2 2.6-1.2-2.7-6H14L1 1Z" fill="#fff" stroke="#111" stroke-width="1.2" stroke-linejoin="round"/></svg>';
    cursor.style.opacity = "0";
    screen.appendChild(cursor);
    const rules = $$(".rules button");
    const T_EDGE = 3, CORNER = 20, DETACH = 140;
    let attached = false, touched = false, zone = 0, x = 0, y = 0, labelTimer, vol = 50, bri = 70, muted = false, playing = true;
    let rub = { dir: 0, y: 0, flips: [] };
    const names = { 12: T.start, 6: T.desktop, 9: T.switch, 3: T.tasks, 1: T.edgeWheel, 4: T.edgeWheel, 2: T.edgeRub };

    function say(text, ms = 1200) {
      label.textContent = text;
      label.classList.add("on");
      clearTimeout(labelTimer);
      labelTimer = setTimeout(() => label.classList.remove("on"), ms);
    }
    function rect() {
      const r = screen.getBoundingClientRect();
      return { l: r.left + screen.clientLeft, t: r.top + screen.clientTop, w: screen.clientWidth, h: screen.clientHeight };
    }
    function zoneAt(px, py, w, h) {
      const L = px <= T_EDGE, R = px >= w - T_EDGE, U = py <= T_EDGE, D = py >= h - T_EDGE;
      const nl = px <= CORNER, nr = px >= w - CORNER, nu = py <= CORNER, nd = py >= h - CORNER;
      if ((L || U) && nl && nu) return 9;
      if ((R || U) && nr && nu) return 3;
      if ((L || D) && nl && nd) return 12;
      if ((R || D) && nr && nd) return 6;
      return U ? 1 : R ? 2 : D ? 4 : L ? 8 : 0;
    }
    function hit(z) { rules.forEach((b) => b.classList.toggle("hit", +b.dataset.edge === z)); }
    function setZone(z) {
      if (z === zone) return;
      zone = z;
      $$(".glow", screen).forEach((g) => g.classList.toggle("on", g.classList.contains("e" + z) || g.classList.contains("c" + z)));
      if (!z) return;
      hit(z);
      // 角落：一到达就执行；边：等滚轮 / 点击 / 摩擦
      if (z === 12) screen.classList.toggle("start-open");
      else screen.classList.remove("start-open");
      if (z === 6) screen.classList.toggle("desktop");
      if (z === 9) { screen.classList.toggle("swap"); screen.classList.remove("desktop"); }
      if (z === 3) { screen.classList.add("tasks"); setTimeout(() => screen.classList.remove("tasks"), 1400); }
      if (names[z]) say(names[z]);
      rub = { dir: 0, y, flips: [] };
    }
    function moveTo(px, py) {
      const r = rect();
      x = clamp(px, 0, r.w); y = clamp(py, 0, r.h);
      cursor.style.left = x + "px"; cursor.style.top = y + "px";
      cursor.style.opacity = "1";
      const z = zoneAt(x, y, r.w, r.h);
      if (z === 2 && zone === 2) trackRub();
      setZone(z);
    }
    function trackRub() {
      const dy = y - rub.y;
      if (Math.abs(dy) < 10) return;
      const d = Math.sign(dy);
      if (rub.dir && d !== rub.dir) rub.flips.push(performance.now());
      rub.dir = d; rub.y = y;
      rub.flips = rub.flips.filter((t) => performance.now() - t < 1600);
      if (rub.flips.length >= 3) { rub.flips = []; openTaskmgr(); }
    }
    function openTaskmgr() {
      const tm = $(".tm", screen);
      tm.classList.add("on"); say(T.taskmgr);
      hit(2);
      setTimeout(() => tm.classList.remove("on"), 1800);
    }
    function osd(kind, v) {
      const el = $(`.osd[data-osd="${kind}"]`, screen);
      $("b", el).style.width = v + "%";
      $(".v", el).textContent = kind === "vol" && muted ? T.muted : v;
      el.classList.add("on");
      clearTimeout(el._t);
      el._t = setTimeout(() => el.classList.remove("on"), 1200);
      if (kind === "bri") $(".dim", screen).style.opacity = ((100 - v) / 100 * 0.6).toFixed(2);
    }
    function wheel(up) {
      if (zone === 1) { muted = false; vol = clamp(vol + (up ? 4 : -4), 0, 100); osd("vol", vol); return true; }
      if (zone === 4) { bri = clamp(bri + (up ? 5 : -5), 10, 100); osd("bri", bri); return true; }
      return false;
    }

    function detach() {
      attached = false; cursor.style.opacity = "0"; setZone(0);
      screen.style.cursor = "";
    }
    // 用户接手时清掉自动巡演留下的状态，从干净的桌面开始
    const takeOver = () => {
      if (!touched) { screen.classList.remove("start-open", "desktop", "swap"); zone = -1; }
      attached = touched = true;
    };
    screen.addEventListener("pointerenter", () => { takeOver(); screen.style.cursor = "none"; });
    screen.addEventListener("pointerdown", (e) => { if (e.pointerType !== "mouse") { takeOver(); screen.setPointerCapture(e.pointerId); } });
    addEventListener("pointermove", (e) => {
      if (!attached) return;
      const r = rect();
      const px = e.clientX - r.l, py = e.clientY - r.t;
      const out = Math.max(-px, px - r.w, -py, py - r.h, 0);
      if (out > DETACH) return detach();
      screen.style.cursor = out > 0 ? "" : "none";
      moveTo(px, py);
    });
    screen.addEventListener("pointerup", (e) => { if (e.pointerType !== "mouse") detach(); });
    addEventListener("wheel", (e) => {
      if (attached && zone && wheel(e.deltaY < 0)) e.preventDefault();
    }, { passive: false });
    addEventListener("mousedown", (e) => {
      if (!attached || zone !== 1) return;
      if (e.button === 1) { e.preventDefault(); muted = !muted; osd("vol", vol); say(muted ? T.muted : T.volume); }
      if (e.button === 2) { playing = !playing; say(playing ? "▶" : "❚❚"); }
    });
    addEventListener("contextmenu", (e) => { if (attached && zone) e.preventDefault(); });

    // --- 演示：光标从中间滑到某个边角，再做对应动作 ---
    const spot = {
      12: (r) => [0, r.h], 6: (r) => [r.w, r.h], 9: () => [0, 0], 3: (r) => [r.w, 0],
      1: (r) => [r.w * 0.5, 0], 4: (r) => [r.w * 0.5, r.h], 2: (r) => [r.w, r.h * 0.45],
    };
    let demoBusy = false;
    async function demo(z, kind) {
      if (demoBusy || attached) return;
      demoBusy = true;
      const r = rect();
      const [tx, ty] = spot[z](r);
      setZone(0);
      await glide(x || r.w * 0.5, y || r.h * 0.5, tx, ty, reduce ? 1 : 520);
      if (kind === "wheel") for (let i = 0; i < 4; i++) { await wait(140); wheel(true); }
      if (kind === "rub") {
        let cy = ty;
        for (let i = 0; i < 5; i++) { const ny = ty + (i % 2 ? -1 : 1) * r.h * 0.12; await glide(tx, cy, tx, ny, reduce ? 1 : 170); cy = ny; }
      }
      await wait(700);
      // 离开边角回到中间，下次才能再触发（和真程序一样）
      await glide(x, y, r.w * 0.5, r.h * 0.55, reduce ? 1 : 420);
      if (!attached) cursor.style.opacity = "0";
      demoBusy = false;
    }
    rules.forEach((b) => b.addEventListener("click", () => demo(+b.dataset.edge, b.dataset.with)));
    function glide(x0, y0, x1, y1, ms) {
      return new Promise((done) => {
        const t0 = performance.now();
        const step = (t) => {
          if (attached) return done();
          const k = Math.min(1, (t - t0) / ms), e = k < 0.5 ? 4 * k ** 3 : 1 - (-2 * k + 2) ** 3 / 2;
          moveTo(x0 + (x1 - x0) * e, y0 + (y1 - y0) * e);
          if (k < 1) requestAnimationFrame(step); else done();
        };
        requestAnimationFrame(step);
      });
    }
    // 第一次看到时自己巡一遍：左下开始菜单 → 右下显示桌面 → 再回来 → 上边滚轮调音量
    if (!reduce) {
      const io = new IntersectionObserver(async (es) => {
        if (!es[0].isIntersecting) return;
        io.disconnect();
        await wait(900);
        for (const [z, k] of [[12], [6], [6], [1, "wheel"]]) {
          if (touched) break;
          await demo(z, k);
          await wait(250);
        }
        screen.classList.remove("start-open");
      }, { threshold: 0.7 });
      io.observe(screen);
    }
  }

  // ---------- 程序开关 ----------
  $$("#apps .row").forEach((row) => {
    const switches = $$(".sw", row), st = $(".state", row);
    switches.forEach((sw) => sw.addEventListener("change", () => {
      switches.forEach((other) => { other.checked = sw.checked; });
      row.classList.toggle("off", !sw.checked);
      st.textContent = sw.checked ? st.dataset.on : st.dataset.off;
    }));
  });

  // ---------- 进程内存 ----------
  const proc = $("#proc");
  if (proc) {
    const engine = $(".pr:not(.webview)", proc), web = $(".pr.webview", proc), toggle = $("#proc-toggle", proc);
    const setBar = (row, on) => { const b = $(".meter b", row); b.style.width = on ? b.dataset.w + "%" : "0"; };
    proc.addEventListener("revealed", () => setTimeout(() => setBar(engine, true), 400), { once: true });
    if (reduce) setBar(engine, true);
    const count = (el, to, ms) => {
      const from = parseFloat(el.textContent) || 0, t0 = performance.now();
      const step = (t) => {
        const k = Math.min(1, (t - t0) / (reduce ? 1 : ms));
        el.textContent = (from + (to - from) * (1 - Math.pow(1 - k, 3))).toFixed(to % 1 ? 1 : 0) + " MB";
        if (k < 1) requestAnimationFrame(step);
      };
      requestAnimationFrame(step);
    };
    toggle.addEventListener("change", () => {
      const on = toggle.checked;
      web.classList.toggle("on", on);
      setBar(web, on);
      const m = $(".m", web);
      if (on) count(m, 185, 900); else m.textContent = "—";
      // 设置进程是独立的：引擎几乎不变
      count($(".m", engine), on ? 19.5 : 19.3, 400);
    });
  }

  // ---------- 下载区：鼠标划过留下轨迹 ----------
  if (get && !reduce) {
    const c = document.createElement("canvas");
    c.setAttribute("aria-hidden", "true");
    const old = $("canvas", get);
    old ? old.replaceWith(c) : get.prepend(c);
    const g = c.getContext("2d");
    let trail = [], raf = 0;
    const size = () => { const dpr = Math.min(devicePixelRatio || 1, 2); c.width = get.clientWidth * dpr; c.height = get.clientHeight * dpr; g.setTransform(dpr, 0, 0, dpr, 0, 0); };
    new ResizeObserver(size).observe(get);
    get.addEventListener("pointermove", (e) => {
      const r = get.getBoundingClientRect();
      trail.push({ x: e.clientX - r.left, y: e.clientY - r.top, t: performance.now() });
      if (!raf) raf = requestAnimationFrame(tick);
    });
    function tick(now) {
      trail = trail.filter((p) => now - p.t < 700);
      g.clearRect(0, 0, c.width, c.height);
      g.lineCap = "round"; g.lineJoin = "round";
      const col = css("--trail");
      for (let i = 1; i < trail.length; i++) {
        const a = 1 - (now - trail[i].t) / 700;
        g.strokeStyle = col; g.globalAlpha = a * 0.55; g.lineWidth = 2 + a * 4;
        g.beginPath(); g.moveTo(trail[i - 1].x, trail[i - 1].y); g.lineTo(trail[i].x, trail[i].y); g.stroke();
      }
      g.globalAlpha = 1;
      raf = trail.length ? requestAnimationFrame(tick) : 0;
    }
  }

  // ---------- 布局变化时重算轨迹线 ----------
  layoutSpine();
  let relayout = 0;
  const queueLayout = () => { cancelAnimationFrame(relayout); relayout = requestAnimationFrame(layoutSpine); };
  addEventListener("resize", queueLayout);
  new ResizeObserver(queueLayout).observe(document.body);
  document.fonts?.ready.then(queueLayout);
  $$(".faq details").forEach((d) => d.addEventListener("toggle", () => setTimeout(queueLayout, 500)));
  onScroll();
})();
