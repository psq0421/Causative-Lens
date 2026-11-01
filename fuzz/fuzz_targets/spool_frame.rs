#![no_main]

use clens_storage::{recover_frames, SpoolFrame};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let _ = SpoolFrame::decode(bytes);
    let _ = recover_frames(bytes);
});
