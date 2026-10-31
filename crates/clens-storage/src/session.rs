use crate::GRAPH_SCHEMA_V1;
use clens_types::{Assertion, EventPayload, SysEvent, TargetPattern};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct SessionStore {
    connection: Connection,
    path: PathBuf,
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
        let connection = Connection::open(&path)?;
        connection.execute_batch(GRAPH_SCHEMA_V1)?;
        add_column_if_missing(&connection, "event_meta", "payload_json", "TEXT")?;
        add_column_if_missing(
            &connection,
            "evidence",
            "state",
            "TEXT NOT NULL DEFAULT 'partial'",
        )?;
        Ok(Self { connection, path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn insert_path(&self, id: u32, path: &str) -> Result<(), StoreError> {
        self.connection.execute(
            "INSERT INTO paths(id,path) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET path=excluded.path",
            params![i64::from(id), path],
        )?;
        Ok(())
    }

    pub fn insert_event(&self, event: &SysEvent, path: Option<&str>) -> Result<(), StoreError> {
        if let (EventPayload::File { path: path_id, .. }, Some(value)) = (&event.payload, path) {
            self.insert_path(*path_id, value)?;
        }
        let payload = serde_json::to_string(&event.payload)?;
        self.connection.execute(
            "INSERT INTO event_meta(seq,schema_v,ts_qpc,pid,tid,kind,is_rundown,path_id,remote,payload_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10) ON CONFLICT(seq) DO NOTHING",
            params![
                i64::try_from(event.seq).unwrap_or(i64::MAX),
                i64::from(event.schema_v),
                i64::try_from(event.ts_qpc).unwrap_or(i64::MAX),
                i64::from(event.pid),
                i64::from(event.tid),
                format!("{:?}", event.kind),
                i64::from(u8::from(event.is_rundown)),
                match event.payload { EventPayload::File { path, .. } => Some(i64::from(path)), _ => None },
                remote_for_event(event),
                payload,
            ],
        )?;
        self.connection.execute(
            "DELETE FROM event_search WHERE seq=?1",
            params![i64::try_from(event.seq).unwrap_or(i64::MAX)],
        )?;
        self.connection.execute(
            "INSERT INTO event_search(seq,text,path,remote) VALUES(?1,?2,?3,?4)",
            params![
                i64::try_from(event.seq).unwrap_or(i64::MAX),
                payload,
                path,
                remote_for_event(event)
            ],
        )?;
        Ok(())
    }

    pub fn insert_assertion(&self, assertion: &Assertion, source: &str) -> Result<(), StoreError> {
        self.connection.execute(
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
                source,
            ],
        )?;
        Ok(())
    }

    pub fn insert_match(
        &self,
        assertion_id: u64,
        event_seq: u64,
        score: f32,
        components: &Value,
        state: &str,
    ) -> Result<(), StoreError> {
        self.connection.execute(
            "INSERT INTO evidence(assertion_id,event_seq,score,components_json,state) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(assertion_id,event_seq) DO UPDATE SET score=excluded.score,components_json=excluded.components_json,state=excluded.state",
            params![i64::try_from(assertion_id).unwrap_or(i64::MAX), i64::try_from(event_seq).unwrap_or(i64::MAX), score, components.to_string(), state],
        )?;
        self.connection.execute(
            "INSERT INTO verdicts(assertion_id,schema_v,state,decided_at) VALUES(?1,1,?2,strftime('%s','now')) ON CONFLICT(assertion_id) DO UPDATE SET state=excluded.state,decided_at=excluded.decided_at",
            params![i64::try_from(assertion_id).unwrap_or(i64::MAX), state],
        )?;
        Ok(())
    }

    pub fn commit(&self) -> Result<(), StoreError> {
        self.connection
            .execute_batch("PRAGMA wal_checkpoint(PASSIVE);")?;
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
