use crate::GRAPH_SCHEMA_V1;
use clens_types::{Assertion, EventPayload, SysEvent, TargetPattern};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct SessionStore {
    connection: Connection,
    path: PathBuf,
}

/// Location of the complete event payload in the append-only spool. SQLite
/// stores this pointer and the searchable metadata only; it never stores the
/// serialized `SysEvent` payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpoolLocation {
    pub block_id: u64,
    pub block_offset: u64,
}

#[derive(Debug, Clone)]
pub struct EventWrite {
    pub event: SysEvent,
    pub path: Option<String>,
    pub spool: Option<SpoolLocation>,
}

#[derive(Debug, Clone)]
pub struct AssertionWrite {
    pub assertion: Assertion,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct MatchWrite {
    pub assertion_id: u64,
    pub event_seq: u64,
    pub score: f32,
    pub components: Value,
    pub state: String,
}

/// A single SQLite commit unit. The storage boundary owns the transaction so
/// callers cannot accidentally persist one event with autocommit and another
/// event later in a separate transaction.
#[derive(Debug, Default, Clone)]
pub struct StorageBatch {
    pub events: Vec<EventWrite>,
    pub assertions: Vec<AssertionWrite>,
    pub matches: Vec<MatchWrite>,
}

impl StorageBatch {
    pub fn is_empty(&self) -> bool {
        self.events.is_empty() && self.assertions.is_empty() && self.matches.is_empty()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

impl SessionStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                StoreError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
            })?;
        }
        let mut connection = Connection::open(&path)?;
        connection.execute_batch(GRAPH_SCHEMA_V1)?;
        add_column_if_missing(
            &connection,
            "evidence",
            "state",
            "TEXT NOT NULL DEFAULT 'partial'",
        )?;
        migrate_legacy_event_payload(&mut connection)?;
        Ok(Self { connection, path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write_batch(&mut self, batch: StorageBatch) -> Result<(), StoreError> {
        if batch.is_empty() {
            return Ok(());
        }
        let transaction = self.connection.transaction()?;
        for event in &batch.events {
            insert_event_meta(&transaction, event)?;
        }
        for assertion in &batch.assertions {
            insert_assertion(&transaction, assertion)?;
        }
        for matched in &batch.matches {
            insert_match(&transaction, matched)?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn counts(&self) -> Result<(u64, u64, u64), StoreError> {
        let count = |table: &str| -> Result<u64, StoreError> {
            Ok(self
                .connection
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                    row.get::<_, i64>(0)
                })?
                .max(0) as u64)
        };
        Ok((
            count("assertions")?,
            count("event_meta")?,
            count("evidence")?,
        ))
    }

    pub fn query_why(&self, path: &str) -> Result<Value, StoreError> {
        let pattern = format!("%{}%", path.replace('\\', "/"));
        let assertion: Option<(i64, i64, String, String, String)> = self.connection.query_row(
            "SELECT a.id,a.step,a.op,a.target,a.raw_quote FROM assertions a WHERE lower(a.target) LIKE lower(?1) ORDER BY a.step LIMIT 1",
            params![pattern],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        ).optional()?;
        let Some((id, step, op, target, raw_quote)) = assertion else {
            return Ok(
                serde_json::json!({"query":"why","mode":"observe","entity":{"path":path},"chain":{"trigger_assertion":null,"evidence_events":[],"verdict":null,"reverse_attribution":[],"confidence_health":{"events_seen":0,"note":"no persisted assertion"}}}),
            );
        };
        let mut statement = self.connection.prepare("SELECT e.event_seq,e.score,e.state,m.ts_qpc,m.pid,m.tid,m.kind,m.path_id FROM evidence e LEFT JOIN event_meta m ON m.seq=e.event_seq WHERE e.assertion_id=?1 ORDER BY e.score DESC")?;
        let rows = statement.query_map(params![id], |row| {
            Ok(serde_json::json!({"event_seq":row.get::<_,i64>(0)?,"score":row.get::<_,f32>(1)?,"state":row.get::<_,String>(2)?,"ts_qpc":row.get::<_,Option<i64>>(3)?,"pid":row.get::<_,Option<i64>>(4)?,"tid":row.get::<_,Option<i64>>(5)?,"operation":row.get::<_,Option<String>>(6)?,"path_id":row.get::<_,Option<i64>>(7)?}))
        })?.collect::<Result<Vec<_>, _>>()?;
        let state = rows
            .first()
            .and_then(|row| row.get("state"))
            .cloned()
            .unwrap_or_else(|| Value::String("unobserved".into()));
        Ok(
            serde_json::json!({"query":"why","mode":"observe","entity":{"path":path},"chain":{"trigger_assertion":{"id":id,"step":step,"op":op,"target":target,"raw_quote":raw_quote},"evidence_events":rows,"verdict":{"state":state},"reverse_attribution":[],"confidence_health":{"events_seen":self.counts()?.1,"note":"persisted SQLite evidence"}}}),
        )
    }
}

fn add_column_if_missing(
    connection: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), StoreError> {
    let exists = connection
        .prepare(&format!("PRAGMA table_info({table})"))?
        .query_map([], |row| row.get::<_, String>(1))?
        .any(|result| result.is_ok_and(|value| value == column));
    if !exists {
        connection.execute_batch(&format!(
            "ALTER TABLE {table} ADD COLUMN {column} {definition}"
        ))?;
    }
    Ok(())
}

fn migrate_legacy_event_payload(connection: &mut Connection) -> Result<(), StoreError> {
    if !table_has_column(connection, "event_meta", "payload_json")? {
        return Ok(());
    }
    let transaction = connection.transaction()?;
    transaction.execute_batch(
        "CREATE TABLE event_meta_without_payload (
            seq INTEGER PRIMARY KEY,
            schema_v INTEGER NOT NULL,
            ts_qpc INTEGER,
            pid INTEGER,
            tid INTEGER,
            kind TEXT,
            is_rundown INTEGER NOT NULL DEFAULT 0,
            block_id INTEGER,
            block_off INTEGER,
            path_id INTEGER,
            remote TEXT
        ) WITHOUT ROWID;
        INSERT INTO event_meta_without_payload
            (seq,schema_v,ts_qpc,pid,tid,kind,is_rundown,block_id,block_off,path_id,remote)
            SELECT seq,schema_v,ts_qpc,pid,tid,kind,is_rundown,block_id,block_off,path_id,remote
            FROM event_meta;
        DROP TABLE event_meta;
        ALTER TABLE event_meta_without_payload RENAME TO event_meta;
        DELETE FROM event_search;",
    )?;
    transaction.commit()?;
    Ok(())
}

fn table_has_column(
    connection: &Connection,
    table: &str,
    column: &str,
) -> Result<bool, StoreError> {
    Ok(connection
        .prepare(&format!("PRAGMA table_info({table})"))?
        .query_map([], |row| row.get::<_, String>(1))?
        .any(|result| result.is_ok_and(|value| value == column)))
}

fn insert_event_meta(transaction: &Transaction<'_>, record: &EventWrite) -> Result<(), StoreError> {
    let event = &record.event;
    if let (EventPayload::File { path: path_id, .. }, Some(value)) = (&event.payload, &record.path)
    {
        transaction.execute(
            "INSERT INTO paths(id,path) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET path=excluded.path",
            params![i64::from(*path_id), value],
        )?;
    }
    let (block_id, block_off) = record
        .spool
        .map(|location| {
            (
                Some(i64::try_from(location.block_id).unwrap_or(i64::MAX)),
                Some(i64::try_from(location.block_offset).unwrap_or(i64::MAX)),
            )
        })
        .unwrap_or((None, None));
    let seq = i64::try_from(event.seq).unwrap_or(i64::MAX);
    let remote = remote_for_event(event);
    transaction.execute(
        "INSERT INTO event_meta(seq,schema_v,ts_qpc,pid,tid,kind,is_rundown,block_id,block_off,path_id,remote) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(seq) DO NOTHING",
        params![
            seq,
            i64::from(event.schema_v),
            i64::try_from(event.ts_qpc).unwrap_or(i64::MAX),
            i64::from(event.pid),
            i64::from(event.tid),
            format!("{:?}", event.kind),
            i64::from(u8::from(event.is_rundown)),
            block_id,
            block_off,
            match event.payload {
                EventPayload::File { path, .. } => Some(i64::from(path)),
                _ => None,
            },
            remote.as_deref(),
        ],
    )?;
    transaction.execute("DELETE FROM event_search WHERE seq=?1", params![seq])?;
    transaction.execute(
        "INSERT INTO event_search(seq,text,path,remote) VALUES(?1,?2,?3,?4)",
        params![
            seq,
            event_search_text(event, record.path.as_deref(), remote.as_deref()),
            record.path,
            remote
        ],
    )?;
    Ok(())
}

fn insert_assertion(
    transaction: &Transaction<'_>,
    record: &AssertionWrite,
) -> Result<(), StoreError> {
    let assertion = &record.assertion;
    transaction.execute(
        "INSERT INTO assertions(id,schema_v,step,span_json,op,target,io_mode,expect,order_key,confidence,raw_quote,source) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12) ON CONFLICT(id) DO UPDATE SET confidence=excluded.confidence",
        params![
            i64::try_from(assertion.id).unwrap_or(i64::MAX),
            i64::from(assertion.schema_v),
            i64::try_from(assertion.span.step).unwrap_or(i64::MAX),
            serde_json::to_string(&assertion.span)?,
            format!("{:?}", assertion.op),
            target_text(&assertion.target),
            format!("{:?}", assertion.io_mode),
            format!("{:?}", assertion.expect),
            serde_json::to_string(&assertion.order_key)?,
            assertion.confidence,
            assertion.raw_quote.as_ref(),
            record.source,
        ],
    )?;
    Ok(())
}

fn insert_match(transaction: &Transaction<'_>, matched: &MatchWrite) -> Result<(), StoreError> {
    transaction.execute(
        "INSERT INTO evidence(assertion_id,event_seq,score,components_json,state) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(assertion_id,event_seq) DO UPDATE SET score=excluded.score,components_json=excluded.components_json,state=excluded.state",
        params![
            i64::try_from(matched.assertion_id).unwrap_or(i64::MAX),
            i64::try_from(matched.event_seq).unwrap_or(i64::MAX),
            matched.score,
            matched.components.to_string(),
            matched.state,
        ],
    )?;
    transaction.execute(
        "INSERT INTO verdicts(assertion_id,schema_v,state,decided_at) VALUES(?1,1,?2,strftime('%s','now')) ON CONFLICT(assertion_id) DO UPDATE SET state=excluded.state,decided_at=excluded.decided_at",
        params![
            i64::try_from(matched.assertion_id).unwrap_or(i64::MAX),
            matched.state,
        ],
    )?;
    Ok(())
}

fn target_text(target: &TargetPattern) -> String {
    match target {
        TargetPattern::Path(value) => value.to_string(),
        TargetPattern::RegistryKey(value) => value.to_string(),
        TargetPattern::Endpoint { host, port, proto } => format!("{}:{:?}:{:?}", host, port, proto),
        TargetPattern::FreeText(value) => value.to_string(),
    }
}

fn remote_for_event(event: &SysEvent) -> Option<String> {
    match &event.payload {
        EventPayload::Net { remote, .. } => Some(format!("{}:{}", remote.host, remote.port)),
        _ => None,
    }
}

fn event_search_text(event: &SysEvent, path: Option<&str>, remote: Option<&str>) -> String {
    let kind = format!("{:?}", event.kind);
    match &event.payload {
        EventPayload::Proc { image, cmd, .. } => format!("{kind} {image} {cmd}"),
        EventPayload::File { .. } => format!("{kind} {}", path.unwrap_or_default()),
        EventPayload::Net { .. } => format!("{kind} {}", remote.unwrap_or_default()),
        EventPayload::Reg { key, .. } => format!("{kind} {key}"),
        EventPayload::None => kind,
    }
}
