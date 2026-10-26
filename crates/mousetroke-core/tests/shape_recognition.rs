use mousetroke_core::matcher::{self, Trigger};
use mousetroke_core::recognizer::{best_score, PathTracker, Segment, ACCEPT_MAX, EXACT_EPS};
use mousetroke_core::schema::{Action, Binding, Config, MouseButton, PatternKey, Scope, ScopeKind, Step, WindowOpKind};
use mousetroke_core::shape::{self, ACCEPT, MARGIN, SAMPLE_POINTS};
use std::collections::BTreeMap;
use std::f64::consts::PI;

#[derive(serde::Deserialize)]
struct RealShapes {
    classes: BTreeMap<String, Vec<Vec<i32>>>,
    engine_g: Vec<Vec<i32>>,
    engine_straight: Vec<Vec<i32>>,
}

fn real_shapes() -> RealShapes {
    serde_json::from_str(include_str!("fixtures/real-shapes.json")).unwrap()
}

fn ranked(path: &[Point], samples: &[Vec<Vec<i32>>]) -> (usize, f64, f64) {
    let mut scores: Vec<_> = samples.iter().enumerate().map(|(id, samples)| {
        (id, samples.iter().map(|s| shape::similarity(path, &shape::decode(s).unwrap())).fold(0.0f64, f64::max))
    }).collect();
    scores.sort_by(|a, b| b.1.total_cmp(&a.1));
    (scores[0].0, scores[0].1, scores[1].1)
}

#[test]
fn real_hand_drawn_acceptance() {
    let data = real_shapes();
    let samples: Vec<_> = data.classes.values().cloned().collect();
    let candidates: Vec<_> = samples.iter().enumerate().map(|(id, s)| (id, s.as_slice())).collect();
    let g = data.classes.keys().position(|k| k == "G").unwrap();
    let (mut loo, mut engine, mut wrong) = (0, 0, 0);
    for (expected, class) in samples.iter().enumerate() {
        for (i, sample) in class.iter().enumerate() {
            let mut training = samples.clone();
            training[expected].remove(i);
            let candidates: Vec<_> = training.iter().enumerate().map(|(id, s)| (id, s.as_slice())).collect();
            match shape::best_match(&shape::decode(sample).unwrap(), &candidates) {
                Some((id, _)) if id == expected => loo += 1,
                Some(_) => wrong += 1,
                None => {},
            }
        }
    }
    for sample in &data.engine_g {
        match shape::best_match(&shape::decode(sample).unwrap(), &candidates) {
            Some((id, _)) if id == g => engine += 1,
            Some(_) => wrong += 1,
            None => {},
        }
    }
    let straight = data.engine_straight.iter().filter(|s| shape::best_match(&shape::decode(s).unwrap(), &candidates).is_some()).count();
    println!("real: leave-one-out {loo}/30, engine G {engine}/12, wrong {wrong}, straight {straight}/3");
    assert_eq!(loo, 30, "leave-one-out {loo}/30");
    assert_eq!(engine, 12, "engine G {engine}/12");
    assert_eq!(wrong, 0);
    assert_eq!(straight, 0);
}

#[test]
fn acceptance_candidate_table() {
    use shape::{classify_normalized_stroke, StrokeKind};
    let data = real_shapes();
    let real: Vec<_> = data.classes.values().cloned().collect();
    let g = data.classes.keys().position(|k| k == "G").unwrap();
    // Rank once, then scan thresholds without changing the fixtures or random streams.
    let row = |sample: &[i32], training: &[Vec<Vec<i32>>]| {
        // Re-enter through normalization, as match_chain_path does for actual mouse input.
        let raw: Vec<_> = sample.as_chunks::<2>().0.iter().map(|&[x, y]| (x as f64, y as f64)).collect();
        let path = shape::normalize(&raw).unwrap();
        (classify_normalized_stroke(&path), ranked(&path, training))
    };
    let mut loo = Vec::new();
    for (expected, class) in real.iter().enumerate() {
        for (i, sample) in class.iter().enumerate() {
            let mut training = real.clone();
            training[expected].remove(i);
            let (kind, score) = row(sample, &training);
            if data.classes.keys().nth(expected).unwrap() == "Z" {
                println!("real Z {i}: {kind:?}, score {:.6}, margin {:.6}", score.1, score.1 - score.2);
                assert_eq!(kind, StrokeKind::Polyline);
                assert!(score.1 >= ACCEPT && score.1 - score.2 >= MARGIN);
            }
            loo.push((expected, kind, score));
        }
    }
    let engine: Vec<_> = data.engine_g.iter().map(|s| { let (kind, score) = row(s, &real); (g, kind, score) }).collect();
    let straight_real: Vec<_> = data.engine_straight.iter().map(|s| row(s, &real)).collect();
    let samples = templates();
    let mut synthetic = Vec::new();
    let mut rng = Xorshift(0xd1b54a32d192ed03);
    for (expected, drawing) in SHAPES.iter().enumerate() {
        for _ in 0..300 {
            let path = shape::normalize(&noisy(*drawing, &mut rng)).unwrap();
            synthetic.push((expected, classify_normalized_stroke(&path), ranked(&path, &samples)));
        }
    }
    let mut straight = Vec::new();
    let mut breakdown = BTreeMap::<String, usize>::new();
    let mut rng = Xorshift(0x94d049bb133111eb);
    for drawing in STRAIGHTS {
        for _ in 0..300 {
            let path = shape::normalize(&noisy(drawing, &mut rng)).unwrap();
            let (kind, score) = (classify_normalized_stroke(&path), ranked(&path, &samples));
            if score.1 >= 0.70 && score.1 - score.2 >= MARGIN {
                *breakdown.entry(format!("{drawing:?} -> {:?}, {kind:?}", SHAPES[score.0])).or_default() += 1;
                assert_eq!(kind, StrokeKind::Polyline, "global 0.70 false accepts must use strict tier");
            }
            straight.push((kind, score));
        }
    }
    for (key, count) in &breakdown { println!("global 0.70 false accept (current classifier): {key}: {count}"); }
    assert_eq!(breakdown.values().sum::<usize>(), 624);
    let strict_max = straight.iter().filter(|(_, s)| s.1 - s.2 >= MARGIN).map(|(_, s)| s.1).fold(0.0f64, f64::max);
    let strict_floor = ((strict_max * 1000.0).floor() + 1.0) / 1000.0;
    println!("strict false-accept maximum {strict_max:.6}; minimum strict threshold {strict_floor:.3}");
    assert!(ACCEPT >= strict_floor);
    let (mut first_safe, mut chosen) = (None, None);
    for millis in 0..=801 {
        let curve = millis as f64 / 1000.0;
        let accepted = |kind: StrokeKind, s: &(usize, f64, f64)| {
            s.1 >= if kind == StrokeKind::Shape { curve } else { ACCEPT } && s.1 - s.2 >= MARGIN
        };
        let correct = |rows: &Vec<(usize, StrokeKind, (usize, f64, f64))>| rows.iter().filter(|(id, kind, s)| accepted(*kind, s) && *id == s.0).count();
        let wrong = loo.iter().chain(&engine).chain(&synthetic).filter(|(id, kind, s)| accepted(*kind, s) && *id != s.0).count();
        let false_synthetic = straight.iter().filter(|(kind, s)| accepted(*kind, s)).count();
        let false_real = straight_real.iter().filter(|(kind, s)| accepted(*kind, s)).count();
        let accuracy = (correct(&engine), correct(&loo), correct(&synthetic));
        if [0, 400, 401, 402, 500, 550, 600, 650, 680, 687, 688, 689, 690, 691, 695, 696, 697, 698, 699, 700, 710, 750, 760, 790, 800, 801].contains(&millis) {
            println!("curve {curve:.3}: G {}/12, LOO {}/30, synthetic {}/1800; false straight {false_synthetic}/2700 + {false_real}/3, wrong {wrong}", accuracy.0, accuracy.1, accuracy.2);
        }
        if wrong + false_synthetic + false_real == 0 {
            first_safe.get_or_insert(curve);
            // Maximize G, then LOO, then synthetic accuracy; prefer the stricter threshold on ties.
            if chosen.is_none_or(|(_, best)| accuracy >= best) { chosen = Some((curve, accuracy)); }
        }
    }
    println!("first safe curve {first_safe:?}; optimum (curve, (G, LOO, synthetic)) {chosen:?}");
    assert_eq!(chosen, Some((shape::ACCEPT_CURVE, (12, 30, 1794))));
}

type Point = (f64, f64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Drawing {
    G, C, S, O, Six, Check, Z,
    Left, Right, DownRight, RightDown, DownLeft, LeftUpRight, Square, Southwest, Northeast,
}

const SHAPES: [Drawing; 6] = [Drawing::G, Drawing::C, Drawing::S, Drawing::O, Drawing::Check, Drawing::Z];
const STRAIGHTS: [Drawing; 9] = [
    Drawing::Left, Drawing::Right, Drawing::DownRight, Drawing::RightDown, Drawing::DownLeft,
    Drawing::LeftUpRight, Drawing::Square, Drawing::Southwest, Drawing::Northeast,
];

struct Xorshift(u64);

impl Xorshift {
    fn unit(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }

    fn range(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.unit()
    }
}

fn lerp(a: Point, b: Point, t: f64) -> Point {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}

fn polyline(points: &[Point], t: f64) -> Point {
    let position = t.clamp(0.0, 1.0) * (points.len() - 1) as f64;
    let i = (position as usize).min(points.len() - 2);
    lerp(points[i], points[i + 1], position - i as f64)
}

fn bezier(points: [Point; 4], t: f64) -> Point {
    let u = 1.0 - t;
    let weights = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
    points.iter().zip(weights).fold((0.0, 0.0), |(x, y), (p, w)| (x + p.0 * w, y + p.1 * w))
}

fn point(drawing: Drawing, t: f64, variant: f64) -> Point {
    let arc = |t: f64| {
        let angle = PI / 4.0 + 1.5 * PI * t;
        (angle.cos(), -angle.sin())
    };
    let (x, y) = match drawing {
        Drawing::C => arc(t),
        Drawing::G if t < 0.78 => arc(t / 0.78),
        Drawing::G if t < 0.89 => lerp(arc(1.0), (0.82, 0.0), (t - 0.78) / 0.11),
        Drawing::G => lerp((0.82, 0.0), (0.05, 0.0), (t - 0.89) / 0.11),
        Drawing::O => {
            // 屏幕坐标 y 向下：从顶端向左行进，逆时针闭合。
            let angle = PI / 2.0 + 2.0 * PI * t;
            (angle.cos(), -angle.sin())
        }
        Drawing::S if t < 0.5 => bezier([(0.85, -0.85), (-0.95, -1.4), (-1.1, -0.15), (0.0, 0.0)], t * 2.0),
        Drawing::S => bezier([(0.0, 0.0), (1.1, 0.15), (0.95, 1.4), (-0.85, 0.85)], (t - 0.5) * 2.0),
        Drawing::Six if t < 0.4 => lerp((0.7, -1.0), (-0.65, 0.4), t / 0.4),
        Drawing::Six => {
            let angle = PI + 2.0 * PI * (t - 0.4) / 0.6;
            (angle.cos() * 0.65, 0.4 - angle.sin() * 0.6)
        },
        Drawing::Check => polyline(&[(-0.9, 0.0), (-0.3, 0.65), (1.0, -0.9)], t),
        Drawing::Z => polyline(&[(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)], t),
        Drawing::Left => polyline(&[(1.0, 0.0), (-1.0, 0.0)], t),
        Drawing::Right => polyline(&[(-1.0, 0.0), (1.0, 0.0)], t),
        Drawing::DownRight => polyline(&[(-1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)], t),
        Drawing::RightDown => polyline(&[(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0)], t),
        Drawing::DownLeft => polyline(&[(1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)], t),
        Drawing::LeftUpRight => polyline(&[(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)], t),
        Drawing::Square => polyline(&[(-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)], t),
        Drawing::Southwest => polyline(&[(1.0, -1.0), (-1.0, 1.0)], t),
        Drawing::Northeast => polyline(&[(-1.0, 1.0), (1.0, -1.0)], t),
    };
    (x * (1.0 + 0.04 * variant), y * (1.0 - 0.03 * variant))
}

fn clean(drawing: Drawing, variant: f64) -> Vec<Point> {
    (0..=160).map(|i| {
        let (x, y) = point(drawing, i as f64 / 160.0, variant);
        (x * 160.0, y * 160.0)
    }).collect()
}

fn rotate(path: &[Point], angle: f64) -> Vec<Point> {
    let (sin, cos) = angle.sin_cos();
    path.iter().map(|&(x, y)| (x * cos - y * sin, x * sin + y * cos)).collect()
}

fn noisy(drawing: Drawing, rng: &mut Xorshift) -> Vec<Point> {
    let scale = 160.0 * rng.range(0.4, 2.5);
    let angle = rng.range(-15.0, 15.0).to_radians();
    let (sin, cos) = angle.sin_cos();
    let offset = (rng.range(-1200.0, 1200.0), rng.range(-800.0, 800.0));
    let jitter = 2.0 * rng.range(0.02, 0.04);
    let variant = rng.range(-1.0, 1.0);
    // 每次改变点密度与速度；每个点独立加入相对尺寸 2%~4% 的二维扰动。
    let count = rng.range(35.0, 75.0) as usize;
    let weights: Vec<_> = (0..count).map(|_| rng.range(0.15, 2.5)).collect();
    let total: f64 = weights.iter().sum();
    let power = rng.range(0.85, 1.2);
    // 起笔位置允许小偏移；圆允许小幅相位偏移，不循环对齐起点。
    let start = rng.range(0.0, 0.012);
    let end = 1.0 - rng.range(0.0, 0.012);
    let phase = if drawing == Drawing::O { rng.range(-0.012, 0.012) } else { 0.0 };
    let mut times = vec![0.0];
    let mut sum = 0.0;
    for weight in weights {
        sum += weight;
        times.push((sum / total).powf(power));
    }
    let mut result: Vec<_> = times.iter().map(|t| {
        let t = start + t * (end - start) + phase;
        let (x, y) = point(drawing, t, variant);
        let (x, y) = (x + rng.range(-jitter, jitter), y + rng.range(-jitter, jitter));
        (offset.0 + scale * (x * cos - y * sin), offset.1 + scale * (x * sin + y * cos))
    }).collect();
    if rng.unit() < 0.25 {
        let first = result[0];
        result.insert(0, (first.0 + scale * rng.range(-0.08, 0.08), first.1 + scale * rng.range(-0.08, 0.08)));
    }
    if rng.unit() < 0.25 {
        let last = *result.last().unwrap();
        result.push((last.0 + scale * rng.range(-0.08, 0.08), last.1 + scale * rng.range(-0.08, 0.08)));
    }
    result
}

fn templates() -> Vec<Vec<Vec<i32>>> {
    SHAPES.iter().map(|&drawing| [-1.0, 1.0].iter().map(|&v| {
        shape::encode(&shape::normalize(&clean(drawing, v)).unwrap())
    }).collect()).collect()
}

// 在每个拐角切去一小段，用二次曲线连起来，再加入尺寸、速度和手抖变化。
fn hand_drawn_trace(encoded: &[i32], rng: &mut Xorshift) -> Vec<Point> {
    let vertices: Vec<_> = encoded.as_chunks::<2>().0.iter().map(|&[x, y]| (x as f64, y as f64)).collect();
    let mut path = vec![vertices[0]];
    for i in 1..vertices.len() {
        let a = *path.last().unwrap();
        let b = vertices[i];
        let radius = rng.range(0.06, 0.16);
        let before = if i + 1 < vertices.len() { lerp(b, vertices[i - 1], radius) } else { b };
        path.extend((1..=24).map(|j| lerp(a, before, j as f64 / 24.0)));
        if i + 1 < vertices.len() {
            let after = lerp(b, vertices[i + 1], radius);
            path.extend((1..=12).map(|j| {
                let t = j as f64 / 12.0;
                lerp(lerp(before, b, t), lerp(b, after, t), t)
            }));
        }
    }
    let scale = rng.range(1.5, 6.0);
    let angle = rng.range(-8.0, 8.0).to_radians();
    let (sin, cos) = angle.sin_cos();
    let offset = (rng.range(-1000.0, 1000.0), rng.range(-1000.0, 1000.0));
    path.iter().map(|&(x, y)| {
        let (x, y) = (x + rng.range(-0.6, 0.6), y + rng.range(-0.6, 0.6));
        (offset.0 + scale * (x * cos - y * sin), offset.1 + scale * (x * sin + y * cos))
    }).collect()
}

#[test]
fn stroke_classification_default_traces_and_curves() {
    use shape::{classify_stroke, StrokeKind};
    let config: Config = serde_json::from_str(include_str!("../assets/default-config.json")).unwrap();
    let mut rng = Xorshift(0x2545f4914f6cdd1d);
    let (mut traces, mut polylines) = (0, 0);
    for points in config.scopes.iter().flat_map(|s| &s.bindings).flat_map(|b| &b.pattern).filter_map(|s| {
        if let Step::Trace { points } = s { Some(points) } else { None }
    }) {
        let mut correct = 0;
        for _ in 0..300 {
            correct += usize::from(classify_stroke(&hand_drawn_trace(points, &mut rng)) == StrokeKind::Polyline);
        }
        println!("trace {points:?}: {correct}/300");
        traces += 300;
        polylines += correct;
        assert!(correct >= 297, "trace {points:?}: {correct}/300");
    }
    for encoded in [
        vec![0, 0, 50, 0, 0, 50, 50, 50],
        vec![0, 0, 0, 50, 50, 50],
        vec![0, 0, 0, 50, 50, 50, 50, 0],
    ] {
        let correct = (0..300).filter(|_| classify_stroke(&hand_drawn_trace(&encoded, &mut rng)) == StrokeKind::Polyline).count();
        println!("rounded corner {encoded:?}: {correct}/300");
        assert!(correct >= 297);
    }
    let (mut curves, mut shapes) = (0, 0);
    for drawing in [Drawing::G, Drawing::C, Drawing::O, Drawing::S, Drawing::Six] {
        let mut correct = 0;
        for _ in 0..300 {
            correct += usize::from(classify_stroke(&noisy(drawing, &mut rng)) == StrokeKind::Shape);
        }
        println!("curve {drawing:?}: {correct}/300");
        curves += 300;
        shapes += correct;
        assert!(correct >= 297, "curve {drawing:?}: {correct}/300");
    }
    let data = real_shapes();
    let mut real_correct = 0;
    let mut non_z_correct = 0;
    for (name, samples) in data.classes.iter().map(|(k, v)| (k.as_str(), v)).chain(std::iter::once(("engine G", &data.engine_g))) {
        let mut correct = 0;
        for sample in samples {
            let raw: Vec<_> = sample.as_chunks::<2>().0.iter().map(|&[x, y]| (x as f64, y as f64)).collect();
            correct += usize::from(classify_stroke(&raw) == StrokeKind::Shape);
        }
        println!("real curve {name}: {correct}/{}", samples.len());
        real_correct += correct;
        if name != "Z" {
            non_z_correct += correct;
            assert_eq!(correct, samples.len(), "real curve {name}");
        } else {
            assert_eq!(correct, 0, "真实 Z 的三段折线应遵循几何判据");
        }
    }
    println!("classification: traces {polylines}/{traces}, synthetic curves {shapes}/{curves}, real shapes {real_correct}/42");
    assert!(polylines * 100 >= traces * 99);
    assert!(shapes * 100 >= curves * 99);
    assert_eq!(non_z_correct, 37);
    assert_eq!(real_correct, 37);
    for turns in [0.5, 1.0, 2.5] {
        for count in [32, 64, 257] {
            let raw: Vec<_> = (0..count).map(|i| {
                let t = i as f64 / (count - 1) as f64;
                let angle = 2.0 * PI * turns * t;
                let radius = if turns > 1.0 { 30.0 + 100.0 * t } else { 100.0 };
                (radius * angle.cos(), radius * angle.sin())
            }).collect();
            assert_eq!(classify_stroke(&raw), StrokeKind::Shape, "arc/spiral {turns}, {count}");
        }
    }
    // Near-line jitter must not suppress curves with a clear transverse departure from their chord.
    let quarter_circle: Vec<_> = (0..=160).map(|i| {
        let angle = std::f64::consts::FRAC_PI_2 * i as f64 / 160.0;
        (angle.cos() * 100.0, angle.sin() * 100.0)
    }).collect();
    assert_eq!(classify_stroke(&quarter_circle), StrokeKind::Shape);
    for raw in [vec![], vec![(0.0, 0.0)], vec![(1.0, 1.0); 100], vec![(0.0, 0.0), (f64::NAN, 1.0)], vec![(-f64::MAX, 0.0), (f64::MAX, 1.0)]] {
        assert_eq!(classify_stroke(&raw), StrokeKind::Polyline);
    }
}

#[test]
fn normalized_classification_agrees_with_raw() {
    use shape::{classify_normalized_stroke, classify_stroke, StrokeKind};
    let check = |name: &str, raw: &[Point]| {
        let normalized = shape::normalize(raw).unwrap();
        assert_eq!(classify_stroke(raw), classify_normalized_stroke(&normalized), "{name}");
    };
    let data = real_shapes();
    let mut real = 0;
    for sample in data.classes.values().flatten().chain(&data.engine_g).chain(&data.engine_straight) {
        let raw: Vec<_> = sample.as_chunks::<2>().0.iter().map(|&[x, y]| (x as f64, y as f64)).collect();
        check("real", &raw);
        // Stored fixture coordinates are already normalized but quantized: test them directly too.
        assert_eq!(classify_stroke(&raw), classify_normalized_stroke(&shape::decode(sample).unwrap()));
        real += 1;
    }
    let mut synthetic = 0;
    for (drawings, seed) in [(&SHAPES[..], 0xd1b54a32d192ed03), (&STRAIGHTS[..], 0x94d049bb133111eb)] {
        let mut rng = Xorshift(seed);
        for &drawing in drawings {
            for _ in 0..300 {
                let raw = noisy(drawing, &mut rng);
                check("synthetic", &raw);
                // The independently jittered straight strokes that used to trigger the loose tier.
                if [Drawing::Left, Drawing::Right, Drawing::Southwest, Drawing::Northeast].contains(&drawing) {
                    assert_eq!(classify_stroke(&raw), StrokeKind::Polyline, "{drawing:?}");
                }
                synthetic += 1;
            }
        }
    }
    let config: Config = serde_json::from_str(include_str!("../assets/default-config.json")).unwrap();
    let extra = [vec![0, 0, 50, 0, 0, 50, 50, 50], vec![0, 0, 0, 50, 50, 50], vec![0, 0, 0, 50, 50, 50, 50, 0]];
    let mut rounded = 0;
    let mut rng = Xorshift(0x2545f4914f6cdd1d);
    for points in config.scopes.iter().flat_map(|s| &s.bindings).flat_map(|b| &b.pattern).filter_map(|s| {
        if let Step::Trace { points } = s { Some(points) } else { None }
    }).chain(extra.iter()) {
        for _ in 0..300 { check("rounded", &hand_drawn_trace(points, &mut rng)); rounded += 1; }
    }
    for drawing in [Drawing::G, Drawing::C, Drawing::O, Drawing::S, Drawing::Six] {
        for _ in 0..300 { check("curve", &noisy(drawing, &mut rng)); rounded += 1; }
    }
    println!("raw/normalized classification agreement: real {real}/{real}, synthetic {synthetic}/{synthetic}, rounded/additional curves {rounded}/{rounded}");
    for bad in [vec![], vec![(0.0, 0.0); SAMPLE_POINTS], vec![(f64::NAN, 0.0); SAMPLE_POINTS], vec![(f64::INFINITY, 0.0); SAMPLE_POINTS], vec![(1.0, 0.0); SAMPLE_POINTS + 1]] {
        assert_eq!(classify_normalized_stroke(&bad), StrokeKind::Polyline);
        assert_eq!(shape::acceptance_threshold(&bad), ACCEPT);
    }
}

#[test]
fn noisy_shape_accuracy() {
    let samples = templates();
    let candidates: Vec<_> = samples.iter().enumerate().map(|(id, s)| (id, s.as_slice())).collect();
    let mut rng = Xorshift(0xd1b54a32d192ed03);
    let mut results = Vec::new();
    for (expected, drawing) in SHAPES.iter().enumerate() {
        let (mut correct, mut wrong) = (0, 0);
        let (mut below_threshold, mut ambiguous) = (0, 0);
        for _ in 0..300 {
            let path = shape::normalize(&noisy(*drawing, &mut rng)).unwrap();
            match shape::best_match(&path, &candidates) {
                Some((id, _)) if id == expected => correct += 1,
                Some(_) => wrong += 1,
                None => {
                    let scores: Vec<_> = samples.iter().map(|s| s.iter().map(|p| {
                        shape::similarity(&path, &shape::decode(p).unwrap())
                    }).fold(0.0f64, f64::max)).collect();
                    if scores.iter().copied().fold(0.0f64, f64::max) < shape::acceptance_threshold(&path) { below_threshold += 1; }
                    else { ambiguous += 1; }
                }
            }
        }
        println!("{drawing:?}: correct {correct}/300 ({:.2}%), wrong {wrong}/300 ({:.2}%)", correct as f64 / 3.0, wrong as f64 / 3.0);
        println!("{drawing:?}: below threshold {below_threshold}, ambiguous {ambiguous}");
        results.push((*drawing, correct, wrong));
    }
    for (drawing, correct, wrong) in results {
        assert!(correct >= 285, "{drawing:?}: {correct} correct");
        assert_eq!(wrong, 0, "{drawing:?}");
    }
}

#[test]
fn straight_gestures_do_not_match_shapes() {
    let samples = templates();
    let candidates: Vec<_> = samples.iter().enumerate().map(|(id, s)| (id, s.as_slice())).collect();
    let mut rng = Xorshift(0x94d049bb133111eb);
    let mut total = 0;
    for drawing in STRAIGHTS {
        let mut accepted = 0;
        for _ in 0..300 {
            let path = shape::normalize(&noisy(drawing, &mut rng)).unwrap();
            if shape::best_match(&path, &candidates).is_some() {
                accepted += 1;
            }
        }
        println!("{drawing:?}: false positives {accepted}/300 ({:.2}%)", accepted as f64 / 3.0);
        total += accepted;
    }
    println!("straight total: false positives {total}/2700 ({:.3}%)", total as f64 / 27.0);
    assert_eq!(total, 0);
}

#[test]
fn normalization_preserves_geometry_and_handles_bad_paths() {
    for raw in [vec![], vec![(0.0, 0.0)], vec![(3.0, 4.0); 100], vec![(0.0, 0.0), (0.1, 0.1)],
        vec![(f64::NAN, 0.0), (1.0, 1.0)], vec![(0.0, 0.0), (f64::INFINITY, 1.0)],
        vec![(-f64::MAX, 0.0), (f64::MAX, 1.0)]] {
        assert!(shape::normalize(&raw).is_none());
    }
    let path = shape::normalize(&[(10.0, 5.0), (10.0, 5.0), (110.0, 5.0)]).unwrap();
    assert_eq!(path.len(), SAMPLE_POINTS);
    for (i, &(x, y)) in path.iter().enumerate() {
        assert!((x - (i as f64 / 63.0 - 0.5)).abs() < 1e-12);
        assert_eq!(y, 0.0);
    }
    let thin = shape::normalize(&[(0.0, 0.0), (50.0, 0.1), (100.0, 0.0)]).unwrap();
    assert!(thin.iter().all(|p| p.1.abs() < 0.002));
    let g = clean(Drawing::G, 0.0);
    let norm = shape::normalize(&g).unwrap();
    let moved: Vec<_> = g.iter().map(|&(x, y)| (x * 2.3 + 800.0, y * 2.3 - 400.0)).collect();
    assert_eq!(shape::similarity(&norm, &shape::normalize(&moved).unwrap()), 1.0);
    let centroid = norm.iter().fold((0.0, 0.0), |(x, y), p| (x + p.0, y + p.1));
    assert!(centroid.0.abs() < 1e-12 && centroid.1.abs() < 1e-12);
    let mut duplicated = Vec::new();
    for p in &g { duplicated.extend([*p; 4]); }
    assert_eq!(shape::normalize(&duplicated), Some(norm));
}

#[test]
fn encoding_and_invalid_templates() {
    let path = shape::normalize(&clean(Drawing::G, 0.0)).unwrap();
    let encoded = shape::encode(&path);
    assert_eq!(encoded.len(), 128);
    let decoded = shape::decode(&encoded).unwrap();
    for (a, b) in path.iter().zip(&decoded) {
        assert!((a.0 - b.0).abs() <= 0.0005 && (a.1 - b.1).abs() <= 0.0005);
    }
    for bad in [vec![], vec![0; 127], vec![0; 129], vec![i32::MAX; 128]] {
        assert!(shape::decode(&bad).is_none());
        assert!(shape::best_match(&path, &[(0, &[bad][..])]).is_none());
    }
    assert!(shape::best_match(&path, &[(0, &[vec![0; 128]][..])]).is_none());
    assert!(shape::best_match::<usize>(&path, &[]).is_none());
    assert!(shape::best_match(&path, &[(0, &[][..])]).is_none());
    assert!(shape::best_match(&vec![(f64::NAN, 0.0); 64], &[(0, &[encoded][..])]).is_none());
    assert_eq!(shape::similarity(&[], &path), 0.0);
    assert_eq!(shape::similarity(&vec![(0.0, 0.0); 64], &path), 0.0);
    assert_eq!(shape::similarity(&vec![(f64::MAX, 0.0); 64], &path), 0.0);
}

#[test]
fn bounded_rotation_start_point_and_direction() {
    let raw = clean(Drawing::G, 0.0);
    let path = shape::normalize(&raw).unwrap();
    let samples = vec![shape::encode(&path)];
    for degrees in [-20.0f64, -15.0, 0.0, 15.0, 20.0] {
        let rotated = shape::normalize(&rotate(&raw, degrees.to_radians())).unwrap();
        assert!(shape::similarity(&path, &rotated) > 0.999999);
        assert!(shape::best_match(&rotated, &[("g", &samples)]).is_some());
    }
    for degrees in [90.0f64, 180.0, -90.0] {
        let rotated = shape::normalize(&rotate(&raw, degrees.to_radians())).unwrap();
        assert!(shape::best_match(&rotated, &[("g", &samples)]).is_none());
    }
    // 超限部分不再对齐：21° 比 20° 多留下约 1° 的向量夹角。
    let rotated = shape::normalize(&rotate(&raw, 21.0f64.to_radians())).unwrap();
    assert!((shape::similarity(&path, &rotated) - (1.0 - 1.0 / 90.0)).abs() < 1e-10);
    let reversed: Vec<_> = raw.iter().rev().copied().collect();
    assert!(shape::best_match(&shape::normalize(&reversed).unwrap(), &[("g", &samples)]).is_none());
    let circle = clean(Drawing::O, 0.0);
    let samples = vec![shape::encode(&shape::normalize(&circle).unwrap())];
    let shifted = rotate(&circle, PI / 2.0);
    assert!(shape::best_match(&shape::normalize(&shifted).unwrap(), &[("o", &samples)]).is_none());
}

#[test]
fn samples_compete_only_across_bindings() {
    let path = shape::normalize(&clean(Drawing::G, 0.0)).unwrap();
    let samples = vec![shape::encode(&path); 2];
    let c = vec![shape::encode(&shape::normalize(&clean(Drawing::C, 0.0)).unwrap())];
    assert_eq!(shape::best_match(&path, &[(1, &samples), (2, &c)]).unwrap().0, 1);
    assert_eq!(shape::best_match(&path, &[(1, &samples), (1, &samples)]).unwrap().0, 1);
    assert!(shape::best_match(&path, &[(1, &samples), (2, &samples)]).is_none());
    let rotated = shape::normalize(&rotate(&clean(Drawing::G, 0.0), 22.0f64.to_radians())).unwrap();
    let close = vec![shape::encode(&rotated)];
    assert!(shape::best_match(&path, &[(1, &samples), (2, &close)]).is_none());
    let reversed: Vec<_> = path.iter().rev().copied().collect();
    let mixed = vec![shape::encode(&reversed), shape::encode(&path)];
    assert_eq!(shape::best_match(&path, &[(1, &mixed)]).unwrap().0, 1);
    const { assert!(ACCEPT > 0.0 && ACCEPT <= 1.0 && MARGIN > 0.0) };
}

fn binding(name: &str, pattern: Vec<Step>) -> Binding {
    Binding { name: name.into(), name_key: None, pattern,
        action: Action::WindowOp { op: WindowOpKind::Close, allow_child: false },
        execute_on_match: false, hide_label: false, secured: false, disabled: false, draft: false }
}

fn scope(bindings: Vec<Binding>) -> Scope {
    Scope { id: "test".into(), name: "test".into(), name_key: None, kind: ScopeKind::Global,
        app: None, special: None, inherit_global: true, disabled: false, bindings }
}

const RIGHT: Step = Step::Press { key: PatternKey::Mouse(MouseButton::Right) };

fn segments(path: &[Point]) -> Vec<Segment> {
    let mut tracker = PathTracker::new(25.0);
    tracker.start(path[0].0, path[0].1);
    for &(x, y) in &path[1..] { tracker.add_point(x, y); }
    tracker.strokes()
}

#[test]
fn real_engine_strokes_use_tiered_acceptance_through_match_chain() {
    let data = real_shapes();
    let scope = scope(data.classes.iter().map(|(name, templates)| binding(name, vec![RIGHT, Step::Shape { templates: templates.clone() }])).collect());
    let built = matcher::build_steps(Trigger::Button(MouseButton::Right), &[], Some(0));
    for sample in &data.engine_g {
        let raw: Vec<_> = sample.as_chunks::<2>().0.iter().map(|&[x, y]| (x as f64, y as f64)).collect();
        assert_eq!(matcher::match_chain_path(&[&scope], &built, &segments(&raw), Some(&raw)).unwrap().0.name, "G");
    }
    for sample in &data.engine_straight {
        let raw: Vec<_> = sample.as_chunks::<2>().0.iter().map(|&[x, y]| (x as f64, y as f64)).collect();
        assert!(matcher::match_chain_path(&[&scope], &built, &segments(&raw), Some(&raw)).is_none());
    }
}

#[test]
fn matcher_prefers_trace_then_shape_and_rejects_scribbles() {
    let g = clean(Drawing::G, 0.0);
    let down_right = clean(Drawing::DownRight, 0.0);
    let scope = scope(vec![
        binding("g", vec![RIGHT, Step::shape_from_samples(std::slice::from_ref(&g)).unwrap()]),
        binding("corner-shape", vec![RIGHT, Step::shape_from_samples(std::slice::from_ref(&down_right)).unwrap()]),
        binding("trace", vec![RIGHT, Step::Trace { points: vec![0, 0, 0, 50, 50, 50] }]),
    ]);
    let built = matcher::build_steps(Trigger::Button(MouseButton::Right), &[], Some(0));
    assert_eq!(matcher::match_binding_path(&scope, &built, &segments(&down_right), Some(&down_right)).unwrap().name, "trace");
    assert_eq!(matcher::match_binding_path(&scope, &built, &segments(&g), Some(&g)).unwrap().name, "g");
    assert_eq!(matcher::match_binding_path(&scope, &built, &[], Some(&g)).unwrap().name, "g");
    assert!(matcher::match_binding(&scope, &built, &segments(&g)).is_none());
    assert!(matcher::match_binding_path(&scope, &built, &segments(&g), None).is_none());
    let scribble: Vec<_> = (0..150).map(|i| ((i as f64 * 1.73).sin() * 150.0, (i as f64 * 2.91).sin() * 150.0)).collect();
    assert!(matcher::match_binding_path(&scope, &built, &segments(&scribble), Some(&scribble)).is_none());
    assert!(matcher::is_feasible_prefix(&scope, &built));
}

#[test]
fn shape_steps_obey_position_presses_wheels_and_disabled_bindings() {
    let g = clean(Drawing::G, 0.0);
    let step = Step::shape_from_samples(std::slice::from_ref(&g)).unwrap();
    let edge = Step::Edge { edge: mousetroke_core::edge::TOP };
    let wheel = Step::Press { key: PatternKey::Wheel(-1) };
    let scope = scope(vec![binding("shape", vec![edge.clone(), RIGHT, step.clone(), Step::Wheel { horizontal: false }])]);
    let built = vec![edge.clone(), RIGHT, Step::Trace { points: vec![] }, wheel.clone()];
    assert!(matcher::is_feasible_prefix(&scope, &built[..3]));
    assert_eq!(matcher::match_binding_path(&scope, &built, &[], Some(&g)).unwrap().name, "shape");
    for bad in [
        vec![edge.clone(), Step::Trace { points: vec![] }, RIGHT, wheel],
        vec![edge.clone(), RIGHT, Step::Trace { points: vec![] }],
        vec![edge.clone(), RIGHT, Step::Trace { points: vec![] }, RIGHT],
    ] { assert!(matcher::match_binding_path(&scope, &bad, &[], Some(&g)).is_none()); }
    let mut disabled = scope.clone();
    disabled.bindings[0].disabled = true;
    assert!(!matcher::is_feasible_prefix(&disabled, &built));
    assert!(matcher::match_binding_path(&disabled, &built, &[], Some(&g)).is_none());
    for pattern in [vec![edge.clone(), RIGHT, step.clone(), step], vec![edge, RIGHT, Step::Trace { points: vec![] },
        Step::shape_from_samples(std::slice::from_ref(&g)).unwrap()]] {
        let invalid = self::scope(vec![binding("two-strokes", pattern)]);
        let built = vec![built[0].clone(), RIGHT, Step::Trace { points: vec![] }, Step::Trace { points: vec![] }];
        assert!(matcher::match_binding_path(&invalid, &built, &[], Some(&g)).is_none());
    }
}

#[test]
fn chain_prefers_specific_scope_before_trace_priority() {
    let g = clean(Drawing::G, 0.0);
    let down_right = clean(Drawing::DownRight, 0.0);
    let mut app = scope(vec![binding("app-shape", vec![RIGHT, Step::shape_from_samples(&[g.clone(), down_right.clone()]).unwrap()])]);
    app.kind = ScopeKind::App;
    let global = scope(vec![
        binding("global-trace", vec![RIGHT, Step::Trace { points: vec![0, 0, 0, 50, 50, 50] }]),
        binding("global-shape", vec![RIGHT, Step::shape_from_samples(std::slice::from_ref(&g)).unwrap()]),
    ]);
    let built = matcher::build_steps(Trigger::Button(MouseButton::Right), &[], Some(0));
    let chain = [&app, &global];
    assert_eq!(matcher::match_chain_path(&chain, &built, &segments(&down_right), Some(&down_right)).unwrap().0.name, "app-shape");
    assert_eq!(matcher::match_chain(&chain, &built, &segments(&down_right)).unwrap().0.name, "global-trace");
    let (binding, matched_scope) = matcher::match_chain_path(&chain, &built, &segments(&g), Some(&g)).unwrap();
    assert_eq!(binding.name, "app-shape");
    assert!(std::ptr::eq(matched_scope, &app));
    assert!(matcher::match_chain(&chain, &built, &segments(&g)).is_none());
    assert_eq!(matcher::match_chain_path(&[&global], &built, &[], Some(&g)).unwrap().0.name, "global-shape");
    assert!(matcher::match_chain_path(&chain, &built, &[], None).is_none());
}

#[test]
fn app_shape_allows_global_straight_fallback() {
    let g = clean(Drawing::G, 0.0);
    let straight = clean(Drawing::Right, 0.0);
    let mut app = scope(vec![binding("app-g", vec![RIGHT, Step::shape_from_samples(std::slice::from_ref(&g)).unwrap()])]);
    app.kind = ScopeKind::App;
    let global = scope(vec![binding("global-straight", vec![RIGHT, Step::Trace { points: vec![0, 0, 50, 0] }])]);
    let built = matcher::build_steps(Trigger::Button(MouseButton::Right), &[], Some(0));
    let chain = [&app, &global];
    let (hit, matched_scope) = matcher::match_chain_path(&chain, &built, &segments(&g), Some(&g)).unwrap();
    assert_eq!(hit.name, "app-g");
    assert!(std::ptr::eq(matched_scope, &app));
    assert!(matcher::match_binding_path(&app, &built, &segments(&straight), Some(&straight)).is_none());
    let (hit, matched_scope) = matcher::match_chain_path(&chain, &built, &segments(&straight), Some(&straight)).unwrap();
    assert_eq!(hit.name, "global-straight");
    assert!(std::ptr::eq(matched_scope, &global));
}

#[test]
fn path_matching_prefers_exact_trace_then_shape_then_fuzzy_trace() {
    let path = vec![(0.0, 0.0), (0.0, 200.0), (200.0, 200.0)];
    let segs = segments(&path);
    let fuzzy = vec![0, 0, 50, 50, 100, 50];
    let exact = vec![0, 0, 0, 50, 50, 50];
    let fuzzy_score = best_score(&[(&fuzzy, &segs)]).unwrap().1;
    assert!((EXACT_EPS..=ACCEPT_MAX).contains(&fuzzy_score));
    assert!(best_score(&[(&exact, &segs)]).unwrap().1 < EXACT_EPS);
    let mut scope = scope(vec![
        binding("fuzzy", vec![RIGHT, Step::Trace { points: fuzzy }]),
        binding("shape", vec![RIGHT, Step::shape_from_samples(std::slice::from_ref(&path)).unwrap()]),
    ]);
    let built = matcher::build_steps(Trigger::Button(MouseButton::Right), &[], Some(0));
    assert_eq!(matcher::match_binding(&scope, &built, &segs).unwrap().name, "fuzzy");
    assert_eq!(matcher::match_chain(&[&scope], &built, &segs).unwrap().0.name, "fuzzy");
    assert_eq!(matcher::match_binding_path(&scope, &built, &segs, Some(&path)).unwrap().name, "shape");
    assert_eq!(matcher::match_chain_path(&[&scope], &built, &segs, Some(&path)).unwrap().0.name, "shape");
    // Missing, degenerate, and rejected raw paths still allow the fuzzy fallback.
    let straight = clean(Drawing::Right, 0.0);
    for raw in [None, Some(&[][..]), Some(straight.as_slice())] {
        assert_eq!(matcher::match_binding_path(&scope, &built, &segs, raw).unwrap().name, "fuzzy");
    }
    scope.bindings.push(binding("exact", vec![RIGHT, Step::Trace { points: exact }]));
    assert_eq!(matcher::match_binding_path(&scope, &built, &segs, Some(&path)).unwrap().name, "exact");
    assert_eq!(matcher::match_chain_path(&[&scope], &built, &segs, Some(&path)).unwrap().0.name, "exact");
}

#[test]
fn chain_keeps_fuzzy_match_ahead_of_less_specific_exact_trace() {
    let path = vec![(0.0, 0.0), (0.0, 200.0), (200.0, 200.0)];
    let g = clean(Drawing::G, 0.0);
    let mut app = scope(vec![binding("app-g", vec![RIGHT, Step::shape_from_samples(&[g]).unwrap()])]);
    app.kind = ScopeKind::App;
    let mut special = scope(vec![binding("special-fuzzy", vec![RIGHT, Step::Trace { points: vec![0, 0, 50, 50, 100, 50] }])]);
    special.kind = ScopeKind::Special;
    let global = scope(vec![binding("global-exact", vec![RIGHT, Step::Trace { points: vec![0, 0, 0, 50, 50, 50] }])]);
    let built = matcher::build_steps(Trigger::Button(MouseButton::Right), &[], Some(0));
    let (hit, matched_scope) = matcher::match_chain_path(&[&app, &special, &global], &built, &segments(&path), Some(&path)).unwrap();
    assert_eq!(hit.name, "special-fuzzy");
    assert!(std::ptr::eq(matched_scope, &special));
}

#[test]
fn shape_step_round_trips_and_helper_filters_degenerate_samples() {
    let g = clean(Drawing::G, 0.0);
    let step = Step::shape_from_samples(&[vec![], vec![(1.0, 1.0); 10], g.clone(), g]).unwrap();
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.starts_with("{\"type\":\"shape\",\"templates\":[["));
    assert_eq!(serde_json::from_str::<Step>(&json).unwrap(), step);
    let Step::Shape { templates } = step.clone() else { unreachable!() };
    assert_eq!(templates.len(), 2);
    assert_eq!(templates[0].len(), 128);
    let mut cfg = Config::default();
    cfg.scopes.push(scope(vec![binding("shape", vec![RIGHT, step])]));
    assert_eq!(Config::from_json(&cfg.to_json()).unwrap(), cfg);
    assert!(Step::shape_from_samples(&[]).is_none());
    assert!(Step::shape_from_samples(&[vec![(0.0, 0.0); 3]]).is_none());
}

#[test]
fn preset_defaults_serialization_includes_default_update_check() {
    let fixture = include_str!("../assets/default-config.json");
    let cfg = Config::from_json(fixture).unwrap();
    assert!(cfg.settings.update_check);
    let json = cfg.to_json();
    let fingerprint = json.bytes().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ byte as u64).wrapping_mul(0x100000001b3)
    });
    println!("preset defaults: {} bytes, FNV-1a {fingerprint:016x}", json.len());
    // 统一资源改用预置动作；仍冻结默认配置（含 update_check）的序列化基线。
    assert_eq!(json.len(), 36_440);
    assert_eq!(fingerprint, 0xba538536bd2a8877);
    assert_eq!(Config::from_json(&json).unwrap().to_json(), json);
}

#[test]
#[ignore = "release timing: cargo test -p mousetroke-core --release -- --include-ignored --nocapture"]
fn timing_100_templates() {
    use std::hint::black_box;
    use std::time::Instant;
    let samples = templates();
    let candidates: Vec<_> = (0..50).map(|id| (id, samples[id % samples.len()].as_slice())).collect();
    assert_eq!(candidates.iter().map(|(_, s)| s.len()).sum::<usize>(), 100);
    let path = shape::normalize(&clean(Drawing::G, 0.0)).unwrap();
    for _ in 0..100 { black_box(shape::best_match(black_box(&path), black_box(&candidates))); }
    let mut times = Vec::new();
    for _ in 0..21 {
        let start = Instant::now();
        for _ in 0..200 { black_box(shape::best_match(black_box(&path), black_box(&candidates))); }
        times.push(start.elapsed().as_nanos() as f64 / 200.0 / 1000.0);
    }
    times.sort_by(f64::total_cmp);
    println!("64-point classification + 100 templates: median {:.3} us, batch maximum {:.3} us (21 x 200 matches)", times[10], times[20]);
    assert!(times[10] < 1000.0, "100 templates must take less than 1 ms");
}
