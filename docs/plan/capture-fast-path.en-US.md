# Capture P0 refinement: mmap ring and manual layout fast path

This plan closes the two main capture bottlenecks together: the JSON file bridge between capture and daemon, and the current per-event TDH decode path. They must be delivered as one data-plane change; replacing only one side cannot justify the throughput budget.

## 1. Current boundary

The current path is:

```text
EVENT_RECORD -> SysEvent -> JSON -> output file -> daemon line reader
             -> JSON parse -> alignment / SQLite
```

`RingHeader`, the 95% spill policy, and `.clspool` CRC recovery exist as protocol models and tests, but the cross-process `CreateFileMapping` producer/consumer is not wired into the daemon. `TdhGetEventInformation` and `TdhGetProperty*` remain on the normal realtime path. Opcode classification and raw-byte path guessing are not a complete manual layout decoder.

Until this plan is complete, CausativeLens must not claim 200k events/s or use JSON bridge numbers as the capture baseline.

## 2. Target data plane

```text
ETW EVENT_RECORD
  -> FastLayoutDecoder (known layout, no TDH)
  -> binary frame
  -> shared-memory ring (SPSC)
  -> daemon ring reader
  -> one IR decode
  -> alignment / SQLite

Unknown event, unknown version, or fast-path validation failure:
  -> raw frame -> TDH fallback worker -> explicit health counters
```

The JSON file bridge remains for diagnostics and compatibility, marked explicitly as `transport=json-fallback`.

## 3. Ring protocol

Windows uses `CreateFileMappingW` and `MapViewOfFile`; the mapping name is `Local\\CausativeLens_<session>_ring`. The daemon creates the mapping and the elevated capture host opens the same per-user object. The ACL is limited to the current user and required system access. Mapping failure may enable JSON fallback only with an explicit health reason.

The 64-byte-aligned header contains `magic`, `schema_v`, `capacity`, `writer_pos`, `reader_pos`, `dropped`, `spill_frames`, `heartbeat`, and `last_seq`. Writer publication uses Release ordering and reader progress uses Release/Acquire ordering.

Frames use an explicit little-endian header:

```text
magic:u32 | schema_v:u16 | flags:u16 | frame_len:u32 |
seq:u64 | ts_qpc:u64 | payload_crc32:u32 | payload:bytes
```

Frame types include `SysEvent`, `PathTable`, `Health`, and `Gap`. `SysEvent` payloads use an audited cursor codec; path text appears only in `PathTable` frames and events carry `PathIdx`.

At 95% occupancy, or when a frame cannot fit, capture spills to `.clspool` without blocking the ETW consumer. The reader validates magic, length, CRC, and sequence, records gaps, and continues. Ring and spool share the same sequence-based idempotency rule.

## 4. Manual layout fast path

Layout selection uses the complete key:

```text
(provider_guid, event_id, version, opcode, task)
```

An exact key miss must fall back to TDH; layouts cannot be selected by opcode alone. Every `LayoutSpec` declares field names, offsets, widths, alignment, encoding, optional fields, minimum payload length, and the destination `SysEvent` fields. The catalog is checked-in Rust constants or generated read-only data backed by a TDH manifest snapshot and Windows build.

The first event families are Process Start/Stop/ImageLoad; File Create/Read/Write/Close/SetInformation/Delete; Registry Open/Query/Set/Delete; and Network Connect/Disconnect/Send/Receive/Listen/Bind. The first implementation covers the fields required for alignment: pid/tid/QPC, FileObject or endpoint, path/key, access mask, and length.

The runtime path is:

```rust
match layout_catalog.lookup(key) {
    Some(layout) => fast_decode(layout, user_data),
    None => slow_queue.push(raw_record),
}
```

The fast path performs no TDH calls or field-name allocation, checks all bounds, and never panics. TDH fallback runs on a slow worker rather than inside the ETW callback.

## 5. TDH oracle and self-check

Each layout stores raw-event fixtures, TDH field names, and typed-value goldens. Manual and TDH results are compared field by field; any mismatch blocks the layout from the default catalog. `CAUSATIVELENS_TDH_SELFTEST=1` may sample the first N events on a new Windows build. Unknown events retain raw payload and increment `tdh_fallback_events`, `unknown_layout_events`, and `decode_failures`.

TDH metadata is cached by `LayoutKey`; even fallback must not call `TdhGetEventInformation` repeatedly for the same event version.

## 6. Tests and performance gates

Tests cover truncation, oversized payloads, alignment errors, invalid UTF-16, integer overflow, ring round-trips, CRC/gaps, sequence de-duplication, crash resume, and property-based monotonicity of reader/writer positions. Architecture tests assert that ring is the daemon default and JSON is only an explicit fallback.

The benchmark matrix is:

```text
manual_decode_process_top
manual_decode_file_top
manual_decode_registry_top
manual_decode_network_top
tdh_cached_decode
tdh_uncached_decode
mmap_ring_write_read
capture_to_verdict_end_to_end
```

Each run records target, Windows build, CPU, commit, sample count, throughput, P50/P99, CPU, memory, `EventsLost`, `RealTimeBuffersLost`, ring drops, spill frames, and TDH fallback ratio. A metric without a measured value must not appear in release notes as verified.

The P0 closes only when the first event families are covered, oracle goldens pass, unknown versions safely fall back, ring stress has no silent loss, crash gaps are visible, x64 end-to-end numbers are recorded, and Windows 10 Build 10240+/Windows Server 2016+ compatibility checks pass.

## 7. Task split

- `T0202`: ring transport, mapping ACL, binary frames, spill, and recovery;
- `T0203`: LayoutKey/LayoutSpec catalog plus Process/File fast decoders;
- `T0204`: Registry/Network fast decoders and TDH oracle goldens;
- `T0205`: daemon ring reader, JSON fallback, health fields, and crash tests;
- `T0206`: capture end-to-end benchmark, performance report, and release gate.

Only after these cards pass may the capture bridge be described as verified real observation rather than a runnable demonstration.
