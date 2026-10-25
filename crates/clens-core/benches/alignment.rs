use clens_core::{align_candidate, AlignmentProfile, TimedAssertion};
use clens_types::{
    Assertion, AssertionOp, EventPayload, Expectation, IoMode, SysEvent, SysKind, TargetPattern,
};
use std::hint::black_box;
use std::time::Instant;
fn main() {
    let a = Assertion {
        schema_v: 1,
        id: 1,
        span: clens_types::HarnessSpan {
            step: 1,
            byte_start: 0,
            byte_end: 5,
        },
        op: AssertionOp::Read,
        target: TargetPattern::Path("config.yaml".into()),
        io_mode: IoMode::ReadOnly,
        expect: Expectation::MustHappen,
        order_key: None,
        confidence: 1.0,
        raw_quote: "read".into(),
    };
    let e = SysEvent {
        schema_v: 1,
        seq: 1,
        ts_qpc: 1000,
        pid: 1,
        tid: 1,
        kind: SysKind::FileRead,
        is_rundown: false,
        payload: EventPayload::File {
            path: 1,
            desired_access: 0,
            io_len: Some(1),
            write_attrs: false,
        },
    };
    let iterations = 10_000u32;
    let start = Instant::now();
    for _ in 0..iterations {
        for _ in 0..50 {
            black_box(align_candidate(
                TimedAssertion {
                    assertion: &a,
                    center_ts_ms: 1000,
                    window_end_ms: 2000,
                },
                &e,
                0,
                1.0,
                1.0,
                AlignmentProfile::default(),
            ));
        }
    }
    let elapsed = start.elapsed().as_nanos();
    println!("alignment_50_active_assertions iterations={iterations} elapsed_ns={elapsed} ns_per_batch={}", elapsed/u128::from(iterations));
}
