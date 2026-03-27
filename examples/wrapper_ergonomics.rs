use governor::Quota;
use http::{HeaderMap, HeaderValue};
use std::num::NonZeroU32;
use std::sync::Arc;

fn main() {
    // Primary rate limit for authenticated users is 5,000 requests per hour.
    let rate_limiter = Arc::new(governor::RateLimiter::direct(Quota::per_hour(
        NonZeroU32::new(5_000).unwrap(),
    )));

    let mut default_headers = HeaderMap::new();
    default_headers.insert("accept", HeaderValue::from_static("application/json"));

    let client = reqwest_rate_limit::Client::builder()
        .user_agent("reqwest-rate-limit-wrapper-ergonomics")
        .default_headers(default_headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();

    let _request = client
        .get("https://api.website.com/rate_limit")
        .header("x-demo", "wrapper-ergonomics")
        .bearer_auth("REDACTED_TOKEN")
        .with_rate_limiter(rate_limiter.clone())
        .send();

    let _create = client
        .post("https://api.website.com/files")
        .header("content-type", "application/json")
        .body("{\"description\":\"example\",\"public\":false,\"files\":{}}")
        .with_rate_limiter(rate_limiter.clone())
        .send();

    let _update = client
        .put("https://api.website.com/files/ID")
        .header("content-type", "application/json")
        .body("{\"description\":\"updated\"}")
        .with_rate_limiter(rate_limiter.clone())
        .send();

    let _patch = client
        .patch("https://api.website.com/files/ID")
        .header("content-type", "application/json")
        .body("{\"description\":\"patched\"}")
        .with_rate_limiter(rate_limiter.clone())
        .send();

    let _delete = client
        .delete("https://api.website.com/files/ID")
        .with_rate_limiter(rate_limiter.clone())
        .send();

    let _head = client
        .head("https://api.website.com/rate_limit")
        .with_rate_limiter(rate_limiter)
        .send();
}
