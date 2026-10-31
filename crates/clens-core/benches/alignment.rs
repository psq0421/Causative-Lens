use clens_core::{align_candidate, AlignmentProfile, TimedAssertion};
use clens_types::{
    Assertion, AssertionOp, EventPayload, Expectation, HarnessSpan, IoMode, SysEvent, SysKind,
    TargetPattern,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn sample() -> (Assertion, SysEvent) {
    (
        Assertion {
            schema_v: 1,
            id: 1,
            span: HarnessSpan {
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
        },
        SysEvent {
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
        },
    )
}

fn alignment_benchmark(c: &mut Criterion) {
    let (assertion, event) = sample();
    let profile = AlignmentProfile::default();
    c.bench_function("alignment/one_candidate", |b| {
        b.iter(|| {
            align_candidate(
                black_box(TimedAssertion {
                    assertion: &assertion,
                    center_ts_ms: 1000,
                    window_end_ms: 2000,
                }),
                black_box(&event),
                black_box(0),
                black_box(1.0),
                black_box(1.0),
                black_box(profile),
            )
        })
    });
}

criterion_group!(benches, alignment_benchmark);
criterion_main!(benches);
