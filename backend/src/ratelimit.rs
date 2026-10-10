//! Request rate limits (D6, M4 decision 6): auth routes per client IP, `/api/data/*` per user.

use axum::{
    extract::{ConnectInfo, FromRequestParts, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};
use std::{
    hash::Hash,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use uuid::Uuid;

use crate::{auth::UserCtx, config::RateLimit, error::AppError, AppState};

/// How often idle limiter keys are dropped so the key maps stay bounded.
const CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct RateLimiters {
    auth: Arc<DefaultKeyedRateLimiter<IpAddr>>,
    data: Arc<DefaultKeyedRateLimiter<Uuid>>,
}

fn quota(limit: RateLimit) -> Quota {
    Quota::per_minute(limit.per_minute).allow_burst(limit.burst)
}

impl RateLimiters {
    pub fn new(auth: RateLimit, data: RateLimit) -> Self {
        Self {
            auth: Arc::new(RateLimiter::keyed(quota(auth))),
            data: Arc::new(RateLimiter::keyed(quota(data))),
        }
    }

    /// Periodically forget keys whose limit has fully replenished. The task ends with the runtime.
    pub fn spawn_cleanup(&self) {
        let limiters = self.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(CLEANUP_INTERVAL);
            loop {
                tick.tick().await;
                limiters.cleanup();
            }
        });
    }

    fn cleanup(&self) {
        prune(&self.auth);
        prune(&self.data);
    }
}

fn prune<K: Hash + Eq + Clone>(limiter: &DefaultKeyedRateLimiter<K>) {
    limiter.retain_recent();
    limiter.shrink_to_fit();
}

/// Register and login: keyed by the peer IP from `ConnectInfo` (the server runs with
/// `into_make_service_with_connect_info`). A request without it shares one bucket.
pub async fn limit_auth(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    if state.limiters.auth.check_key(&ip).is_err() {
        tracing::warn!(%ip, "auth rate limit exceeded");
        return AppError::RateLimited.into_response();
    }
    next.run(req).await
}

/// `/api/data/*`: authenticates first (401 before 429), then limits per user id. The resolved
/// `UserCtx` is stored in the request so the handler does not decode the token again.
pub async fn limit_data(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();
    let user = match UserCtx::from_request_parts(&mut parts, &state).await {
        Ok(user) => user,
        Err(e) => return e.into_response(),
    };
    if state.limiters.data.check_key(&user.user_id).is_err() {
        tracing::warn!(user_id = %user.user_id, "data rate limit exceeded");
        return AppError::RateLimited.into_response();
    }
    next.run(Request::from_parts(parts, body)).await
}

#[cfg(test)]
mod tests {
    use super::RateLimiters;
    use crate::config::RateLimit;
    use std::net::IpAddr;
    use std::num::NonZeroU32;

    #[test]
    fn burst_then_limit_per_key_and_cleanup_keeps_working() {
        let one = NonZeroU32::new(1).unwrap();
        let two = NonZeroU32::new(2).unwrap();
        let limiters = RateLimiters::new(
            RateLimit {
                per_minute: one,
                burst: two,
            },
            RateLimit {
                per_minute: one,
                burst: one,
            },
        );
        let a: IpAddr = [127, 0, 0, 1].into();
        let b: IpAddr = [127, 0, 0, 2].into();
        assert!(limiters.auth.check_key(&a).is_ok());
        assert!(limiters.auth.check_key(&a).is_ok());
        assert!(limiters.auth.check_key(&a).is_err());
        assert!(limiters.auth.check_key(&b).is_ok());
        limiters.cleanup();
        // Recently used keys survive cleanup, so the limit still holds.
        assert!(limiters.auth.check_key(&a).is_err());
        assert_eq!(limiters.auth.len(), 2);
    }
}
