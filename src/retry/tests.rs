// SPDX-License-Identifier: Apache-2.0 OR MIT
//! RetryPolicy unit tests: the backoff, `Retry-After` and when to stop.

use super::*;
use serde_json::Value;

const MS: Duration = Duration::from_millis(1);

fn reply(status: u16, retry_after: Option<&str>) -> Result<Reply, Error> {
    Ok(Reply {
        status,
        retry_after: retry_after.map(str::to_owned),
        value: Value::Null,
    })
}

fn transport() -> Result<Reply, Error> {
    Err(Error::transport(std::io::Error::other("refused")))
}

#[test]
fn the_defaults_match_the_suite() {
    let policy = RetryPolicy::new();
    assert_eq!(policy, RetryPolicy::default());
    assert_eq!(policy.max_attempts(), 3);
    assert_eq!(policy.base_delay(), 200 * MS);
    assert_eq!(policy.max_delay(), Duration::from_secs(5));
    assert_eq!(policy.with_max_attempts(0).max_attempts(), 1);
}

#[test]
fn the_backoff_doubles_up_to_the_cap_with_jitter_in_the_upper_half() {
    let policy = RetryPolicy::new()
        .with_base_delay(100 * MS)
        .with_max_delay(350 * MS);
    // random 0 picks the bottom of the upper half, u64::MAX % (span + 1)
    // somewhere in it; never below half or above the ceiling.
    assert_eq!(policy.backoff(1, 0), 50 * MS);
    assert_eq!(policy.backoff(2, 0), 100 * MS);
    assert_eq!(policy.backoff(3, 0), 175 * MS);
    assert_eq!(policy.backoff(1, 50_000_000), 100 * MS);
    for retry in 1..40 {
        for random in [0, 1, 12_345_678_901, u64::MAX] {
            let wait = policy.backoff(retry, random);
            assert!(wait <= 350 * MS, "retry {retry}: {wait:?}");
        }
    }
    let unbounded = RetryPolicy::new().with_max_delay(Duration::MAX);
    assert!(unbounded.backoff(40, u64::MAX) > Duration::from_secs(1));
}

#[test]
fn transport_errors_429_and_503_are_retried_until_the_last_attempt() {
    let policy = RetryPolicy::new().with_base_delay(10 * MS);
    let now = SystemTime::now();
    assert_eq!(policy.next_wait(1, &transport(), now, 0), Some(5 * MS));
    assert_eq!(
        policy.next_wait(2, &reply(503, None), now, 0),
        Some(10 * MS)
    );
    assert_eq!(
        policy.next_wait(1, &reply(429, Some("1")), now, 0),
        Some(1000 * MS)
    );
    assert_eq!(policy.next_wait(3, &transport(), now, 0), None);
    assert_eq!(policy.next_wait(1, &reply(500, None), now, 0), None);
    assert_eq!(policy.next_wait(1, &reply(200, None), now, 0), None);
    let decode = Err(Error::Decode(serde_json::from_str::<u8>("x").unwrap_err()));
    assert_eq!(policy.next_wait(1, &decode, now, 0), None);
}

#[test]
fn a_retry_after_is_waited_exactly_unless_it_is_too_long() {
    let policy = RetryPolicy::new();
    let now = SystemTime::now();
    assert_eq!(
        policy.next_wait(1, &reply(503, Some("5")), now, 0),
        Some(5000 * MS)
    );
    assert_eq!(policy.next_wait(1, &reply(503, Some("6")), now, 0), None);
    // An unreadable value falls back to the backoff.
    assert_eq!(
        policy.next_wait(1, &reply(503, Some("soon")), now, 0),
        Some(100 * MS)
    );
}

#[test]
fn retry_after_reads_seconds_and_http_dates() {
    // 1994-11-06T08:49:37Z, the RFC 9110 example.
    let example = UNIX_EPOCH + Duration::from_secs(784_111_777);
    let before = example - Duration::from_secs(90);
    let date = "Sun, 06 Nov 1994 08:49:37 GMT";
    assert_eq!(retry_after(date, before), Some(Duration::from_secs(90)));
    assert_eq!(retry_after(date, example + MS), Some(Duration::ZERO));
    assert_eq!(retry_after(" 30 ", before), Some(Duration::from_secs(30)));
    let huge = "99999999999999999999999";
    assert_eq!(
        retry_after(huge, before),
        Some(Duration::from_secs(u64::MAX))
    );
    assert_eq!(retry_after("", before), None);
}

#[test]
fn http_dates_across_months_and_leap_years() {
    let at = |s: u64| Some(UNIX_EPOCH + Duration::from_secs(s));
    assert_eq!(http_date("Thu, 01 Jan 1970 00:00:00 GMT"), at(0));
    assert_eq!(http_date("Sat, 01 Mar 1997 00:00:00 GMT"), at(857_174_400));
    assert_eq!(http_date("Tue, 29 Feb 2000 23:59:59 GMT"), at(951_868_799));
    assert_eq!(
        http_date("Sat, 14 Nov 2026 12:00:00 GMT"),
        at(1_794_657_600)
    );
    assert_eq!(http_date("Wed, 31 Dec 1969 23:59:59 GMT"), None);
}

#[test]
fn malformed_http_dates_are_refused() {
    for value in [
        "Sunday, 06-Nov-94 08:49:37 GMT",
        "Sun Nov  6 08:49:37 1994",
        "Sun, 06 Nov 1994 08:49:37 UTC",
        "Sun, 06 Nob 1994 08:49:37 GMT",
        "Sun, 06 Nov 99999 08:49:37 GMT",
        "Sun, 6th Nov 1994 08:49:37 GMT",
        "Sun, 06 Nov 1994 08:49 GMT",
        "Sun, 06 Nov 1994 xx:49:37 GMT",
        "Sun, 06 Nov 1994 08:xx:37 GMT",
        "Sun, 06 Nov 1994 08:49:xx GMT",
    ] {
        assert_eq!(http_date(value), None, "{value}");
    }
}

#[test]
fn run_sleeps_between_attempts_and_returns_the_last_outcome() {
    let policy = RetryPolicy::new().with_base_delay(MS).with_max_delay(MS);
    let mut answers = vec![reply(200, None), reply(503, Some("0")), transport()];
    let mut slept = Vec::new();
    let outcome = run(
        &policy,
        || answers.pop().expect("an answer"),
        |d| slept.push(d),
    );
    assert_eq!(outcome.expect("third").status, 200);
    assert_eq!(slept.len(), 2);
    assert_eq!(slept[1], Duration::ZERO);
    assert!(answers.is_empty());
    assert_ne!(random(), random());
}
