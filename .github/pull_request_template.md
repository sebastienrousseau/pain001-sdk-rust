<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->
<!--
Pull request description for pain001-sdk-rust. The shape follows the
shared PR template (section order, Highlights, fenced Checksums block,
Full Changelog line), aligned with the release page format at
https://github.com/sebastienrousseau/passmcp/releases/tag/v0.0.5.

- Title: a Conventional Commit, e.g. "fix: keep the detail of a 422".
- Open with a two-line BLUF: line 1 begins "This PR " and says what it
  does; line 2 says why it matters or the outcome. Put a blank line
  between them, and no paragraph above.
- Write like a person: direct, concrete, no filler. No em dash
  characters anywhere; use commas, colons, parentheses or a spaced
  hyphen. No tool-attribution footer.
- Keep the sections and their order. Fill every placeholder from real
  evidence: commands that ran and what they printed.
- Every commit is signed and carries a Signed-off-by from its author.
-->

This PR <what it does, in one plain line>.

<why it matters or the outcome, in one plain line>

## Highlights ⭐️

* **<Headline>**: <one or two plain sentences on what changed for the user>.
* **<Headline>**: <one or two plain sentences>.

## What's Changed

* `<commit subject>` by @<author>

<optional: a few detail bullets naming files and the concrete change>

## Validation

* `make check`: <tests passed, clippy, doc and cargo-deny results>.
* `PAIN001_BASE_URL=... make conformance`: <cases passed of the kit, server version>.

## Checksums

SHA-256 of the artifact produced by `cargo package` on this branch:

```text
<sha256>  target/package/pain001-sdk-<version>.crate
```

**Full Changelog**: <https://github.com/sebastienrousseau/pain001-sdk-rust/compare/BASE...HEAD>
