//! 禁用事件的纯决策：时间由调用者传入，不依赖 TCC、run loop 或系统时钟。
use std::time::Duration;

pub(crate) const WINDOW: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    Reenable,
    Fault,
}

pub(crate) fn decide(permissions_ready: bool, timeouts: &[Duration], now: Duration) -> Action {
    if !permissions_ready
        || timeouts
            .iter()
            .filter(|&&at| at <= now && now - at <= WINDOW)
            .take(2)
            .count()
            >= 2
    {
        Action::Fault
    } else {
        Action::Reenable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const fn s(value: u64) -> Duration {
        Duration::from_secs(value)
    }

    #[test]
    fn revocation_never_reenables_even_without_timeouts() {
        assert_eq!(decide(false, &[], s(50)), Action::Fault);
        assert_eq!(decide(false, &[s(50)], s(50)), Action::Fault);
    }

    #[test]
    fn user_disable_and_isolated_timeout_can_recover() {
        assert_eq!(decide(true, &[], s(50)), Action::Reenable);
        assert_eq!(decide(true, &[s(50)], s(50)), Action::Reenable);
        assert_eq!(decide(true, &[s(19), s(50)], s(50)), Action::Reenable);
    }

    #[test]
    fn second_timeout_including_boundary_faults_and_expires() {
        assert_eq!(decide(true, &[s(20), s(50)], s(50)), Action::Fault);
        assert_eq!(decide(true, &[s(20), s(49)], s(50)), Action::Fault);
        assert_eq!(decide(true, &[s(20), s(50)], s(51)), Action::Reenable);
        assert_eq!(decide(true, &[s(50), s(80)], s(50)), Action::Reenable);
    }
}
