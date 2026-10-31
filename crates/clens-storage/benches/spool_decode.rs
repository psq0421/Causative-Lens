use clens_storage::{recover_frames, SpoolFrame};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn spool_benchmark(c: &mut Criterion) {
    let mut bytes = Vec::new();
    for seq in 0..10_000_u32 {
        let frame = SpoolFrame {
            schema_v: 1,
            flags: 0,
            payload: seq.to_le_bytes().repeat(12),
            raw_len: 48,
        };
        bytes.extend_from_slice(&frame.encode());
    }
    c.bench_function("spool/recover_10k_frames", |b| {
        b.iter(|| black_box(recover_frames(black_box(&bytes))))
    });
}

criterion_group!(benches, spool_benchmark);
criterion_main!(benches);
