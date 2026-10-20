//! 输入回调只排队；窗口显示、文字测量和 Core Animation 事务全部延至绘制计时器。
use mousetroke_engine::Canvas;
use std::collections::VecDeque;

enum Command {
    Color(String),
    Width(f64),
    Start(i32, i32),
    Line(i32, i32),
    Finish,
    Fade,
    Label(String, i32, i32, String),
    ClearLabel,
}

pub(crate) struct DeferredCanvas {
    inner: Box<dyn Canvas>,
    queue: VecDeque<Command>,
    rect: (i32, i32, i32, i32),
}

impl DeferredCanvas {
    pub(crate) fn new(inner: Box<dyn Canvas>) -> Self {
        let rect = inner.virtual_rect();
        Self {
            inner,
            queue: VecDeque::with_capacity(1024),
            rect,
        }
    }

    fn push(&mut self, command: Command) {
        if self.queue.len() == self.queue.capacity() {
            // 过载时丢弃尚未绘制的帧并清屏，不能无限扩容或保留旧轨迹。
            self.queue.clear();
            self.queue.push_back(Command::Finish);
        }
        self.queue.push_back(command);
    }
}

fn text(value: &str) -> String {
    value.chars().take(2048).collect()
}

impl Canvas for DeferredCanvas {
    fn set_color(&mut self, value: &str) {
        self.push(Command::Color(text(value)));
    }
    fn set_width(&mut self, width: f64) {
        self.push(Command::Width(width));
    }
    fn start_session(&mut self, x: i32, y: i32) {
        self.push(Command::Start(x, y));
    }
    fn line_to(&mut self, x: i32, y: i32) {
        self.push(Command::Line(x, y));
    }
    fn finish(&mut self) {
        self.push(Command::Finish);
    }
    fn fade_out(&mut self) {
        self.push(Command::Fade);
    }
    fn draw_label(&mut self, value: &str, x: i32, y: i32, color: &str) {
        self.push(Command::Label(text(value), x, y, text(color)));
    }
    fn clear_label(&mut self) {
        self.push(Command::ClearLabel);
    }
    fn virtual_rect(&self) -> (i32, i32, i32, i32) {
        self.rect
    }
    fn tick_fade(&mut self) {
        while let Some(command) = self.queue.pop_front() {
            match command {
                Command::Color(value) => self.inner.set_color(&value),
                Command::Width(width) => self.inner.set_width(width),
                Command::Start(x, y) => self.inner.start_session(x, y),
                Command::Line(x, y) => self.inner.line_to(x, y),
                Command::Finish => self.inner.finish(),
                Command::Fade => self.inner.fade_out(),
                Command::Label(value, x, y, color) => self.inner.draw_label(&value, x, y, &color),
                Command::ClearLabel => self.inner.clear_label(),
            }
        }
        self.inner.tick_fade();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    struct Probe(Rc<RefCell<Vec<&'static str>>>);
    impl Canvas for Probe {
        fn set_color(&mut self, _: &str) {
            self.0.borrow_mut().push("color");
        }
        fn set_width(&mut self, _: f64) {
            self.0.borrow_mut().push("width");
        }
        fn start_session(&mut self, _: i32, _: i32) {
            self.0.borrow_mut().push("start");
        }
        fn line_to(&mut self, _: i32, _: i32) {
            self.0.borrow_mut().push("line");
        }
        fn finish(&mut self) {
            self.0.borrow_mut().push("finish");
        }
        fn fade_out(&mut self) {
            self.0.borrow_mut().push("fade");
        }
        fn draw_label(&mut self, _: &str, _: i32, _: i32, _: &str) {
            self.0.borrow_mut().push("label");
        }
        fn clear_label(&mut self) {
            self.0.borrow_mut().push("clear");
        }
        fn tick_fade(&mut self) {
            self.0.borrow_mut().push("tick");
        }
        fn virtual_rect(&self) -> (i32, i32, i32, i32) {
            (0, 0, 100, 100)
        }
    }

    #[test]
    fn input_never_calls_the_renderer_and_tick_preserves_order() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut canvas = DeferredCanvas::new(Box::new(Probe(calls.clone())));
        canvas.set_color("red");
        canvas.set_width(3.0);
        canvas.start_session(1, 2);
        canvas.line_to(3, 4);
        canvas.draw_label("label", 1, 2, "white");
        canvas.fade_out();
        canvas.clear_label();
        canvas.finish();
        assert!(calls.borrow().is_empty());
        canvas.tick_fade();
        assert_eq!(
            *calls.borrow(),
            ["color", "width", "start", "line", "label", "fade", "clear", "finish", "tick"]
        );
        for i in 0..10_000 {
            canvas.line_to(i, i);
        }
        assert!(canvas.queue.len() <= 1024);
        canvas.tick_fade();
        assert!(calls.borrow().contains(&"finish"));
    }
}
