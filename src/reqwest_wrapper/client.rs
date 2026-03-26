//! Wrapper client for [reqwest::Client]

#[derive(Debug, Clone)]
pub struct Client {
    inner: reqwest::Client,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn get<U: reqwest::IntoUrl>(&self, url: U) -> crate::RequestBuilder {
        let inner = self.inner.get(url);
        crate::RequestBuilder {
            client: self.clone(),
            inner,
        }
    }

    pub fn post<U: reqwest::IntoUrl>(&self, url: U) -> crate::RequestBuilder {
        let inner = self.inner.post(url);
        crate::RequestBuilder {
            client: self.clone(),
            inner,
        }
    }

    pub fn put<U: reqwest::IntoUrl>(&self, url: U) -> crate::RequestBuilder {
        let inner = self.inner.put(url);
        crate::RequestBuilder {
            client: self.clone(),
            inner,
        }
    }

    pub fn patch<U: reqwest::IntoUrl>(&self, url: U) -> crate::RequestBuilder {
        let inner = self.inner.patch(url);
        crate::RequestBuilder {
            client: self.clone(),
            inner,
        }
    }

    pub fn delete<U: reqwest::IntoUrl>(&self, url: U) -> crate::RequestBuilder {
        let inner = self.inner.delete(url);
        crate::RequestBuilder {
            client: self.clone(),
            inner,
        }
    }

    pub fn head<U: reqwest::IntoUrl>(&self, url: U) -> crate::RequestBuilder {
        let inner = self.inner.head(url);
        crate::RequestBuilder {
            client: self.clone(),
            inner,
        }
    }

    pub fn request<U: reqwest::IntoUrl>(
        &self,
        method: reqwest::Method,
        url: U,
    ) -> crate::RequestBuilder {
        let inner = self.inner.request(method, url);
        crate::RequestBuilder {
            client: self.clone(),
            inner,
        }
    }

    pub async fn execute(&self, request: reqwest::Request) -> crate::Result<crate::Response> {
        self.inner
            .execute(request)
            .await
            .map_err(crate::Error)
            .map(crate::Response)
    }
}

impl Client {
    pub(crate) fn inner(&self) -> &reqwest::Client {
        &self.inner
    }
}

pub struct ClientBuilder {}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn attach_client(&self, client: reqwest::Client) -> Client {
        todo!()
    }
}
