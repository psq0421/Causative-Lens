#![no_main]

use clens_harnessd::{HarnessAdapter, JsonlAdapter};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let adapter = JsonlAdapter::default();
    let _ = adapter.parse(bytes);
});
