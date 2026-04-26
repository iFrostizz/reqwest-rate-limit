use reqwest_rate_limit::{ResponseMiddleware, governor::Quota};
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

// Track only the high-level state needed to choose the next wait strategy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RateLimitState {
    PrimaryRateLimit,
    // Secondary limits require progressive backoff across retries.
    SecondaryRateLimit { retries: u32 },
}

// Response middleware used to interpret GitHub's rate limit headers.
#[derive(Clone)]
pub struct RateLimitResponseMiddleware {
    state: Arc<Mutex<Option<RateLimitState>>>,
    // Safety valve to stop retry loops on persistent secondary limits.
    max_secondary_retries: u32,
}

#[derive(Debug)]
pub enum RateLimitError {
    Transport(reqwest::Error),
    /// You should not retry your request until after the time specified by the `x-ratelimit-reset` header.
    PrimaryRateLimit {
        x_ratelimit_reset: u64,
    },
    /// You should not retry your request until after `retry-after` seconds has elapsed.
    SecondaryRateLimitRetryAfter {
        retry_after: u64,
    },
    /// You should not retry your request until after the time specified by the `x-ratelimit-reset` header.
    SecondaryRateLimitReset {
        x_ratelimit_reset: u64,
    },
    /// Wait for at least one minute before retrying
    SecondaryRateLimitWait {
        retry_after: u64,
    },
    /// Wait for an exponentially increasing amount of time between retries,
    /// and throw an error after a specific number of retries.
    SecondaryRateLimitExponentialBackoff {
        retry_after: u64,
        attempt: u32,
    },
    /// Exceeded the max number of retries for secondary rate limits.
    SecondaryRateLimitExhausted {
        retries: u32,
    },
}

impl RateLimitResponseMiddleware {
    // GitHub recommends at least 60 seconds when secondary limits apply.
    const MIN_SECONDARY_WAIT_SECS: u64 = 60;
    // Avoid unbounded waits while still backing off aggressively.
    const MAX_SECONDARY_BACKOFF_SECS: u64 = 60 * 60;
    const DEFAULT_MAX_SECONDARY_RETRIES: u32 = 5;

    pub fn new(max_secondary_retries: u32) -> Self {
        Self {
            state: Arc::new(Mutex::new(None)),
            max_secondary_retries,
        }
    }

    // Parse numeric headers like x-ratelimit-reset/retry-after.
    fn parse_header_u64(headers: &http::HeaderMap, name: &str) -> Option<u64> {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
    }

    // Exponential backoff (bounded) for repeated secondary-limit failures.
    fn secondary_backoff_seconds(attempt: u32) -> u64 {
        let exp = attempt.saturating_sub(1).min(16);
        let backoff = Self::MIN_SECONDARY_WAIT_SECS.saturating_mul(1u64 << exp);
        backoff.min(Self::MAX_SECONDARY_BACKOFF_SECS)
    }

    async fn sleep_until_epoch_seconds(epoch_seconds: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        if epoch_seconds > now {
            let wait = Duration::from_secs(epoch_seconds - now);
            sleep(wait).await;
        }
    }

    /// Apply the error to the state machine and update it.
    fn apply(&self, error: Option<&RateLimitError>) {
        let mut state = self.state.lock().unwrap();
        let Some(error) = error else {
            *state = None;
            return;
        };
        let new_state = match error {
            RateLimitError::PrimaryRateLimit { .. } => Some(RateLimitState::PrimaryRateLimit),
            RateLimitError::SecondaryRateLimitRetryAfter { .. }
            | RateLimitError::SecondaryRateLimitReset { .. }
            | RateLimitError::SecondaryRateLimitWait { .. }
            | RateLimitError::SecondaryRateLimitExponentialBackoff { .. }
            | RateLimitError::SecondaryRateLimitExhausted { .. } => {
                let retries = match *state {
                    Some(RateLimitState::SecondaryRateLimit { retries }) => {
                        retries.saturating_add(1)
                    }
                    _ => 0,
                };
                Some(RateLimitState::SecondaryRateLimit { retries })
            }
            RateLimitError::Transport(_) => *state,
        };
        *state = new_state;
    }
}

impl Default for RateLimitResponseMiddleware {
    fn default() -> Self {
        Self::new(Self::DEFAULT_MAX_SECONDARY_RETRIES)
    }
}

impl From<reqwest::Error> for RateLimitError {
    fn from(value: reqwest::Error) -> Self {
        Self::Transport(value)
    }
}

impl ResponseMiddleware for RateLimitResponseMiddleware {
    type Error = RateLimitError;

    async fn on_response(
        &self,
        response: reqwest::Result<reqwest::Response>,
    ) -> Result<reqwest::Response, Self::Error> {
        let response = response.map_err(RateLimitError::Transport)?;

        let status = response.status();
        if status != reqwest::StatusCode::FORBIDDEN
            && status != reqwest::StatusCode::TOO_MANY_REQUESTS
        {
            // Clear rate-limit state on successful responses.
            self.apply(None);
            return Ok(response);
        }

        let headers = response.headers();
        let remaining = Self::parse_header_u64(headers, "x-ratelimit-remaining");
        let reset = Self::parse_header_u64(headers, "x-ratelimit-reset");
        let retry_after = Self::parse_header_u64(headers, "retry-after");

        let error = {
            let mut state = self.state.lock().unwrap();
            let prev_secondary_retries = match *state {
                Some(RateLimitState::SecondaryRateLimit { retries }) => Some(retries),
                _ => None,
            };

            // ResponseMiddleware is sync; we can't read the body to inspect the secondary-rate-limit message.
            // Infer secondary using headers or prior secondary state.
            let is_secondary = if prev_secondary_retries.is_some() || retry_after.is_some() {
                true
            } else {
                remaining != Some(0)
            };

            let error = if !is_secondary {
                // Primary limit: do not retry until x-ratelimit-reset.
                RateLimitError::PrimaryRateLimit {
                    x_ratelimit_reset: reset.unwrap_or(0),
                }
            } else {
                let attempt = prev_secondary_retries.unwrap_or(0).saturating_add(1);
                if attempt > self.max_secondary_retries {
                    // Stop the loop after too many retries.
                    RateLimitError::SecondaryRateLimitExhausted { retries: attempt }
                } else if attempt > 1 {
                    // Subsequent failures: exponential backoff.
                    let retry_after = Self::secondary_backoff_seconds(attempt);
                    RateLimitError::SecondaryRateLimitExponentialBackoff {
                        retry_after,
                        attempt,
                    }
                } else if let Some(retry_after) = retry_after {
                    // Honor explicit Retry-After for secondary limits.
                    RateLimitError::SecondaryRateLimitRetryAfter { retry_after }
                } else if remaining == Some(0) {
                    // If remaining is 0, use x-ratelimit-reset even for secondary.
                    RateLimitError::SecondaryRateLimitReset {
                        x_ratelimit_reset: reset.unwrap_or(0),
                    }
                } else {
                    // Otherwise wait at least one minute.
                    RateLimitError::SecondaryRateLimitWait {
                        retry_after: Self::MIN_SECONDARY_WAIT_SECS,
                    }
                }
            };

            *state = Some(match error {
                RateLimitError::PrimaryRateLimit { .. } => RateLimitState::PrimaryRateLimit,
                RateLimitError::SecondaryRateLimitRetryAfter { .. }
                | RateLimitError::SecondaryRateLimitReset { .. }
                | RateLimitError::SecondaryRateLimitWait { .. }
                | RateLimitError::SecondaryRateLimitExponentialBackoff { .. }
                | RateLimitError::SecondaryRateLimitExhausted { .. } => {
                    let retries = prev_secondary_retries.unwrap_or(0).saturating_add(1);
                    RateLimitState::SecondaryRateLimit { retries }
                }
                RateLimitError::Transport(_) => return Err(error),
            });
            error
        };

        match error {
            RateLimitError::PrimaryRateLimit { x_ratelimit_reset }
            | RateLimitError::SecondaryRateLimitReset { x_ratelimit_reset } => {
                Self::sleep_until_epoch_seconds(x_ratelimit_reset).await;
            }
            _ => {}
        }

        Err(error)
    }
}

#[tokio::main]
async fn main() {
    let middleware = RateLimitResponseMiddleware::default();

    // use the client without the wrapper
    {
        // Primary rate limit for authenticated users is 5,000 requests per hour.
        let rate_limiter =
            governor::RateLimiter::direct(Quota::per_hour(NonZeroU32::new(5_000).unwrap()));

        // GitHub REST API requires a User-Agent header.
        let reqwest_client = reqwest::Client::builder()
            .user_agent("reqwest-rate-limit-example")
            .build()
            .unwrap();
        // Example request wired with the primary rate limiter.
        let request = reqwest_client.get("https://api.github.com/rate_limit");
        // Send this in an async context and await the returned future.
        let req = reqwest_rate_limit::send_with_rate_limiter_and_middleware(
            request,
            &rate_limiter,
            &middleware,
        );
        let _res = req.await.unwrap();
    }

    // use the client with the wrapper
    {
        let rate_limiter =
            governor::RateLimiter::direct(Quota::per_hour(NonZeroU32::new(5_000).unwrap()));

        let client = reqwest_rate_limit::Client::builder()
            .user_agent("reqwest-rate-limit-example")
            .response_middleware(middleware)
            .rate_limiter(Arc::new(rate_limiter))
            .build()
            .unwrap();

        let req = client.get("https://github.com").send();
        let _res = req.await.unwrap();
    }
}
