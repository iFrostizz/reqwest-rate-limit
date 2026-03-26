use crate::{NoopResponseMiddleware, RequestBuilder, ResponseMiddleware};

#[derive(Debug, Clone)]
pub struct Client<MW = NoopResponseMiddleware> {
    inner: reqwest::Client,
    response_middleware: MW,
}

impl Client {
    pub fn builder() -> ClientBuilder<NoopResponseMiddleware> {
        ClientBuilder::new()
    }
}

impl<MW> Client<MW>
where
    MW: ResponseMiddleware + Clone,
{
    pub fn get<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<MW> {
        let inner = self.inner.get(url);
        RequestBuilder::from_parts(self.clone(), inner)
    }

    pub fn post<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<MW> {
        let inner = self.inner.post(url);
        RequestBuilder::from_parts(self.clone(), inner)
    }

    pub fn put<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<MW> {
        let inner = self.inner.put(url);
        RequestBuilder::from_parts(self.clone(), inner)
    }

    pub fn patch<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<MW> {
        let inner = self.inner.patch(url);
        RequestBuilder::from_parts(self.clone(), inner)
    }

    pub fn delete<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<MW> {
        let inner = self.inner.delete(url);
        RequestBuilder::from_parts(self.clone(), inner)
    }

    pub fn head<U: reqwest::IntoUrl>(&self, url: U) -> RequestBuilder<MW> {
        let inner = self.inner.head(url);
        RequestBuilder::from_parts(self.clone(), inner)
    }

    pub fn request<U: reqwest::IntoUrl>(
        &self,
        method: reqwest::Method,
        url: U,
    ) -> RequestBuilder<MW> {
        let inner = self.inner.request(method, url);
        RequestBuilder::from_parts(self.clone(), inner)
    }

    pub(crate) fn middleware(&self) -> &MW {
        &self.response_middleware
    }
}

pub struct ClientBuilder<MW> {
    inner: reqwest::ClientBuilder,
    response_middleware: MW,
}

impl Default for ClientBuilder<NoopResponseMiddleware> {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder<NoopResponseMiddleware> {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::builder(),
            response_middleware: NoopResponseMiddleware,
        }
    }
}

impl<MW> ClientBuilder<MW>
where
    MW: ResponseMiddleware + Clone,
{
    pub fn response_middleware<NewMW>(self, response_middleware: NewMW) -> ClientBuilder<NewMW>
    where
        NewMW: ResponseMiddleware + Clone,
    {
        ClientBuilder {
            inner: self.inner,
            response_middleware,
        }
    }

    pub fn user_agent<V>(self, value: V) -> Self
    where
        V: TryInto<reqwest::header::HeaderValue>,
        V::Error: Into<http::Error>,
    {
        Self {
            inner: self.inner.user_agent(value),
            response_middleware: self.response_middleware,
        }
    }

    pub fn default_headers(self, headers: reqwest::header::HeaderMap) -> Self {
        Self {
            inner: self.inner.default_headers(headers),
            response_middleware: self.response_middleware,
        }
    }

    pub fn timeout(self, timeout: std::time::Duration) -> Self {
        Self {
            inner: self.inner.timeout(timeout),
            response_middleware: self.response_middleware,
        }
    }

    pub fn build(self) -> Result<Client<MW>, reqwest::Error> {
        let inner = self.inner.build()?;
        Ok(Client {
            inner,
            response_middleware: self.response_middleware,
        })
    }

    pub fn attach_client(self, client: reqwest::Client) -> Client<MW> {
        Client {
            inner: client,
            response_middleware: self.response_middleware,
        }
    }
}
