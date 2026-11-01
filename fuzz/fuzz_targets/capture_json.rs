#![no_main]

use clens_capture::{
    decode_file_record, decode_network_record, decode_process_record, decode_registry_record,
};
use clens_types::SysKind;
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

fuzz_target!(|bytes: &[u8]| {
    let Ok(value) = serde_json::from_slice::<Value>(bytes) else {
        return;
    };
    let _ = decode_process_record(&value);
    let _ = decode_file_record(&value, SysKind::FileRead);
    let _ = decode_registry_record(&value, SysKind::RegQuery);
    let _ = decode_network_record(&value, SysKind::NetConnect);
});
