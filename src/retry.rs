// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Opt-in retries for idempotent calls.

use std::hash::{BuildHasher, RandomState};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::client::Reply;
use crate::Error;

/// When a [`Client`](crate::Client) tries a `GET` call again.
///
/// Retries are off unless a policy is set with
/// [`Client::retry`](crate::Client::retry). With one, a `GET` call
/// ([`health`](crate::Client::health), [`faces`](crate::Client::faces),
/// and [`request`](crate::Client::request) with `"GET"`) is tried again
/// after a connection failure or timeout ([`Error::Transport`]), or an
/// HTTP `429` or `503` answer, up to [`max_attempts`](Self::max_attempts)
/// attempts in all. A `POST` is never retried: repeating `validate`,
/// `generate`, `check` or `project_face` could submit a payment file
/// twice.
///
/// The wait before retry `n` (from 1) is exponential with jitter: a random
/// point in the upper half of `min(max_delay, base_delay * 2^(n-1))`. A
/// `Retry-After` header on a `429` or `503`, in seconds or as an HTTP
/// date, is waited exactly instead; one asking for longer than
/// [`max_delay`](Self::max_delay) ends the retries and returns that
/// answer, so the client never comes back earlier than the server asked.
/// The client's timeout applies to each attempt.
///
/// ```
/// use pain001_sdk::{Client, RetryPolicy};
/// use std::time::Duration;
///
/// let policy = RetryPolicy::new()
///     .with_max_attempts(4)
///     .with_base_delay(Duration::from_millis(100));
/// let client = Client::new("http://127.0.0.1:8000").retry(policy);
/// assert_eq!(policy.max_delay(), RetryPolicy::DEFAULT_MAX_DELAY);
/// # drop(client);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy {
            max_attempts: Self::DEFAULT_MAX_ATTEMPTS,
            base_delay: Self::DEFAULT_BASE_DELAY,
            max_delay: Self::DEFAULT_MAX_DELAY,
        }
    }
}

impl RetryPolicy {
    /// Attempts in all, the first one included, of [`RetryPolicy::new`].
    pub const DEFAULT_MAX_ATTEMPTS: u32 = 3;
    /// The wait before the first retry, before jitter.
    pub const DEFAULT_BASE_DELAY: Duration = Duration::from_millis(200);
    /// The longest wait before any retry, and the longest `Retry-After`
    /// honoured.
    pub const DEFAULT_MAX_DELAY: Duration = Duration::from_secs(5);

    /// The defaults: 3 attempts, 200 ms base delay, 5 s maximum delay.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the number of attempts in all, the first one included. `1`
    /// (or `0`, read as `1`) makes the policy retry nothing.
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts.max(1);
        self
    }

    /// Sets the wait before the first retry, before jitter; each later
    /// wait doubles it, up to the maximum delay.
    pub fn with_base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Sets the longest wait before any retry, which is also the longest
    /// `Retry-After` the client waits for.
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Attempts in all, the first one included.
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    /// The wait before the first retry, before jitter.
    pub fn base_delay(&self) -> Duration {
        self.base_delay
    }

    /// The longest wait before any retry.
    pub fn max_delay(&self) -> Duration {
        self.max_delay
    }

    /// The backoff before retry `retry` (from 1): a point in the upper
    /// half of `min(max_delay, base_delay * 2^(retry-1))`, picked by
    /// `random`.
    pub(crate) fn backoff(&self, retry: u32, random: u64) -> Duration {
        let shift = retry.saturating_sub(1).min(31);
        let ceiling = self
            .base_delay
            .saturating_mul(1 << shift)
            .min(self.max_delay);
        let half = ceiling / 2;
        let span = (ceiling - half).as_nanos();
        let extra = u64::try_from(u128::from(random) % (span + 1)).unwrap_or(u64::MAX);
        half.saturating_add(Duration::from_nanos(extra))
    }

    /// The wait after attempt `attempt` (from 1) ended in `outcome`, or
    /// `None` when that outcome is the call's result.
    pub(crate) fn next_wait(
        &self,
        attempt: u32,
        outcome: &Result<Reply, Error>,
        now: SystemTime,
        random: u64,
    ) -> Option<Duration> {
        if attempt >= self.max_attempts {
            return None;
        }
        match outcome {
            Err(Error::Transport(_)) => Some(self.backoff(attempt, random)),
            Ok(reply) if matches!(reply.status, 429 | 503) => {
                self.server_wait(reply, attempt, now, random)
            }
            _ => None,
        }
    }

    /// The wait the server asked for with `Retry-After`, or the backoff
    /// without a readable one; `None` when it asked for too long.
    fn server_wait(
        &self,
        reply: &Reply,
        attempt: u32,
        now: SystemTime,
        random: u64,
    ) -> Option<Duration> {
        match reply
            .retry_after
            .as_deref()
            .and_then(|v| retry_after(v, now))
        {
            Some(wait) if wait > self.max_delay => None,
            Some(wait) => Some(wait),
            None => Some(self.backoff(attempt, random)),
        }
    }
}

/// Makes attempts until `policy` says to stop, sleeping with `sleep`
/// between them, and returns the last outcome.
pub(crate) fn run<A, S>(policy: &RetryPolicy, mut attempt: A, mut sleep: S) -> Result<Reply, Error>
where
    A: FnMut() -> Result<Reply, Error>,
    S: FnMut(Duration),
{
    let mut made = 1;
    loop {
        let outcome = attempt();
        match policy.next_wait(made, &outcome, SystemTime::now(), random()) {
            Some(wait) => sleep(wait),
            None => return outcome,
        }
        made += 1;
    }
}

/// A random `u64` for jitter, from the standard library's per-process
/// random hash keys; no dependency, and good enough to spread retries.
fn random() -> u64 {
    RandomState::new().hash_one(SystemTime::now())
}

/// A `Retry-After` value as a wait from `now`: delta-seconds, or an
/// IMF-fixdate (RFC 9110, section 10.2.3). A date in the past is no wait.
pub(crate) fn retry_after(value: &str, now: SystemTime) -> Option<Duration> {
    let value = value.trim();
    if !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()) {
        return Some(Duration::from_secs(value.parse().unwrap_or(u64::MAX)));
    }
    let at = http_date(value)?;
    Some(at.duration_since(now).unwrap_or(Duration::ZERO))
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// An IMF-fixdate, `Sun, 06 Nov 1994 08:49:37 GMT`, as a time. The
/// obsolete RFC 850 and asctime forms are not read.
fn http_date(value: &str) -> Option<SystemTime> {
    let fields: Vec<&str> = value.split(' ').collect();
    let [_weekday, day, month, year, time, "GMT"] = fields.as_slice() else {
        return None;
    };
    let month = MONTHS.iter().position(|m| m == month)?;
    // Four-digit years and two-digit days: u16 and u8 bound the arithmetic.
    let year = i64::from(year.parse::<u16>().ok()?);
    let day = i64::from(day.parse::<u8>().ok()?);
    let seconds = days_from_civil(year, month, day) * 86_400 + seconds_of_day(time)?;
    u64::try_from(seconds)
        .ok()
        .map(|s| UNIX_EPOCH + Duration::from_secs(s))
}

/// `HH:MM:SS` as seconds since midnight.
fn seconds_of_day(time: &str) -> Option<i64> {
    let fields: Vec<&str> = time.split(':').collect();
    let [hours, minutes, seconds] = fields.as_slice() else {
        return None;
    };
    let [hours, minutes, seconds] =
        [hours, minutes, seconds].map(|f| f.parse::<u8>().ok().map(i64::from));
    Some(hours? * 3600 + minutes? * 60 + seconds?)
}

/// Days from 1970-01-01 to `year`-`month0 + 1`-`day` in the proleptic
/// Gregorian calendar (Howard Hinnant's `days_from_civil`).
fn days_from_civil(year: i64, month0: usize, day: i64) -> i64 {
    // Months counted from March, so the leap day ends the year.
    let march = i64::try_from((month0 + 10) % 12).unwrap_or(0);
    let year = if march >= 10 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * march + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests;
