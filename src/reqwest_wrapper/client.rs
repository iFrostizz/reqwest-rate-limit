//! Wrapper client for [reqwest::Client]

use crate::{RequestBuilder, reqwest_wrapper::middleware::NoopResponseMiddleware};

#[derive(Debug, Clone)]
pub struct Client {
    inner: reqwest::Client,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn get<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<NoopResponseMiddleware> {
        let inner = self.inner.get(url);
        RequestBuilder {
            client: self.clone(),
            inner,
            response_middleware: NoopResponseMiddleware,
            rate_limiter: None,
        }
    }

    pub fn post<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<NoopResponseMiddleware> {
        let inner = self.inner.post(url);
        RequestBuilder {
            client: self.clone(),
            inner,
            response_middleware: NoopResponseMiddleware,
            rate_limiter: None,
        }
    }

    pub fn put<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<NoopResponseMiddleware> {
        let inner = self.inner.put(url);
        RequestBuilder {
            client: self.clone(),
            inner,
            response_middleware: NoopResponseMiddleware,
            rate_limiter: None,
        }
    }

    pub fn patch<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<NoopResponseMiddleware> {
        let inner = self.inner.patch(url);
        RequestBuilder {
            client: self.clone(),
            inner,
            response_middleware: NoopResponseMiddleware,
            rate_limiter: None,
        }
    }

    pub fn delete<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<NoopResponseMiddleware> {
        let inner = self.inner.delete(url);
        RequestBuilder {
            client: self.clone(),
            inner,
            response_middleware: NoopResponseMiddleware,
            rate_limiter: None,
        }
    }

    pub fn head<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<NoopResponseMiddleware> {
        let inner = self.inner.head(url);
        RequestBuilder {
            client: self.clone(),
            inner,
            response_middleware: NoopResponseMiddleware,
            rate_limiter: None,
        }
    }

    pub fn request<U: reqwest::IntoUrl>(
        &self,
        method: reqwest::Method,
        url: U,
    ) -> RequestBuilder<NoopResponseMiddleware> {
        let inner = self.inner.request(method, url);
        RequestBuilder {
            client: self.clone(),
            inner,
            response_middleware: NoopResponseMiddleware,
            rate_limiter: None,
        }
    }

    pub async fn execute(&self, request: reqwest::Request) -> reqwest::Result<reqwest::Response> {
        self.inner.execute(request).await
    }
}

impl Client {
    pub(crate) fn inner(&self) -> &reqwest::Client {
        &self.inner
    }
}

pub struct ClientBuilder {}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn attach_client(&self, client: reqwest::Client) -> Client {
        Client { inner: client }
    }
}
