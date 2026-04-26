use governor::Quota;
use std::num::NonZeroU32;

fn main() {
    // Simple primary rate limiter: 5,000 requests per hour.
    let rate_limiter =
        governor::RateLimiter::direct(Quota::per_hour(NonZeroU32::new(5_000).unwrap()));

    let reqwest_client = reqwest::Client::new();
    let request = reqwest_client.get("https://api.website.com/");
    let _send = reqwest::send_with_rate_limiter(request, &rate_limiter);
}
