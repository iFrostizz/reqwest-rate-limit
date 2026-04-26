use crate::{ResponseMiddleware, reqwest_wrapper::client::Client};
use governor::Jitter;
use std::sync::Arc;
use std::time::Duration;

/// Wrapper around `reqwest::RequestBuilder` that applies rate limiting and middleware.
#[derive(Debug)]
pub struct RequestBuilder<MW>
where
    MW: ResponseMiddleware + Clone,
{
    pub(crate) client: Client<MW>,
    pub(crate) inner: reqwest::RequestBuilder,
    pub(crate) response_middleware: MW,
    pub(crate) rate_limiter: Option<Arc<governor::DefaultDirectRateLimiter>>,
}

impl<MW> RequestBuilder<MW>
where
    MW: ResponseMiddleware + Clone,
{
    pub(crate) fn from_parts(client: Client<MW>, inner: reqwest::RequestBuilder) -> Self {
        let response_middleware = client.middleware().clone();
        let rate_limiter = client.rate_limiter();
        Self {
            client,
            inner,
            response_middleware,
            rate_limiter,
        }
    }

    /// Override the rate limiter for this request with a shared limiter.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use governor::Quota;
    /// use std::num::NonZeroU32;
    /// use std::sync::Arc;
    ///
    /// let limiter = Arc::new(governor::RateLimiter::direct(Quota::per_second(
    ///     NonZeroU32::new(10).unwrap(),
    /// )));
    ///
    /// let client = reqwest::Client::builder().build().unwrap();
    /// let _request = client
    ///     .get("https://api.example.com/health")
    ///     .with_rate_limiter(limiter);
    /// ```
    pub fn with_rate_limiter(self, rate_limiter: Arc<governor::DefaultDirectRateLimiter>) -> Self {
        Self {
            rate_limiter: Some(rate_limiter),
            ..self
        }
    }

    /// Override the rate limiter for this request with an owned limiter.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use governor::Quota;
    /// use std::num::NonZeroU32;
    ///
    /// let limiter = governor::RateLimiter::direct(Quota::per_second(
    ///     NonZeroU32::new(10).unwrap(),
    /// ));
    ///
    /// let client = reqwest::Client::builder().build().unwrap();
    /// let _request = client
    ///     .get("https://api.example.com/health")
    ///     .with_rate_limiter_owned(limiter);
    /// ```
    pub fn with_rate_limiter_owned(self, rate_limiter: governor::DefaultDirectRateLimiter) -> Self {
        Self {
            rate_limiter: Some(Arc::new(rate_limiter)),
            ..self
        }
    }

    /// Add a header to this request.
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

    /// Replace all headers on this request.
    pub fn headers(self, headers: http::HeaderMap) -> Self {
        let inner = self.inner.headers(headers);
        Self { inner, ..self }
    }

    /// Add basic authentication to this request.
    pub fn basic_auth<U, P>(self, username: U, password: Option<P>) -> Self
    where
        U: std::fmt::Display,
        P: std::fmt::Display,
    {
        let inner = self.inner.basic_auth(username, password);
        Self { inner, ..self }
    }

    /// Add bearer authentication to this request.
    pub fn bearer_auth<T>(self, token: T) -> Self
    where
        T: std::fmt::Display,
    {
        let inner = self.inner.bearer_auth(token);
        Self { inner, ..self }
    }

    /// Set the body of this request.
    pub fn body<T: Into<reqwest::Body>>(self, body: T) -> Self {
        let inner = self.inner.body(body);
        Self { inner, ..self }
    }

    /// Override the timeout for this request.
    pub fn timeout(self, timeout: Duration) -> Self {
        let inner = self.inner.timeout(timeout);
        Self { inner, ..self }
    }

    #[cfg(feature = "multipart")]
    /// Set a multipart body for this request.
    pub fn multipart(self, multipart: reqwest::multipart::Form) -> Self {
        let inner = self.inner.multipart(multipart);
        Self { inner, ..self }
    }

    #[cfg(feature = "query")]
    /// Set the query string for this request.
    pub fn query<T: serde::Serialize + ?Sized>(self, query: &T) -> Self {
        let inner = self.inner.query(query);
        Self { inner, ..self }
    }

    /// Override the HTTP version used for this request.
    pub fn version(self, version: reqwest::Version) -> Self {
        let inner = self.inner.version(version);
        Self { inner, ..self }
    }

    #[cfg(feature = "form")]
    /// Set a form body for this request.
    pub fn form<T: serde::Serialize + ?Sized>(self, form: &T) -> Self {
        let inner = self.inner.form(form);
        Self { inner, ..self }
    }

    #[cfg(feature = "json")]
    /// Set a JSON body for this request.
    pub fn json<T: serde::Serialize + ?Sized>(self, json: &T) -> Self {
        let inner = self.inner.json(json);
        Self { inner, ..self }
    }

    /// Build the underlying `reqwest::Request`.
    pub fn build(self) -> reqwest::Result<reqwest::Request> {
        self.inner.build()
    }

    /// Send the request, applying the rate limiter and middleware.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), reqwest::Error> {
    /// let client = reqwest::Client::builder().build().unwrap();
    /// let _response = client.get("https://api.example.com/health").send().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send(self) -> Result<reqwest::Response, MW::Error> {
        if let Some(rate_limiter) = &self.rate_limiter {
            // TODO configurable
            rate_limiter
                .until_ready_with_jitter(Jitter::up_to(Duration::from_millis(50)))
                .await;
        }
        let res = self.inner.send().await;
        self.response_middleware.on_response(res).await
    }

    /// Attempt to clone this builder for re-use.
    pub fn try_clone(&self) -> Option<Self> {
        let inner = self.inner.try_clone()?;
        Some(Self {
            client: self.client.clone(),
            inner,
            response_middleware: self.response_middleware.clone(),
            rate_limiter: None, // rate limiters aren't cloneable
        })
    }
}
