# Stroke-aware Acceptance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Recover real G recognition while retaining zero observed straight-stroke and wrong-class accepts.

**Architecture:** Share the existing classifier between raw recording and normalized matching. Select the acceptance threshold from the drawn stroke's geometry in core, and expose per-sample limits to settings hints.

**Tech Stack:** Rust core/Tauri, React/TypeScript, deterministic synthetic and recorded fixtures.

**Spec:** User review of commit `0de1c92`, round 2, 2026-10-19 (requirements reproduced below).

## Global Constraints

- Work only in this worktree on `feat/shape-tune`; one new commit on `0de1c92`, no amend or push.
- Every cargo command: `CARGO_TARGET_DIR=D:\project\reaWGestures2\.claude\worktrees\target-shape-tune`, `CARGO_BUILD_JOBS=2`.
- Keep recorder classification/switching, complex-trace hint, fixture and i18n.
- Strict threshold at least the zero-false-accept value; curve threshold selected by zero-error scan, then G, leave-one-out and synthetic accuracy.
- Classify only the normalized 64-point path in matching; smoke on port 8768 in zh-CN, en-US and de-DE after UI changes.

## Review Focus

- Polyline strokes resembling Z/check templates must retain strict acceptance.
- Real Z samples must still recognize under the strict tier; mixed curve/polyline sample hints need per-sample limits.
- Matching must reject malformed normalized paths without unbounded classification.
- Recorded G paths must reach at least 10/12 without wrong-class accepts.
- Full-cap release processing with 100 templates must include classification and stay below 1 ms.

### Task 1: Diagnose and calibrate

**Files:** `crates/mousetroke-core/tests/shape_recognition.rs`.

- [x] Reproduce 624 false accepts at 0.70; group by drawing, matched class and raw classification.
- [x] Scan curve thresholds with the strict tier fixed; retain zero straight and wrong-class accepts and report all accuracy counts.

### Task 2: Core matching and regression coverage

**Files:** `crates/mousetroke-core/src/shape.rs`, `crates/mousetroke-core/tests/shape_recognition.rs`, `crates/mousetroke-win/src/engine_tests.rs`.

**Interfaces:** `classify_normalized_stroke(&[(f64, f64)]) -> StrokeKind`; `acceptance_threshold(&[(f64, f64)]) -> f64`; existing `best_match` consumes the selected threshold.

- [x] Extract the fixed-size classifier, retain its existing checks and add a near-line jitter guard, add curve constant/data comments and wire `best_match`.
- [x] Assert raw/normalized agreement on recorded, synthetic and rounded-trace sets; raise fixture accuracy assertions and pin scan selection.
- [x] Extend release timing checks to include both classification tiers and full-cap releases.

### Task 3: Shared settings hints and delivery

**Files:** `crates/mousetroke-settings/src/lib.rs`, `crates/mousetroke-settings/ui/src/api.ts`, `crates/mousetroke-settings/ui/src/components/ShapeRecorder.tsx`, `target/uitest/mock.js`, `target/uitest/smoke.mjs`.

**Interfaces:** `shape_limits(sample)` returns core-selected `accept` and `margin` for one encoded drawn sample. Pair consistency checks both directions; conflicts warn if either direction approaches its acceptance threshold.

- [x] Test settings limits against core for curves, polylines and invalid encodings; update UI hints and smoke coverage.
- [x] Run all requested debug/release tests, both builds, i18n/typecheck/UI build, and three-language smoke on 8768.
- [x] Review the final diff; record calibration, agreement, timing, checks and limitations; prepare the single Chinese-titled commit with the requested coauthor.

## Calibration results

Baseline `0de1c92`, single threshold 0.70, unchanged seeds and templates:

| Drawn stroke → matched template | Drawn Polyline | Drawn Shape | Total |
| --- | ---: | ---: | ---: |
| DownRight (↓→) → C | 103 | 0 | 103 |
| Right (→) → Check (✓) | 217 | 4 | 221 |
| Northeast (↗) → Check (✓) | 283 | 17 | 300 |
| Total | 603 | 21 | 624 |

The other six straight/polyline families contribute zero. No false accept matches Z; 103 match the curved C template, so the template-side hypothesis is only partly supported.

Two tiers alone, with the original classifier, leave 21 false accepts at curve=0.70; curve=0.80 gives zero but only 6/12 G. The 21 are jittered straight lines. Retain the classifier's RDP, stem-turn and quantization checks, preceded by a near-line guard: endpoint chord ≥90% of bounding-box diagonal and transverse deviation ≤8% of chord. All 624 now classify Polyline. Existing classification tests retain 11098/11100 default traces, 1500/1500 synthetic curves and all 37 non-Z real shapes; all five real Z remain Polyline.

Scan 0.000–0.801 in steps of 0.001, fixing strict `ACCEPT=0.801` and `MARGIN=0.04`. Rank once using the same random streams. Recorded inputs also pass through the normalization used by `match_chain_path`:

| Curve threshold | Real G /12 | LOO /30 | Synthetic correct /1800 | Synthetic false /2700 | Real straight false /3 | Wrong class (real + synthetic) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 0.000 | 12 | 30 | 1794 | 25 | 0 | 0 |
| 0.400 | 12 | 30 | 1794 | 1 | 0 | 0 |
| 0.401 | 12 | 30 | 1794 | 1 | 0 | 0 |
| 0.402 | 12 | 30 | 1794 | 0 | 0 | 0 |
| 0.600 | 12 | 30 | 1794 | 0 | 0 | 0 |
| 0.650 | 12 | 30 | 1794 | 0 | 0 | 0 |
| 0.680 | 12 | 30 | 1794 | 0 | 0 | 0 |
| **0.687** | **12** | **30** | **1794** | **0** | **0** | **0** |
| 0.688 | 11 | 30 | 1794 | 0 | 0 | 0 |
| 0.690 | 11 | 30 | 1794 | 0 | 0 | 0 |
| 0.700 | 11 | 30 | 1794 | 0 | 0 | 0 |
| 0.710 | 10 | 30 | 1794 | 0 | 0 | 0 |
| 0.750 | 9 | 30 | 1794 | 0 | 0 | 0 |
| 0.760 | 8 | 29 | 1794 | 0 | 0 | 0 |
| 0.790 | 6 | 28 | 1793 | 0 | 0 | 0 |
| 0.800 | 6 | 28 | 1793 | 0 | 0 | 0 |
| 0.801 | 6 | 28 | 1793 | 0 | 0 | 0 |

Select the highest threshold preserving the lexicographic optimum (G, LOO, synthetic): `ACCEPT_CURVE=0.687`. Directly decoded fixtures would permit 0.690; using the actual matching entry avoids losing one G to resampling. Both entries pass 12/12 G and 30/30 LOO at the chosen value. Strict synthetic maximum is 0.800959, retaining the strict floor 0.801.

Synthetic accuracy is 99.67%: G/C/S/O/Z each 300/300, Check 294/300; six Check samples are rejected, no wrong-class accepts. Real Z LOO scores after actual-input normalization are 0.893845, 0.866449, 0.920619, 0.918763, 0.894589. All five exceed 0.801 and the minimum margin is 0.214692.

Raw/normalized classification decisions agree for 45/45 recorded, 4500/4500 calibration synthetic and 13500/13500 rounded/default traces plus additional curves: **18045/18045 (100%)**. Recorded encoded coordinates also agree when classified directly, without resampling. Matching classification receives exactly 64 points; RDP is bounded by O(64²), other scans/quantization by O(64), with O(64) temporary storage.

## Release timing

Core `best_match`, classification included, 100 templates, 21 batches ×200 matches: median **30.648 µs**, maximum batch average **41.922 µs**.

Full release callback, 2048-point cap + normalization + classification +100 templates, 101 releases per case:

| Input tier | Accepted | Median µs | Maximum µs |
| --- | --- | ---: | ---: |
| Shape | no (tied bindings) | 68.500 | 118.300 |
| Shape | yes | 85.900 | 131.200 |
| Polyline | no (tied bindings) | 39.500 | 55.600 |
| Polyline | yes | 50.100 | 126.600 |

Worst observed full callback **0.1312 ms**, below the 1 ms assertion. These are local wall-clock samples with other work running, not a hard scheduling guarantee.

## Validation and limits

Every Cargo invocation used `CARGO_TARGET_DIR=D:\project\reaWGestures2\.claude\worktrees\target-shape-tune` and `CARGO_BUILD_JOBS=2`. No engine production code or translations changed. Settings obtains each drawn sample's threshold from the same core function; pair consistency checks both directions, and conflicts warn if either direction approaches its threshold. Three-language smoke also exercises a 0.72 score between the tiers for curve consistency, mixed curve/polyline consistency and curve conflicts.

The finite fixture and deterministic synthetic streams establish observed regression counts, not accuracy for every handwriting style. Near-line curves can use the strict tier; classification is a geometric heuristic, and manual recorder type switching remains available. UI browser smoke uses a mock backend; Rust settings tests separately verify per-sample limits on the real fixture. The UI build retains Vite's existing >500 kB bundle warning.


Final checks (all exit 0; release test runs include `--nocapture` to record calibration/timing output):

| Command | Result |
| --- | --- |
| `cargo test --workspace --locked -- --skip new_session_clears_interrupted_fade` | 154 passed; 5 ignored; the requested fade test excluded |
| `cargo test -p mousetroke-core --release -- --include-ignored --nocapture` | 88 passed; no ignored tests |
| `cargo test -p mousetroke-win --release -- --include-ignored --skip new_session_clears_interrupted_fade --nocapture` | 61 passed; requested fade test excluded |
| `cargo build -p mousetroke-settings` | Passed |
| `cargo build -p mousetroke` | Passed |
| `python scripts\i18n_lint.py --check` | Passed |
| `npm run typecheck` (settings UI) | Passed |
| `npm run build` (settings UI) | Passed; existing bundle-size warning |
| `node smoke.mjs <lang> http://127.0.0.1:8768/?lang=<lang>` | zh-CN, en-US, de-DE: `errors: []` each |
| `git diff --check` | Passed |
