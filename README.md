# reqwest-rate-limit

Rate-limit helpers and optional wrapper ergonomics for `reqwest`.

## Highlights

- Small helpers to rate-limit any `reqwest::RequestBuilder`
- Optional wrapper `Client` with middleware hooks
- `ClientBuilder::configure` exposes all `reqwest::ClientBuilder` options

## Example

```rust
use governor::Quota;
use std::num::NonZeroU32;
use std::sync::Arc;

let rate_limiter = Arc::new(governor::RateLimiter::direct(Quota::per_hour(
    NonZeroU32::new(5_000).unwrap(),
)));

let client = reqwest_rate_limit::Client::builder()
    .user_agent("reqwest-rate-limit-readme")
    .configure(|b| b.timeout(std::time::Duration::from_secs(10)))
    .build()
    .unwrap();

let _request = client
    .get("https://api.example.com/health")
    .with_rate_limiter(rate_limiter)
    .send();
```

## Examples

- [examples/simple_rate_limit.rs](examples/simple_rate_limit.rs): Minimal rate limiting using a plain `reqwest::Client`.
- [examples/wrapper_ergonomics.rs](examples/wrapper_ergonomics.rs): Wrapper client ergonomics with builder configuration and per-request rate limits.
- [examples/github_rest_api.rs](examples/github_rest_api.rs): GitHub REST API rate-limit handling with a `ResponseMiddleware` implementation.
