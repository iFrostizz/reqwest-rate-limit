//! Rate limiting helpers and optional wrapper ergonomics for `reqwest`.
//!
//! This crate keeps `reqwest` at the center while adding helpers to apply rate limits
//! and, optionally, a small wrapper client with middleware hooks.
//!
//! # Example
//!
//! ```no_run
//! use governor::Quota;
//! use std::num::NonZeroU32;
//! use std::sync::Arc;
//!
//! let rate_limiter = Arc::new(governor::RateLimiter::direct(Quota::per_hour(
//!     NonZeroU32::new(5_000).unwrap(),
//! )));
//!
//! let client = reqwest_rate_limit::Client::builder()
//!     .user_agent("reqwest-rate-limit-docs")
//!     .configure(|b| b.timeout(std::time::Duration::from_secs(30)))
//!     .build()
//!     .unwrap();
//!
//! let _future = client
//!     .get("https://api.example.com/v1/health")
//!     .with_rate_limiter(rate_limiter)
//!     .send();
//! ```
//!
//! # Why `configure`?
//!
//! `ClientBuilder::configure` gives you access to the full surface of
//! `reqwest::ClientBuilder` without this crate having to mirror every method.
//! That means you can use all of reqwest's options and still keep the wrapper
//! ergonomics and middleware hooks.
//!
//! ```no_run
//! # use governor::Quota;
//! # use std::num::NonZeroU32;
//! # use std::sync::Arc;
//! # let rate_limiter = Arc::new(governor::RateLimiter::direct(Quota::per_hour(
//! #     NonZeroU32::new(5_000).unwrap(),
//! # )));
//! let client = reqwest_rate_limit::Client::builder()
//!     .configure(|b| {
//!         b.timeout(std::time::Duration::from_secs(10))
//!          .pool_max_idle_per_host(8)
//!          .https_only(true)
//!     })
//!     .rate_limiter(rate_limiter)
//!     .build()
//!     .unwrap();
//! ```
//!
//! If you do not want the wrapper, use `send_with_rate_limiter` with a plain
//! `reqwest::Client` instead.
//!
//! # ResponseMiddleware
//!
//! Implement `ResponseMiddleware` to inspect responses and apply rate-limit rules.
//! The [GitHub REST API example](../examples/github_rest_api.rs) shows how to translate
//! `retry-after` headers into concrete waits and backoff behavior.
//!
//! # Features
//!
//! This crate forwards optional `reqwest` features:
//! `json`, `form`, `query`, and `multipart`.
pub mod reqwest_wrapper;

/// Re-exported for convenience when constructing rate limiters.
pub use governor;
/// Wrapper client that layers rate limiting and middleware hooks over `reqwest`.
pub use reqwest_wrapper::{Client, ClientBuilder, RequestBuilder};

/// Intercept a response to apply rate-limit aware behavior.
pub trait ResponseMiddleware {
    type Error;

    /// Inspect or transform the `reqwest` response.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// struct Passthrough;
    ///
    /// impl reqwest_rate_limit::ResponseMiddleware for Passthrough {
    ///     type Error = reqwest::Error;
    ///
    ///     fn on_response(
    ///         &self,
    ///         response: reqwest::Result<reqwest::Response>,
    ///     ) -> Result<reqwest::Response, Self::Error> {
    ///         response
    ///     }
    /// }
    /// ```
    fn on_response(
        &self,
        response: reqwest::Result<reqwest::Response>,
    ) -> Result<reqwest::Response, Self::Error>;
}

/// Default middleware that returns the response unchanged.
#[derive(Clone, Default)]
pub struct NoopResponseMiddleware;

impl ResponseMiddleware for NoopResponseMiddleware {
    type Error = reqwest::Error;

    fn on_response(
        &self,
        response: reqwest::Result<reqwest::Response>,
    ) -> Result<reqwest::Response, Self::Error> {
        response
    }
}

/// Send a request after waiting for the rate limiter to allow it.
///
/// # Examples
///
/// ```no_run
/// use governor::{Quota, RateLimiter};
/// use std::num::NonZeroU32;
///
/// # async fn example() -> reqwest::Result<()> {
/// let client = reqwest::Client::new();
/// let request = client.get("https://api.example.com/health");
/// let limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(5).unwrap()));
/// let _response = reqwest_rate_limit::send_with_rate_limiter(request, &limiter).await?;
/// # Ok(())
/// # }
/// ```
pub async fn send_with_rate_limiter(
    request: reqwest::RequestBuilder,
    rate_limiter: &governor::DefaultDirectRateLimiter,
) -> reqwest::Result<reqwest::Response> {
    rate_limiter.until_ready().await;
    request.send().await
}

/// Send a request through a response middleware after rate limiting.
///
/// # Examples
///
/// ```no_run
/// use governor::{Quota, RateLimiter};
/// use std::num::NonZeroU32;
///
/// struct Passthrough;
///
/// impl reqwest_rate_limit::ResponseMiddleware for Passthrough {
///     type Error = reqwest::Error;
///
///     fn on_response(
///         &self,
///         response: reqwest::Result<reqwest::Response>,
///     ) -> Result<reqwest::Response, Self::Error> {
///         response
///     }
/// }
///
/// # async fn example() -> Result<(), reqwest::Error> {
/// let client = reqwest::Client::new();
/// let request = client.get("https://api.example.com/health");
/// let limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(5).unwrap()));
/// let middleware = Passthrough;
/// let _response =
///     reqwest_rate_limit::send_with_rate_limiter_and_middleware(request, &limiter, &middleware)
///         .await?;
/// # Ok(())
/// # }
/// ```
pub async fn send_with_rate_limiter_and_middleware<MW>(
    request: reqwest::RequestBuilder,
    rate_limiter: &governor::DefaultDirectRateLimiter,
    middleware: &MW,
) -> Result<reqwest::Response, MW::Error>
where
    MW: ResponseMiddleware,
{
    let response = send_with_rate_limiter(request, rate_limiter).await;
    middleware.on_response(response)
}
