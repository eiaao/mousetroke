use super::*;

/// 模拟 60fps 播放到结束，返回总时长；顺带检查像素是合法的预乘值（颜色分量不超过不透明度）。
fn run(name: &str, w: i32, h: i32, at: (f32, f32), capture: bool) -> f32 {
    let tex = capture.then(|| (0..w * h).map(|i| 0xFF00_0000 | (i as u32).wrapping_mul(2_654_435_761) >> 8).collect());
    let mut fx = create(name, w, h, at, tex).expect(name);
    let mut px = vec![0u32; (w * h) as usize];
    let dt = 1.0 / 60.0;
    let mut t = 0.0;
    let mut frame = 0;
    loop {
        let more = fx.frame(&mut Canvas { px: &mut px, w, h }, t, dt);
        if frame % 7 == 0 {
            for &p in &px {
                let a = p >> 24;
                assert!((p >> 16 & 255) <= a && (p >> 8 & 255) <= a && (p & 255) <= a, "{name}: pixel {p:08x} is not premultiplied (t={t:.2})");
            }
        }
        if !more {
            return t;
        }
        t += dt;
        frame += 1;
        assert!(t < 15.0, "{name} never finished");
    }
}

#[test]
fn every_effect_finishes_at_any_size_and_position() {
    for &name in ALL {
        for (w, h) in [(240, 150), (150, 240), (3, 2)] {
            let (wf, hf) = (w as f32, h as f32);
            for at in [(wf * 0.5, hf * 0.5), (0.0, 0.0), (wf - 1.0, hf - 1.0), (-50.0, hf * 2.0)] {
                for capture in [false, true] {
                    let t = run(name, w, h, at, capture);
                    assert!(t > 1.0, "{name} finished too early: {t}");
                }
            }
        }
    }
}

#[test]
fn unknown_name_and_bad_capture_are_rejected_safely() {
    assert!(create("nope", 100, 100, (0.0, 0.0), None).is_none());
    assert!(create("bomb", 0, 100, (0.0, 0.0), None).is_none());
    // 截图尺寸不符时当作没有截图，不越界
    run_with_bad_capture();
}

fn run_with_bad_capture() {
    let (w, h) = (64, 48);
    let mut fx = create("black_hole", w, h, (10.0, 10.0), Some(vec![0xFFFF_FFFF; 10])).unwrap();
    let mut px = vec![0u32; (w * h) as usize];
    let mut t = 0.0;
    while fx.frame(&mut Canvas { px: &mut px, w, h }, t, 1.0 / 30.0) {
        t += 1.0 / 30.0;
    }
}

#[test]
fn capture_list_matches_effects() {
    for &name in ALL {
        assert!(create(name, 10, 10, (5.0, 5.0), None).is_some(), "{name}");
    }
    assert!(wants_capture("bomb") && wants_capture("black_hole") && !wants_capture("crack"));
}
