pub mod reqwest_wrapper;

pub use governor;
pub use reqwest_wrapper::{Client, ClientBuilder, RequestBuilder};

/// Intercept a response to apply rate-limit aware behavior.
pub trait ResponseMiddleware {
    type Error;

    fn on_response(
        &self,
        response: reqwest::Result<reqwest::Response>,
    ) -> Result<reqwest::Response, Self::Error>;
}

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
pub async fn send_with_rate_limiter(
    request: reqwest::RequestBuilder,
    rate_limiter: &governor::DefaultDirectRateLimiter,
) -> reqwest::Result<reqwest::Response> {
    rate_limiter.until_ready().await;
    request.send().await
}

/// Send a request through a response middleware after rate limiting.
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
