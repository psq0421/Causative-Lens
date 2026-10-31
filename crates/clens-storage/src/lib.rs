pub const GRAPH_SCHEMA_V1: &str = r#"PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA page_size=8192;
PRAGMA mmap_size=268435456;
CREATE TABLE IF NOT EXISTS assertions (id INTEGER PRIMARY KEY, schema_v INTEGER NOT NULL, step INTEGER, span_json TEXT, op TEXT, target TEXT, io_mode TEXT, expect TEXT, order_key TEXT, confidence REAL, raw_quote TEXT, source TEXT) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS verdicts (assertion_id INTEGER PRIMARY KEY REFERENCES assertions(id), schema_v INTEGER NOT NULL, state TEXT, deviation_json TEXT, decided_at INTEGER) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS evidence (assertion_id INTEGER, event_seq INTEGER, score REAL, components_json TEXT, PRIMARY KEY (assertion_id, event_seq)) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS event_meta (seq INTEGER PRIMARY KEY, schema_v INTEGER NOT NULL, ts_qpc INTEGER, pid INTEGER, tid INTEGER, kind TEXT, is_rundown INTEGER NOT NULL DEFAULT 0, block_id INTEGER, block_off INTEGER, path_id INTEGER, remote TEXT) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS paths (id INTEGER PRIMARY KEY, path TEXT UNIQUE);
CREATE VIRTUAL TABLE IF NOT EXISTS event_search USING fts5(seq UNINDEXED, text, path, remote);
CREATE TABLE IF NOT EXISTS interventions (id INTEGER PRIMARY KEY, assertion_id INTEGER, payload_json TEXT);
CREATE TABLE IF NOT EXISTS decisions (id INTEGER PRIMARY KEY, intervention_id INTEGER, payload_json TEXT);"#;

pub const SESSION_EXTENSION: &str = ".cllens";
pub const SPOOL_EXTENSION: &str = ".clspool";
pub const SPOOL_MAGIC: u32 = 0x434C5324;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpoolFrame {
    pub schema_v: u16,
    pub flags: u16,
    pub payload: Vec<u8>,
    pub raw_len: u32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpoolError {
    Truncated,
    BadMagic,
    BadCrc,
    InvalidLength,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpoolFileError {
    Io(String),
}

/// Append-only `.clspool` writer used by the capture/daemon boundary.
/// Each append is one complete CRC-protected frame; a crash can therefore
/// leave a tail that `recover_file` reports as a gap instead of hiding it.
#[derive(Debug)]
pub struct SpoolWriter {
    file: std::fs::File,
    offset: u64,
}

impl SpoolWriter {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, SpoolFileError> {
        let path = path.as_ref();
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|error| SpoolFileError::Io(error.to_string()))?;
        let offset = file
            .metadata()
            .map_err(|error| SpoolFileError::Io(error.to_string()))?
            .len();
        Ok(Self { file, offset })
    }

    pub fn append(&mut self, frame: &SpoolFrame) -> Result<u64, SpoolFileError> {
        use std::io::Write;

        let encoded = frame.encode();
        let start = self.offset;
        self.file
            .write_all(&encoded)
            .map_err(|error| SpoolFileError::Io(error.to_string()))?;
        self.offset = self.offset.saturating_add(encoded.len() as u64);
        Ok(start)
    }

    pub fn sync(&mut self) -> Result<(), SpoolFileError> {
        self.file
            .sync_data()
            .map_err(|error| SpoolFileError::Io(error.to_string()))
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }
}

pub fn recover_file(path: impl AsRef<std::path::Path>) -> Result<RecoveryScan, SpoolFileError> {
    let bytes = std::fs::read(path).map_err(|error| SpoolFileError::Io(error.to_string()))?;
    Ok(recover_frames(&bytes))
}
impl SpoolFrame {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(20 + self.payload.len());
        out.extend_from_slice(&SPOOL_MAGIC.to_le_bytes());
        out.extend_from_slice(&self.schema_v.to_le_bytes());
        out.extend_from_slice(&self.flags.to_le_bytes());
        out.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.raw_len.to_le_bytes());
        out.extend_from_slice(&crc32(&self.payload).to_le_bytes());
        out.extend_from_slice(&self.payload);
        out
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, SpoolError> {
        if bytes.len() < 20 {
            return Err(SpoolError::Truncated);
        }
        if u32::from_le_bytes(bytes[0..4].try_into().map_err(|_| SpoolError::Truncated)?)
            != SPOOL_MAGIC
        {
            return Err(SpoolError::BadMagic);
        }
        let n = u32::from_le_bytes(bytes[8..12].try_into().map_err(|_| SpoolError::Truncated)?)
            as usize;
        if bytes.len() != 20 + n {
            return Err(SpoolError::InvalidLength);
        }
        let payload = bytes[20..].to_vec();
        let expected = u32::from_le_bytes(
            bytes[16..20]
                .try_into()
                .map_err(|_| SpoolError::Truncated)?,
        );
        if crc32(&payload) != expected {
            return Err(SpoolError::BadCrc);
        }
        Ok(Self {
            schema_v: u16::from_le_bytes(
                bytes[4..6].try_into().map_err(|_| SpoolError::Truncated)?,
            ),
            flags: u16::from_le_bytes(bytes[6..8].try_into().map_err(|_| SpoolError::Truncated)?),
            raw_len: u32::from_le_bytes(
                bytes[12..16]
                    .try_into()
                    .map_err(|_| SpoolError::Truncated)?,
            ),
            payload,
        })
    }
}
#[derive(Debug, Default)]
pub struct RecoveryScan {
    pub frames: Vec<SpoolFrame>,
    pub gaps: Vec<usize>,
}
pub fn recover_frames(bytes: &[u8]) -> RecoveryScan {
    let mut s = RecoveryScan::default();
    let mut o = 0usize;
    while o + 20 <= bytes.len() {
        let m = u32::from_le_bytes(bytes[o..o + 4].try_into().unwrap_or([0; 4]));
        if m != SPOOL_MAGIC {
            s.gaps.push(o);
            o = o.saturating_add(1);
            continue;
        }
        let n = u32::from_le_bytes(bytes[o + 8..o + 12].try_into().unwrap_or([0; 4])) as usize;
        let l = 20usize.saturating_add(n);
        if o.saturating_add(l) > bytes.len() {
            s.gaps.push(o);
            break;
        }
        match SpoolFrame::decode(&bytes[o..o + l]) {
            Ok(f) => {
                s.frames.push(f);
                o += l
            }
            Err(_) => {
                s.gaps.push(o);
                o = o.saturating_add(l)
            }
        }
    }
    if o < bytes.len() {
        s.gaps.push(o)
    }
    s
}
fn crc32(bytes: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFF;
    for b in bytes {
        c ^= u32::from(*b);
        for _ in 0..8 {
            c = if c & 1 == 1 {
                (c >> 1) ^ 0xEDB8_8320
            } else {
                c >> 1
            }
        }
    }
    !c
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn schema_keeps_forward_compatible_tables() {
        assert!(GRAPH_SCHEMA_V1.contains("CREATE TABLE IF NOT EXISTS interventions"));
        assert!(GRAPH_SCHEMA_V1.contains("CREATE TABLE IF NOT EXISTS decisions"));
        assert!(GRAPH_SCHEMA_V1.contains("USING fts5"));
    }
    #[test]
    fn frame_round_trip_preserves_payload() {
        let f = SpoolFrame {
            schema_v: 1,
            flags: 4,
            payload: b"event-batch".to_vec(),
            raw_len: 11,
        };
        assert_eq!(SpoolFrame::decode(&f.encode()).expect("round trip"), f);
    }
    #[test]
    fn corrupt_frame_is_reported_as_gap_candidate() {
        let f = SpoolFrame {
            schema_v: 1,
            flags: 0,
            payload: b"event".to_vec(),
            raw_len: 5,
        };
        let mut b = f.encode();
        b[20] ^= 0xFF;
        assert_eq!(SpoolFrame::decode(&b), Err(SpoolError::BadCrc));
    }
    #[test]
    fn recovery_keeps_valid_frames_and_records_corrupt_offsets() {
        let g = SpoolFrame {
            schema_v: 1,
            flags: 0,
            payload: b"ok".to_vec(),
            raw_len: 2,
        }
        .encode();
        let mut b = SpoolFrame {
            schema_v: 1,
            flags: 0,
            payload: b"bad".to_vec(),
            raw_len: 3,
        }
        .encode();
        b[20] ^= 0xFF;
        let mut all = g.clone();
        all.extend_from_slice(&b);
        let s = recover_frames(&all);
        assert_eq!(s.frames.len(), 1);
        assert_eq!(s.gaps, vec![g.len()]);
    }

    #[test]
    fn append_only_writer_round_trips_frames_and_reports_offset() {
        let path = std::env::temp_dir().join(format!(
            "causativelens-spool-writer-{}.clspool",
            std::process::id()
        ));
        let frame = SpoolFrame {
            schema_v: 1,
            flags: 0,
            payload: b"event".to_vec(),
            raw_len: 5,
        };
        let mut writer = SpoolWriter::open(&path).expect("open writer");
        assert_eq!(writer.append(&frame).expect("append frame"), 0);
        writer.sync().expect("sync frame");
        assert_eq!(writer.offset(), frame.encode().len() as u64);
        let scan = recover_file(&path).expect("recover file");
        assert_eq!(scan.frames, vec![frame]);
        let _ = std::fs::remove_file(path);
    }
}
mod session;
pub use session::{SessionStore, StoreError};

#[cfg(test)]
mod session_tests {
    use super::*;
    use clens_types::{
        Assertion, AssertionOp, EventPayload, Expectation, HarnessSpan, IoMode, SysEvent, SysKind,
        TargetPattern,
    };

    #[test]
    fn sqlite_wal_persists_graph_and_why_chain() {
        let path =
            std::env::temp_dir().join(format!("causativelens-v020-{}.cllens", std::process::id()));
        let store = SessionStore::open(&path).expect("store");
        let assertion = Assertion {
            schema_v: 1,
            id: 7,
            span: HarnessSpan {
                step: 2,
                byte_start: 0,
                byte_end: 10,
            },
            op: AssertionOp::Read,
            target: TargetPattern::Path("config.yaml".into()),
            io_mode: IoMode::ReadOnly,
            expect: Expectation::MustHappen,
            order_key: None,
            confidence: 1.0,
            raw_quote: "read config.yaml".into(),
        };
        store
            .insert_assertion(&assertion, "rule")
            .expect("assertion");
        let event = SysEvent {
            schema_v: 1,
            seq: 9,
            ts_qpc: 100,
            pid: 10,
            tid: 11,
            kind: SysKind::FileRead,
            is_rundown: false,
            payload: EventPayload::File {
                path: 1,
                desired_access: 0,
                io_len: Some(4),
                write_attrs: false,
            },
        };
        store
            .insert_event(&event, Some("config.yaml"))
            .expect("event");
        store
            .insert_match(7, 9, 0.9, &serde_json::json!({"path":1.0}), "Fulfilled")
            .expect("evidence");
        let result = store.query_why("config.yaml").expect("why");
        assert_eq!(result["chain"]["trigger_assertion"]["id"], 7);
        assert_eq!(result["chain"]["evidence_events"][0]["event_seq"], 9);
        drop(store);
        let reopened = SessionStore::open(&path).expect("reopen");
        assert_eq!(reopened.counts().expect("counts"), (1, 1, 1));
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("cllens-wal"));
        let _ = std::fs::remove_file(path.with_extension("cllens-shm"));
    }
}
