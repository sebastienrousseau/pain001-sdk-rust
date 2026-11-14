// SPDX-License-Identifier: Apache-2.0 OR MIT
//! A 2xx response body, decoded as every typed call's response: it must
//! decode or fail with `Error::Decode`, never panic, and `parse_body` must
//! keep its contract.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        pain001_sdk_fuzz::check_success_body(text);
    }
});
