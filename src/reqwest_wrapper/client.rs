use crate::{NoopReqwestMiddleware, RequestBuilder, ReqwestMiddleware};
use std::sync::Arc;

/// Wrapper client that exposes ergonomic request builders with rate limiting.
#[derive(Debug, Clone)]
pub struct Client<MW = NoopReqwestMiddleware> {
    inner: reqwest::Client,
    middleware: MW,
    rate_limiter: Option<Arc<governor::DefaultDirectRateLimiter>>,
}

impl Client {
    /// Start building a wrapper client with default middleware.
    pub fn builder() -> ClientBuilder<NoopReqwestMiddleware> {
        ClientBuilder::new()
    }
}

impl<MW> Client<MW>
where
    MW: ReqwestMiddleware + Clone,
{
    /// Begin a GET request.
    pub fn get<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<MW> {
        let inner = self.inner.get(url);
        RequestBuilder::from_parts(self.clone(), inner)
    }

    /// Begin a POST request.
    pub fn post<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<MW> {
        let inner = self.inner.post(url);
        RequestBuilder::from_parts(self.clone(), inner)
    }

    /// Begin a PUT request.
    pub fn put<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<MW> {
        let inner = self.inner.put(url);
        RequestBuilder::from_parts(self.clone(), inner)
    }

    /// Begin a PATCH request.
    pub fn patch<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<MW> {
        let inner = self.inner.patch(url);
        RequestBuilder::from_parts(self.clone(), inner)
    }

    /// Begin a DELETE request.
    pub fn delete<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<MW> {
        let inner = self.inner.delete(url);
        RequestBuilder::from_parts(self.clone(), inner)
    }

    /// Begin a HEAD request.
    pub fn head<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<MW> {
        let inner = self.inner.head(url);
        RequestBuilder::from_parts(self.clone(), inner)
    }

    /// Begin a request with an explicit HTTP method.
    pub fn request<U: reqwest::IntoUrl>(
        &self,
        method: reqwest::Method,
        url: U,
    ) -> RequestBuilder<MW> {
        let inner = self.inner.request(method, url);
        RequestBuilder::from_parts(self.clone(), inner)
    }

    pub(crate) fn middleware(&self) -> &MW {
        &self.middleware
    }

    pub(crate) fn rate_limiter(&self) -> Option<Arc<governor::DefaultDirectRateLimiter>> {
        self.rate_limiter.clone()
    }
}

/// Builder for the wrapper client.
pub struct ClientBuilder<MW> {
    inner: reqwest::ClientBuilder,
    middleware: MW,
    rate_limiter: Option<Arc<governor::DefaultDirectRateLimiter>>,
}

impl Default for ClientBuilder<NoopReqwestMiddleware> {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder<NoopReqwestMiddleware> {
    /// Create a new builder with default middleware and no rate limiter.
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::builder(),
            middleware: NoopReqwestMiddleware,
            rate_limiter: None,
        }
    }
}

impl<MW> ClientBuilder<MW>
where
    MW: ReqwestMiddleware + Clone,
{
    /// Wrap an existing `reqwest::ClientBuilder`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let base = reqwest::Client::builder().https_only(true);
    /// let client = reqwest_rate_limit::ClientBuilder::from_reqwest_builder(
    ///     base,
    ///     reqwest_rate_limit::NoopReqwestMiddleware,
    /// )
    /// .build()
    /// .unwrap();
    /// ```
    pub fn from_reqwest_builder(inner: reqwest::ClientBuilder, middleware: MW) -> Self {
        Self {
            inner,
            middleware,
            rate_limiter: None,
        }
    }

    /// Configure the underlying `reqwest::ClientBuilder`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let client = reqwest_rate_limit::Client::builder()
    ///     .configure(|b| b.timeout(std::time::Duration::from_secs(5)))
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn configure<F>(self, f: F) -> Self
    where
        F: FnOnce(reqwest::ClientBuilder) -> reqwest::ClientBuilder,
    {
        Self {
            inner: f(self.inner),
            middleware: self.middleware,
            rate_limiter: self.rate_limiter,
        }
    }

    /// Replace the request middleware for this client.
    pub fn middleware<NewMW>(self, middleware: NewMW) -> ClientBuilder<NewMW>
    where
        NewMW: ReqwestMiddleware + Clone,
    {
        ClientBuilder {
            inner: self.inner,
            middleware,
            rate_limiter: self.rate_limiter,
        }
    }

    /// Attach a shared rate limiter used by default for requests.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use governor::Quota;
    /// use std::num::NonZeroU32;
    /// use std::sync::Arc;
    ///
    /// let limiter = Arc::new(governor::RateLimiter::direct(Quota::per_minute(
    ///     NonZeroU32::new(60).unwrap(),
    /// )));
    ///
    /// let client = reqwest_rate_limit::Client::builder()
    ///     .rate_limiter(limiter)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn rate_limiter(
        self,
        rate_limiter: Arc<governor::DefaultDirectRateLimiter>,
    ) -> ClientBuilder<MW> {
        ClientBuilder {
            inner: self.inner,
            middleware: self.middleware,
            rate_limiter: Some(rate_limiter),
        }
    }

    /// Set the User-Agent header for all requests.
    pub fn user_agent<V>(self, value: V) -> Self
    where
        V: TryInto<reqwest::header::HeaderValue>,
        V::Error: Into<http::Error>,
    {
        Self {
            inner: self.inner.user_agent(value),
            middleware: self.middleware,
            rate_limiter: self.rate_limiter,
        }
    }

    /// Set default headers for all requests.
    pub fn default_headers(self, headers: reqwest::header::HeaderMap) -> Self {
        Self {
            inner: self.inner.default_headers(headers),
            middleware: self.middleware,
            rate_limiter: self.rate_limiter,
        }
    }

    /// Set a default request timeout.
    pub fn timeout(self, timeout: std::time::Duration) -> Self {
        Self {
            inner: self.inner.timeout(timeout),
            middleware: self.middleware,
            rate_limiter: self.rate_limiter,
        }
    }

    /// Build the wrapper client.
    pub fn build(self) -> Result<Client<MW>, reqwest::Error> {
        let inner = self.inner.build()?;
        Ok(Client {
            inner,
            middleware: self.middleware,
            rate_limiter: self.rate_limiter,
        })
    }

    /// Attach an existing `reqwest::Client`.
    pub fn attach_client(self, client: reqwest::Client) -> Client<MW> {
        Client {
            inner: client,
            middleware: self.middleware,
            rate_limiter: self.rate_limiter,
        }
    }
}
