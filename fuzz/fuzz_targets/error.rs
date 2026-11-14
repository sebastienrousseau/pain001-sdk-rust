// SPDX-License-Identifier: Apache-2.0 OR MIT
//! A response with any status: the first line is the status code, the
//! rest is the body. A non-2xx reply must map to the documented error
//! variant with the documented `detail`, never panic.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        if let Some((status, body)) = pain001_sdk_fuzz::split_status(text) {
            pain001_sdk_fuzz::check_reply(status, body);
        }
    }
});
