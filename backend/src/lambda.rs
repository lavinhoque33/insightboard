//! AWS Lambda adapter (M6 decisions 3 and 6), compiled only with `--features lambda`.
//!
//! Function URL and API Gateway HTTP API (payload v2) events reach the same router as the
//! server. The per-IP auth rate limit keys on `ConnectInfo`, which Lambda has no socket for, so
//! it comes from `requestContext.http.sourceIp`: AWS sets that from the TCP connection; it is not
//! a header. Forwarding headers stay untrusted. Without a usable source address the extension is
//! left out and the limiter's shared-bucket fallback applies.

use axum::{
    extract::{ConnectInfo, Request},
    middleware, Router,
};
use lambda_http::{request::RequestContext, RequestExt};
use std::net::{IpAddr, SocketAddr};

/// `router` with `ConnectInfo<SocketAddr>` taken from the Lambda request context.
pub fn with_source_ip(router: Router) -> Router {
    router.layer(middleware::map_request(insert_source_ip))
}

async fn insert_source_ip(mut req: Request) -> Request {
    if let Some(ip) = source_ip(&req) {
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::new(ip, 0)));
    }
    req
}

fn source_ip(req: &Request) -> Option<IpAddr> {
    match req.request_context_ref()? {
        RequestContext::ApiGatewayV2(context) => context.http.source_ip.as_deref()?.parse().ok(),
        // Other event sources are not compiled in (`apigw_http` only); the enum is non-exhaustive.
        _ => None,
    }
}

/// Serve `router` through the Lambda runtime until the execution environment shuts down.
pub async fn run(router: Router) -> Result<(), lambda_http::Error> {
    lambda_http::run(with_source_ip(router)).await
}
