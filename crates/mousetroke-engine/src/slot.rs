//! 引擎单例：线程局部，输入回调与平台定时器都在安装钩子的线程上访问。
use std::cell::RefCell;

use crate::Engine;

thread_local! {
    static ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
}

/// 把引擎放进当前线程；之后的 [`with_engine`] 都在这个线程调用。
pub fn install(engine: Engine) {
    ENGINE.with(|slot| *slot.borrow_mut() = Some(engine));
}

/// 退出时取出并释放引擎。
pub fn uninstall() -> Option<Engine> {
    ENGINE.with(|slot| slot.borrow_mut().take())
}

/// 访问引擎。重入（借用中）或未安装时返回 `None`，绝不 panic（回调里 panic 会越过 FFI）。
pub fn with_engine<R>(f: impl FnOnce(&mut Engine) -> R) -> Option<R> {
    ENGINE.with(|slot| slot.try_borrow_mut().ok().and_then(|mut slot| slot.as_mut().map(f)))
}
