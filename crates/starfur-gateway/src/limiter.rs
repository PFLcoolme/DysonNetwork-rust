use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Mutex,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct RateLimiter {
    limit: u32,
    window_length: Duration,
    windows: Mutex<HashMap<IpAddr, Window>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LimitDecision {
    pub allowed: bool,
    pub limit: u32,
    pub remaining: u32,
    pub retry_after_seconds: u64,
}

#[derive(Debug)]
struct Window {
    started_at: Instant,
    requests: u32,
}

impl RateLimiter {
    pub fn per_minute(limit: u32) -> Self {
        Self {
            limit,
            window_length: Duration::from_secs(60),
            windows: Mutex::new(HashMap::new()),
        }
    }

    pub fn check(&self, client: IpAddr) -> LimitDecision {
        self.check_at(client, Instant::now())
    }

    fn check_at(&self, client: IpAddr, now: Instant) -> LimitDecision {
        if self.limit == 0 {
            return LimitDecision {
                allowed: true,
                limit: 0,
                remaining: 0,
                retry_after_seconds: 0,
            };
        }

        let mut windows = self
            .windows
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let window = windows.entry(client).or_insert(Window {
            started_at: now,
            requests: 0,
        });
        let elapsed = now.saturating_duration_since(window.started_at);
        if elapsed >= self.window_length {
            window.started_at = now;
            window.requests = 0;
        }

        let allowed = window.requests < self.limit;
        if allowed {
            window.requests += 1;
        }
        let retry_after_seconds = if allowed {
            0
        } else {
            self.window_length.saturating_sub(elapsed).as_secs().max(1)
        };

        LimitDecision {
            allowed,
            limit: self.limit,
            remaining: self.limit.saturating_sub(window.requests),
            retry_after_seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::*;

    const CLIENT: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

    #[test]
    fn rejects_requests_after_limit() {
        let limiter = RateLimiter::per_minute(2);
        let now = Instant::now();

        assert!(limiter.check_at(CLIENT, now).allowed);
        assert!(limiter.check_at(CLIENT, now).allowed);
        let rejected = limiter.check_at(CLIENT, now);

        assert!(!rejected.allowed);
        assert_eq!(rejected.remaining, 0);
        assert_eq!(rejected.retry_after_seconds, 60);
    }

    #[test]
    fn starts_a_new_window_after_one_minute() {
        let limiter = RateLimiter::per_minute(1);
        let now = Instant::now();

        assert!(limiter.check_at(CLIENT, now).allowed);
        assert!(!limiter.check_at(CLIENT, now).allowed);
        assert!(
            limiter
                .check_at(CLIENT, now + Duration::from_secs(60))
                .allowed
        );
    }

    #[test]
    fn zero_disables_rate_limiting() {
        let limiter = RateLimiter::per_minute(0);
        let decision = limiter.check_at(CLIENT, Instant::now());

        assert!(decision.allowed);
        assert_eq!(decision.limit, 0);
    }
}
