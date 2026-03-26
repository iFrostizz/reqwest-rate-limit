//! Rate limiting middleware for [crate::Client]

pub trait Middleware {
    fn get<U: reqwest::IntoUrl>(&self, request: reqwest::Request) -> reqwest::RequestBuilder;
    fn post<U: reqwest::IntoUrl>(&self, request: reqwest::Request) -> reqwest::RequestBuilder;
    fn put<U: reqwest::IntoUrl>(&self, request: reqwest::Request) -> reqwest::RequestBuilder;
    fn patch<U: reqwest::IntoUrl>(&self, request: reqwest::Request) -> reqwest::RequestBuilder;
    fn delete<U: reqwest::IntoUrl>(&self, request: reqwest::Request) -> reqwest::RequestBuilder;
    fn head<U: reqwest::IntoUrl>(&self, request: reqwest::Request) -> reqwest::RequestBuilder;
    fn request<U: reqwest::IntoUrl>(&self, request: reqwest::Request) -> reqwest::RequestBuilder;
    fn execute(
        &self,
        request: reqwest::Request,
    ) -> impl Future<Output = reqwest::Result<reqwest::Response>>;
}
