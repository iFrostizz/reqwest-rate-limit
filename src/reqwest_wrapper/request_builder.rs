use crate::reqwest_wrapper::middleware::{NoopResponseMiddleware, ResponseMiddleware};
use governor::Jitter;
use std::time::Duration;

#[derive(Debug)]
pub struct RequestBuilder<MW>
where
    MW: ResponseMiddleware + Clone,
    MW::Error: From<reqwest::Error>,
{
    pub(crate) client: crate::Client,
    pub(crate) inner: reqwest::RequestBuilder,
    pub(crate) response_middleware: MW,
    pub(crate) rate_limiter: Option<governor::DefaultDirectRateLimiter>,
}

impl RequestBuilder<NoopResponseMiddleware> {
    pub fn from_parts(client: crate::Client, request: reqwest::Request) -> Self {
        let inner = reqwest::RequestBuilder::from_parts(client.inner().clone(), request);
        Self {
            client,
            inner,
            response_middleware: NoopResponseMiddleware,
            rate_limiter: None,
        }
    }
}

impl<MW> RequestBuilder<MW>
where
    MW: ResponseMiddleware + Clone,
    MW::Error: From<reqwest::Error>,
{
    pub fn with_rate_limiter(self, rate_limiter: governor::DefaultDirectRateLimiter) -> Self {
        Self {
            rate_limiter: Some(rate_limiter),
            ..self
        }
    }

    pub fn header<KK, V>(self, key: KK, value: V) -> Self
    where
        http::HeaderName: TryFrom<KK>,
        <http::HeaderName as TryFrom<KK>>::Error: Into<http::Error>,
        http::HeaderValue: TryFrom<V>,
        <http::HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        let inner = self.inner.header(key, value);
        Self { inner, ..self }
    }

    pub fn headers(self, headers: http::HeaderMap) -> Self {
        let inner = self.inner.headers(headers);
        Self { inner, ..self }
    }

    pub fn basic_auth<U, P>(self, username: U, password: Option<P>) -> Self
    where
        U: std::fmt::Display,
        P: std::fmt::Display,
    {
        let inner = self.inner.basic_auth(username, password);
        Self { inner, ..self }
    }

    pub fn bearer_auth<T>(self, token: T) -> Self
    where
        T: std::fmt::Display,
    {
        let inner = self.inner.bearer_auth(token);
        Self { inner, ..self }
    }

    pub fn body<T: Into<reqwest::Body>>(self, body: T) -> Self {
        let inner = self.inner.body(body);
        Self { inner, ..self }
    }

    pub fn timeout(self, timeout: Duration) -> Self {
        let inner = self.inner.timeout(timeout);
        Self { inner, ..self }
    }

    #[cfg(feature = "multipart")]
    pub fn multipart(self, multipart: reqwest::multipart::Form) -> Self {
        let inner = self.inner.multipart(multipart);
        Self { inner, ..self }
    }

    #[cfg(feature = "query")]
    pub fn query<T: serde::Serialize + ?Sized>(self, query: &T) -> Self {
        let inner = self.inner.query(query);
        Self { inner, ..self }
    }

    pub fn version(self, version: reqwest::Version) -> Self {
        let inner = self.inner.version(version);
        Self { inner, ..self }
    }

    #[cfg(feature = "form")]
    pub fn form<T: serde::Serialize + ?Sized>(self, form: &T) -> Self {
        let inner = self.inner.form(form);
        Self { inner, ..self }
    }

    #[cfg(feature = "json")]
    pub fn json<T: serde::Serialize + ?Sized>(self, json: &T) -> Self {
        let inner = self.inner.json(json);
        Self { inner, ..self }
    }

    pub fn build(self) -> reqwest::Result<reqwest::Request> {
        self.inner.build()
    }

    pub fn build_split(self) -> (crate::Client, reqwest::Result<reqwest::Request>) {
        (self.client, self.inner.build())
    }

    pub async fn send(self) -> Result<reqwest::Response, MW::Error> {
        if let Some(rate_limiter) = &self.rate_limiter {
            rate_limiter
                .until_ready_with_jitter(Jitter::up_to(Duration::from_secs(20)))
                .await;
        }
        let res = self.inner.send().await;
        self.response_middleware.on_response(res)
    }

    pub fn try_clone(&self) -> Option<Self> {
        let inner = self.inner.try_clone()?;
        Some(Self {
            client: self.client.clone(),
            inner,
            // rate_limiter: self.rate_limiter.map(|r| r),
            response_middleware: self.response_middleware.clone(),
            rate_limiter: None, // TODO
        })
    }
}
