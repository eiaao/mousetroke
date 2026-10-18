(() => {
  const q = (s) => document.querySelector(s);
  const out = {};
  for (const sel of ['.art-trail', '.art-start', '.art-window']) {
    const el = q(sel); if (!el) { out[sel] = 'missing'; continue; }
    const cs = getComputedStyle(el);
    out[sel] = { stroke: cs.stroke, fill: cs.fill, opacity: cs.opacity, dash: cs.strokeDasharray, off: cs.strokeDashoffset, anim: cs.animationName, vis: cs.visibility, display: cs.display };
  }
  document.title = JSON.stringify(out);
})();
