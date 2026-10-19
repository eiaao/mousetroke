# Changelog

Each version lists its highlights in English, then in Chinese. The release page, the in-app update notes and the website changelog are all generated from this file.

## 0.1.1 — 2026-10-19

### English

- Shape gestures: draw a curve such as G, C, O, S or 6 to run an action. Record the same shape 2–5 times and Mousetroke learns how you draw it.
- The recorder tells lines from curves by itself: a straight stroke becomes a line gesture, a curve becomes a shape. One click switches it if it guessed wrong.
- Recognition is tuned on real hand-drawn strokes: loose, wobbly curves are accepted, while straight gestures are never mistaken for shapes.
- Endless scribbling no longer keeps the gesture going: after 100 direction changes it is cancelled with a short message, and nothing runs.
- Check for updates: automatically once a day (can be turned off in Preferences) or from About. Updating takes one click: the new version is downloaded, its checksum verified, and Mousetroke restarts itself.
- About now links to the website, the source code and feedback.

### 中文

- 形状手势：画 G、C、O、S、6 这类曲线来执行动作。同一个形状录 2–5 次，Mousetroke 会记住你的画法。
- 录制时自动区分直线和曲线：直的录成轨迹，弯的录成形状；判断错了点一下就能切换。
- 识别阈值用真实手绘样本校准：画得随意、有点抖的曲线也能认出来，直线手势不会被误认成形状。
- 乱画不再无限持续：方向变化超过 100 次会自动取消并给个提示，不执行任何动作。
- 检查更新：默认每天自动检查一次（可在偏好设置里关闭），也可以在“关于”里手动检查。一键更新：自动下载、校验 SHA-256、替换并重启。
- “关于”页加入主页、源代码和反馈入口。

## 0.1.0 — 2026-10-18

### English

- First public release: mouse gestures with the right, middle or side buttons, about 50 built-in gestures, screen corners and edges, eight action types, per-app gestures, and an interface in ten languages.

### 中文

- 首个公开版本：右键、中键、侧键手势，约 50 个内置手势，屏幕边角触发，八类动作，按程序区分手势，界面支持十种语言。
