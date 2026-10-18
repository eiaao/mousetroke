try { localStorage.setItem("mousetroke.guide.seen", "1"); } catch {}
document.querySelector(".guide-backdrop") && document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
setTimeout(() => {
  const el = document.querySelectorAll(".row .switch")[0];
  el.dispatchEvent(new MouseEvent("mouseover", { bubbles: true }));
}, 300);
