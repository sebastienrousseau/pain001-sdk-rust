<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# 4. Retries are opt-in and for GET only

- Status: accepted
- Date: 2026-11-14

## Context

A client that reaches pain001 through a load balancer, or while the
server restarts, sees refused connections, timeouts, `429` and `503`
answers that a second try would clear. Until 0.0.88 the client sent every
request exactly once, so each caller wrapped `health` and `faces` in a
loop of its own.

Retrying is not free. `validate`, `generate`, `check` and `project_face`
are `POST` calls that carry a payment file. A timeout does not say whether
the server processed the request, so retrying one could submit the same
file twice. The suite's five SDKs agreed on one behaviour.

## Decision

`RetryPolicy`, set with `Client::retry`, retries `GET` calls only:
`health`, `faces`, and `request` with `"GET"`. Without a policy the client
makes one attempt per call, as before.

- Retried: `Error::Transport` (connection refused, timeout, TLS, a body
  cut short), and HTTP `429` and `503`.
- Attempts: `max_attempts` in all, 3 by default; `1` retries nothing.
- Wait before retry `n`: a random point in the upper half of
  `min(max_delay, base_delay * 2^(n-1))`; 200 ms and 5 s by default.
- `Retry-After` on a `429` or `503`, in delta-seconds or as an
  IMF-fixdate, is waited exactly. One longer than `max_delay` ends the
  retries and returns that answer, so the client never returns earlier
  than the server asked.
- The client timeout applies to each attempt, not to the call as a
  whole: ureq's timeout is per request, and a shared deadline would add
  per-request configuration for little gain. The worst case is
  `max_attempts * timeout` plus the waits, which the policy bounds.
- Jitter comes from the standard library's `RandomState`, so retries add
  no dependency.

The names and defaults match the Go, Java, .NET and C++ SDKs.

## Consequences

- Behaviour is unchanged for every existing caller.
- A `POST` that fails is the caller's to retry, with whatever knowledge
  it has of whether the server acted on it.
- The obsolete RFC 850 and asctime date forms are not read; such a
  `Retry-After` falls back to the backoff.
