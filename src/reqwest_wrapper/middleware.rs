//! Rate limiting middleware for [crate::Client]

pub trait ResponseMiddleware {
    type Error;

    fn on_response(
        &self,
        response: reqwest::Result<reqwest::Response>,
    ) -> Result<reqwest::Response, Self::Error>;
}

#[derive(Clone)]
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
