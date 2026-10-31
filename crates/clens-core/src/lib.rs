use clens_types::{
    Assertion, AssertionOp, EventPayload, EvidenceEdge, Expectation, Fulfilment, NetProto,
    SysEvent, SysKind, TargetPattern, Verdict,
};
pub use clens_types::{ScoreComponents, Watermark};
use std::collections::HashMap;

pub fn crate_has_no_io_boundary() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlignmentProfile {
    pub process_weight: f32,
    pub time_weight: f32,
    pub semantic_weight: f32,
    pub path_weight: f32,
    pub time_center_window_ms: i64,
    pub fulfilled_threshold: f32,
    pub partial_threshold: f32,
    pub default_window_ms: i64,
    pub max_probe_window_ms: i64,
    pub disorder_window_ms: i64,
    pub assertion_lag_ms: i64,
    pub batch_window_ms: i64,
}
impl Default for AlignmentProfile {
    fn default() -> Self {
        Self {
            process_weight: 0.25,
            time_weight: 0.25,
            semantic_weight: 0.25,
            path_weight: 0.25,
            time_center_window_ms: 2_000,
            fulfilled_threshold: 0.75,
            partial_threshold: 0.4,
            default_window_ms: 2_000,
            max_probe_window_ms: 30_000,
            disorder_window_ms: 64,
            assertion_lag_ms: 1_000,
            batch_window_ms: 200,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimedAssertion<'a> {
    pub assertion: &'a Assertion,
    pub center_ts_ms: i64,
    pub window_end_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeWindow {
    pub start_ms: i64,
    pub end_ms: i64,
}

/// Build a bounded expectation window from an optional deadline and the
/// observed gap to the next Harness step. A zero/negative gap uses the
/// profile default; asynchronous tools may probe up to the configured cap.
pub fn derive_time_window(
    center_ms: i64,
    declared_deadline_ms: Option<i64>,
    next_step_gap_ms: Option<i64>,
    profile: AlignmentProfile,
) -> TimeWindow {
    let dynamic = next_step_gap_ms.unwrap_or(profile.default_window_ms).max(0);
    let probe = dynamic.max(profile.default_window_ms);
    let deadline = declared_deadline_ms.unwrap_or(probe).max(0);
    let end_offset = deadline.min(profile.max_probe_window_ms.max(profile.default_window_ms));
    TimeWindow {
        start_ms: center_ms,
        end_ms: center_ms.saturating_add(end_offset),
    }
}

pub fn derive_assertion_window(
    assertion: &Assertion,
    center_ms: i64,
    next_step_gap_ms: Option<i64>,
    session_end_ms: Option<i64>,
    profile: AlignmentProfile,
) -> TimeWindow {
    if assertion.expect == Expectation::MustNotHappen {
        TimeWindow {
            start_ms: center_ms,
            end_ms: session_end_ms.unwrap_or(i64::MAX),
        }
    } else {
        derive_time_window(center_ms, None, next_step_gap_ms, profile)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerdictState {
    Fulfilled,
    Partial,
    Unobserved,
    Deviated,
    Unrelated,
}
#[derive(Debug, Clone, PartialEq)]
pub struct AlignmentMatch {
    pub assertion_id: u64,
    pub event_seq: u64,
    pub score: ScoreComponents,
    pub state: VerdictState,
}

pub fn score_candidate(
    input: TimedAssertion<'_>,
    event: &SysEvent,
    process_distance: u8,
    semantic: f32,
    path: f32,
    profile: AlignmentProfile,
) -> ScoreComponents {
    score_candidate_at(
        input,
        event,
        event.ts_qpc as i64,
        process_distance,
        semantic,
        path,
        profile,
    )
}

/// Score a candidate when the caller has already projected ETW QPC into the
/// Harness UTC coordinate. The legacy `score_candidate` wrapper intentionally
/// keeps fixture compatibility by treating `ts_qpc` as the test coordinate.
pub fn score_candidate_at(
    input: TimedAssertion<'_>,
    _event: &SysEvent,
    event_ts_ms: i64,
    process_distance: u8,
    semantic: f32,
    path: f32,
    profile: AlignmentProfile,
) -> ScoreComponents {
    let process = if process_distance == u8::MAX {
        0.0
    } else if process_distance == 0 {
        1.0
    } else {
        0.8_f32.powi(i32::from(process_distance))
    };
    let delta = event_ts_ms
        .saturating_sub(input.center_ts_ms)
        .unsigned_abs();
    let center_window = profile.time_center_window_ms.max(1) as u64;
    let time = if delta <= center_window {
        1.0 - (delta as f32 / center_window as f32)
    } else {
        0.0
    };
    let total = (profile.process_weight * process
        + profile.time_weight * time
        + profile.semantic_weight * semantic
        + profile.path_weight * path)
        .clamp(0.0, 1.0);
    ScoreComponents {
        process,
        time,
        semantic: semantic.clamp(0.0, 1.0),
        path: path.clamp(0.0, 1.0),
        total,
    }
}
pub fn classify_score(score: ScoreComponents, profile: AlignmentProfile) -> VerdictState {
    if score.total >= profile.fulfilled_threshold {
        VerdictState::Fulfilled
    } else if score.total >= profile.partial_threshold {
        VerdictState::Partial
    } else {
        VerdictState::Unrelated
    }
}
pub fn align_candidate(
    input: TimedAssertion<'_>,
    event: &SysEvent,
    process_distance: u8,
    semantic: f32,
    path: f32,
    profile: AlignmentProfile,
) -> AlignmentMatch {
    let score = score_candidate(input, event, process_distance, semantic, path, profile);
    AlignmentMatch {
        assertion_id: input.assertion.id,
        event_seq: event.seq,
        state: classify_score(score, profile),
        score,
    }
}

pub fn align_candidate_at(
    input: TimedAssertion<'_>,
    event: &SysEvent,
    event_ts_ms: i64,
    process_distance: u8,
    semantic: f32,
    path: f32,
    profile: AlignmentProfile,
) -> AlignmentMatch {
    let score = score_candidate_at(
        input,
        event,
        event_ts_ms,
        process_distance,
        semantic,
        path,
        profile,
    );
    AlignmentMatch {
        assertion_id: input.assertion.id,
        event_seq: event.seq,
        state: classify_score(score, profile),
        score,
    }
}

const GENERIC_READ: u32 = 0x8000_0000;
const GENERIC_WRITE: u32 = 0x4000_0000;
const FILE_WRITE_DATA: u32 = 0x0000_0002;
const FILE_APPEND_DATA: u32 = 0x0000_0004;

/// Static semantic template table from blueprint table 10. The function is
/// pure and deliberately does not inspect file contents or call Windows APIs.
pub fn semantic_score(assertion: &Assertion, event: &SysEvent) -> f32 {
    match assertion.op {
        AssertionOp::Read => match event.kind {
            SysKind::FileRead => 1.0,
            SysKind::FileCreate => match &event.payload {
                EventPayload::File { desired_access, .. }
                    if desired_access & (GENERIC_READ | 0x0001_0000) != 0 =>
                {
                    1.0
                }
                _ => 0.0,
            },
            SysKind::FileSetInfo => 0.7,
            SysKind::RegOpen | SysKind::RegQuery => 0.7,
            _ => 0.0,
        },
        AssertionOp::Write => match event.kind {
            SysKind::FileWrite => 1.0,
            SysKind::FileCreate => match &event.payload {
                EventPayload::File { desired_access, .. }
                    if desired_access & (GENERIC_WRITE | FILE_WRITE_DATA | FILE_APPEND_DATA)
                        != 0 =>
                {
                    1.0
                }
                _ => 0.0,
            },
            SysKind::FileSetInfo => 0.7,
            SysKind::RegSet => 0.7,
            _ => 0.0,
        },
        AssertionOp::Delete => match event.kind {
            SysKind::FileDelete | SysKind::FileSetInfo => 1.0,
            SysKind::RegDelete => 0.7,
            _ => 0.0,
        },
        AssertionOp::Rename => match event.kind {
            SysKind::FileRename | SysKind::FileSetInfo => 1.0,
            _ => 0.0,
        },
        AssertionOp::Exec => match event.kind {
            SysKind::ProcStart | SysKind::ImageLoad => 1.0,
            _ => 0.0,
        },
        AssertionOp::Connect => match event.kind {
            SysKind::NetConnect => 1.0,
            SysKind::NetAccept | SysKind::NetSend | SysKind::NetRecv => 0.7,
            _ => 0.0,
        },
        AssertionOp::Bind => match event.kind {
            SysKind::NetBind => 1.0,
            SysKind::NetListen => 0.7,
            _ => 0.0,
        },
        AssertionOp::Listen => match event.kind {
            SysKind::NetListen => 1.0,
            SysKind::NetBind => 0.7,
            _ => 0.0,
        },
        AssertionOp::RegRead => match event.kind {
            SysKind::RegOpen | SysKind::RegQuery => 1.0,
            _ => 0.0,
        },
        AssertionOp::RegWrite => match event.kind {
            SysKind::RegSet | SysKind::RegDelete => 1.0,
            _ => 0.0,
        },
        AssertionOp::Create => match event.kind {
            SysKind::FileCreate => 1.0,
            SysKind::FileSetInfo => 0.7,
            _ => 0.0,
        },
        AssertionOp::Constraint
        | AssertionOp::AssertPrecondition
        | AssertionOp::AssertPostcondition
        | AssertionOp::RollbackPlan => 0.0,
    }
}

pub fn is_constraint_conflict(assertion: &Assertion, event: &SysEvent) -> bool {
    if assertion.op != AssertionOp::Constraint || assertion.io_mode != clens_types::IoMode::ReadOnly
    {
        return false;
    }
    matches!(
        event.kind,
        SysKind::FileWrite
            | SysKind::FileDelete
            | SysKind::FileRename
            | SysKind::FileSetInfo
            | SysKind::RegSet
            | SysKind::RegDelete
            | SysKind::NetSend
    )
}

fn glob_matches(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let mut row = vec![false; value.len() + 1];
    row[0] = true;
    for pattern_byte in pattern {
        let mut next = vec![false; value.len() + 1];
        if *pattern_byte == b'*' {
            next[0] = row[0];
            for index in 1..=value.len() {
                next[index] = next[index - 1] || row[index];
            }
        } else {
            for index in 1..=value.len() {
                if *pattern_byte == b'?' || *pattern_byte == value[index - 1] {
                    next[index] = row[index - 1];
                }
            }
        }
        row = next;
    }
    row[value.len()]
}

pub fn path_match_score(target: &TargetPattern, actual: &str) -> f32 {
    let actual = clens_types::normalize_path_pattern(actual);
    match target {
        TargetPattern::Path(pattern) => {
            let pattern = clens_types::normalize_path_pattern(pattern);
            if pattern == actual {
                1.0
            } else if pattern.eq_ignore_ascii_case(&actual) {
                0.9
            } else if actual
                .to_ascii_lowercase()
                .strip_prefix(&pattern.to_ascii_lowercase())
                .is_some_and(|rest| rest.starts_with('/'))
            {
                0.85
            } else if pattern.contains('*') || pattern.contains('?') {
                if glob_matches(&pattern.to_ascii_lowercase(), &actual.to_ascii_lowercase()) {
                    0.6
                } else {
                    0.0
                }
            } else {
                0.0
            }
        }
        TargetPattern::FreeText(text)
            if actual
                .to_ascii_lowercase()
                .contains(&text.to_ascii_lowercase()) =>
        {
            0.6
        }
        TargetPattern::FreeText(_) => 0.0,
        _ => 0.0,
    }
}

pub fn endpoint_match_score(
    target: &TargetPattern,
    host: &str,
    port: u16,
    proto: &NetProto,
) -> f32 {
    let TargetPattern::Endpoint {
        host: expected_host,
        port: expected_port,
        proto: expected_proto,
    } = target
    else {
        return 0.0;
    };
    let host_matches = if expected_host.as_ref() == host {
        1.0
    } else if expected_host.eq_ignore_ascii_case(host) {
        0.9
    } else if glob_matches(
        &expected_host.to_ascii_lowercase(),
        &host.to_ascii_lowercase(),
    ) {
        0.6
    } else {
        0.0
    };
    let port_matches = expected_port.is_none_or(|expected| expected == port);
    let proto_matches = expected_proto == proto;
    if host_matches > 0.0 && port_matches && proto_matches {
        host_matches
    } else {
        0.0
    }
}

pub fn event_path_score(
    assertion: &Assertion,
    event: &SysEvent,
    paths: &HashMap<u32, String>,
) -> f32 {
    match (&assertion.target, &event.payload) {
        (TargetPattern::Path(_), EventPayload::File { path, .. }) => paths
            .get(path)
            .map_or(0.0, |actual| path_match_score(&assertion.target, actual)),
        (TargetPattern::RegistryKey(expected), EventPayload::Reg { key, .. }) => {
            if expected == key {
                1.0
            } else if expected.eq_ignore_ascii_case(key) {
                0.9
            } else {
                0.0
            }
        }
        (TargetPattern::Endpoint { .. }, EventPayload::Net { proto, remote, .. }) => {
            endpoint_match_score(&assertion.target, &remote.host, remote.port, proto)
        }
        _ => 0.0,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessTree {
    parents: HashMap<u32, u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributionState {
    Anchored,
    Review,
    Unmatched,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProcessAttribution {
    pub pid: u32,
    pub score: f32,
    pub state: AttributionState,
}

fn command_tokens(value: &str) -> std::collections::HashSet<String> {
    value
        .split_whitespace()
        .map(|token| token.trim_matches(['\'', '"']).to_ascii_lowercase())
        .filter(|token| !token.is_empty())
        .collect()
}

pub fn token_jaccard(left: &str, right: &str) -> f32 {
    let left = command_tokens(left);
    let right = command_tokens(right);
    if left.is_empty() && right.is_empty() {
        return 1.0;
    }
    let intersection = left.intersection(&right).count() as f32;
    let union = left.union(&right).count() as f32;
    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

pub fn attribute_child_process(
    root_pid: u32,
    assertion: &Assertion,
    event: &SysEvent,
    event_ts_ms: i64,
    assertion_ts_ms: i64,
    _process_tree: &ProcessTree,
) -> Option<ProcessAttribution> {
    let EventPayload::Proc { ppid, cmd, .. } = &event.payload else {
        return None;
    };
    if event.kind != SysKind::ProcStart || *ppid != root_pid {
        return None;
    }
    let delta = event_ts_ms.saturating_sub(assertion_ts_ms).unsigned_abs();
    if delta > 2_000 {
        return None;
    }
    let score = token_jaccard(&assertion.raw_quote, cmd);
    let state = if score >= 0.5 {
        AttributionState::Anchored
    } else if score >= 0.2 {
        AttributionState::Review
    } else {
        AttributionState::Unmatched
    };
    Some(ProcessAttribution {
        pid: event.pid,
        score,
        state,
    })
}

impl ProcessTree {
    pub fn observe_start(&mut self, pid: u32, ppid: u32) {
        if pid != 0 && pid != ppid {
            self.parents.insert(pid, ppid);
        }
    }

    pub fn observe_stop(&mut self, pid: u32) {
        self.parents.remove(&pid);
    }

    pub fn distance(&self, subject_pid: u32, event_pid: u32) -> Option<u8> {
        if subject_pid == event_pid {
            return Some(0);
        }
        let mut current = event_pid;
        for distance in 1..=u8::MAX {
            let parent = *self.parents.get(&current)?;
            if parent == subject_pid {
                return Some(distance);
            }
            if parent == current {
                return None;
            }
            current = parent;
        }
        None
    }

    pub fn nearest_common_ancestor_distance(&self, left: u32, right: u32) -> Option<u8> {
        let mut left_ancestors = HashMap::new();
        let mut current = left;
        for distance in 0..=u8::MAX {
            left_ancestors.insert(current, distance);
            let Some(parent) = self.parents.get(&current).copied() else {
                break;
            };
            if parent == current {
                break;
            }
            current = parent;
        }
        let mut current = right;
        for distance in 0..=u8::MAX {
            if let Some(left_distance) = left_ancestors.get(&current) {
                return Some(left_distance.saturating_add(distance));
            }
            let Some(parent) = self.parents.get(&current).copied() else {
                break;
            };
            if parent == current {
                break;
            }
            current = parent;
        }
        None
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActiveKindIndex {
    slots: HashMap<(u32, SysKind), Vec<u64>>,
}

impl ActiveKindIndex {
    pub fn activate(&mut self, pid: u32, kind: SysKind, assertion_id: u64) {
        let slots = self.slots.entry((pid, kind)).or_default();
        if !slots.contains(&assertion_id) {
            slots.push(assertion_id);
        }
    }

    pub fn deactivate(&mut self, pid: u32, kind: SysKind, assertion_id: u64) {
        if let Some(slots) = self.slots.get_mut(&(pid, kind)) {
            slots.retain(|candidate| *candidate != assertion_id);
            if slots.is_empty() {
                self.slots.remove(&(pid, kind));
            }
        }
    }

    pub fn candidates(&self, pid: u32, kind: SysKind) -> &[u64] {
        self.slots.get(&(pid, kind)).map_or(&[], Vec::as_slice)
    }
}

pub fn compute_watermark(
    max_system_event_ms: i64,
    current_wall_ms: i64,
    profile: AlignmentProfile,
) -> Watermark {
    Watermark {
        system_wall_ms: max_system_event_ms,
        now_ms: current_wall_ms,
        disorder_ms: profile.disorder_window_ms.max(0),
        assertion_lag_ms: profile.assertion_lag_ms.max(0),
    }
}

pub fn watermark_value_ms(watermark: Watermark) -> i64 {
    watermark
        .system_wall_ms
        .saturating_sub(watermark.disorder_ms)
        .min(watermark.now_ms.saturating_sub(watermark.assertion_lag_ms))
}

pub fn is_window_closed(watermark: Watermark, window_end_ms: i64) -> bool {
    window_end_ms < watermark_value_ms(watermark)
}

#[derive(Debug)]
struct ActiveAssertion {
    assertion: Assertion,
    center_ts_ms: i64,
    window_end_ms: i64,
    best: Option<AlignmentMatch>,
    deviated: Option<AlignmentMatch>,
}

/// Single-threaded, in-memory alignment loop. It owns no file handles or
/// SQLite connections, so replay and fuzz tests can feed identical streams and
/// obtain identical updates.
#[derive(Debug)]
pub struct AlignmentEngine {
    profile: AlignmentProfile,
    active: HashMap<u64, ActiveAssertion>,
    events: Vec<(SysEvent, i64)>,
    updates: Vec<AlignmentMatch>,
    watermark_ms: i64,
}

impl AlignmentEngine {
    pub fn new(profile: AlignmentProfile) -> Self {
        Self {
            profile,
            active: HashMap::new(),
            events: Vec::new(),
            updates: Vec::new(),
            watermark_ms: i64::MIN,
        }
    }

    pub fn profile(&self) -> AlignmentProfile {
        self.profile
    }

    pub fn activate(&mut self, assertion: Assertion, center_ts_ms: i64, window_end_ms: i64) {
        let id = assertion.id;
        let mut active = ActiveAssertion {
            assertion,
            center_ts_ms,
            window_end_ms,
            best: None,
            deviated: None,
        };
        for (event, event_ts_ms) in &self.events {
            Self::consider_event(&mut active, event, *event_ts_ms, self.profile, 0, 1.0);
        }
        self.active.insert(id, active);
    }

    pub fn ingest_event(&mut self, event: SysEvent, event_ts_ms: i64) {
        self.ingest_event_with_context(event, event_ts_ms, &HashMap::new(), &HashMap::new());
    }

    pub fn ingest_event_with_context(
        &mut self,
        event: SysEvent,
        event_ts_ms: i64,
        process_distances: &HashMap<u64, u8>,
        path_scores: &HashMap<u64, f32>,
    ) {
        self.events.push((event, event_ts_ms));
        let Some((event, event_ts_ms)) = self
            .events
            .last()
            .map(|(event, timestamp)| (event, *timestamp))
        else {
            return;
        };
        for (id, active) in &mut self.active {
            let process_distance = process_distances.get(id).copied().unwrap_or(0);
            let path_score = path_scores.get(id).copied().unwrap_or({
                if matches!(active.assertion.target, TargetPattern::Path(_)) {
                    0.0
                } else {
                    1.0
                }
            });
            Self::consider_event(
                active,
                event,
                event_ts_ms,
                self.profile,
                process_distance,
                path_score,
            );
            if let Some(deviated) = active.deviated.as_ref() {
                self.updates.push(deviated.clone());
            } else if let Some(best) = active.best.as_ref() {
                self.updates.push(best.clone());
            }
        }
    }

    fn consider_event(
        active: &mut ActiveAssertion,
        event: &SysEvent,
        event_ts_ms: i64,
        profile: AlignmentProfile,
        process_distance: u8,
        path_score: f32,
    ) {
        let conflict = is_constraint_conflict(&active.assertion, event);
        let semantic = if conflict {
            1.0
        } else {
            semantic_score(&active.assertion, event)
        };
        if semantic == 0.0 {
            return;
        }
        let input = TimedAssertion {
            assertion: &active.assertion,
            center_ts_ms: active.center_ts_ms,
            window_end_ms: active.window_end_ms,
        };
        let mut matched = align_candidate_at(
            input,
            event,
            event_ts_ms,
            process_distance,
            semantic,
            path_score,
            profile,
        );
        if active.assertion.expect == Expectation::MustNotHappen
            && !matches!(matched.state, VerdictState::Unrelated)
            && (conflict || matched.score.total >= profile.partial_threshold)
        {
            matched.state = VerdictState::Deviated;
            active.deviated = Some(matched);
            return;
        }
        if !matches!(matched.state, VerdictState::Unrelated)
            && active
                .best
                .as_ref()
                .is_none_or(|best| matched.score.total > best.score.total)
        {
            active.best = Some(matched);
        }
    }

    pub fn advance_watermark(&mut self, watermark: Watermark) -> i64 {
        let current = watermark_value_ms(watermark);
        self.watermark_ms = self.watermark_ms.max(current);
        let watermark_ms = self.watermark_ms;
        let mut closed = Vec::new();
        for (id, active) in &self.active {
            if active.window_end_ms >= watermark_ms {
                continue;
            }
            let update = if let Some(deviated) = active.deviated.clone() {
                Some(deviated)
            } else if let Some(best) = active.best.clone() {
                Some(best)
            } else if active.assertion.expect == Expectation::MustNotHappen {
                Some(AlignmentMatch {
                    assertion_id: active.assertion.id,
                    event_seq: 0,
                    score: ScoreComponents {
                        process: 1.0,
                        time: 1.0,
                        semantic: 1.0,
                        path: 1.0,
                        total: 1.0,
                    },
                    state: VerdictState::Fulfilled,
                })
            } else {
                Some(AlignmentMatch {
                    assertion_id: active.assertion.id,
                    event_seq: 0,
                    score: ScoreComponents {
                        process: 0.0,
                        time: 0.0,
                        semantic: 0.0,
                        path: 0.0,
                        total: 0.0,
                    },
                    state: VerdictState::Unobserved,
                })
            };
            if let Some(update) = update {
                self.updates.push(update);
            }
            closed.push(*id);
        }
        for id in closed {
            self.active.remove(&id);
        }
        self.watermark_ms
    }

    pub fn drain_updates(&mut self) -> Vec<AlignmentMatch> {
        std::mem::take(&mut self.updates)
    }

    pub fn active_count(&self) -> usize {
        self.active.len()
    }
}

/// Convert one candidate match into the serializable public verdict shape.
/// `Unrelated` candidates intentionally return `None` because they do not
/// represent a graph edge or a decided assertion.
pub fn verdict_from_match(matched: &AlignmentMatch, decided_at: Watermark) -> Option<Verdict> {
    let state = match matched.state {
        VerdictState::Fulfilled => Fulfilment::Fulfilled,
        VerdictState::Partial => Fulfilment::Partial,
        VerdictState::Deviated => Fulfilment::Deviated,
        VerdictState::Unobserved | VerdictState::Unrelated => return None,
    };
    Some(Verdict {
        schema_v: 1,
        assertion_id: matched.assertion_id,
        state,
        evidence: vec![EvidenceEdge {
            event_seq: matched.event_seq,
            score: matched.score.total,
            components: matched.score,
        }],
        deviation: None,
        decided_at,
    })
}
pub fn reverse_attribute(assertion: &Assertion, event: &SysEvent) -> bool {
    if is_constraint_conflict(assertion, event) {
        true
    } else {
        semantic_score(assertion, event) >= 0.7
    }
}

/// Causal Weave is CausativeLens' deterministic tie-breaker for evidence that
/// scores similarly under the four blueprint dimensions. It rewards a
/// candidate that closes an assertion's temporal braid (the nearest prior
/// and next observed facts), then applies an explicit contradiction penalty.
/// The function is pure, replayable, and does not inspect the filesystem.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CausalWeaveInput {
    pub base_score: f32,
    pub prior_gap_ms: Option<i64>,
    pub next_gap_ms: Option<i64>,
    pub contradiction: bool,
    pub process_distance: u8,
}

pub fn causal_weave_score(input: CausalWeaveInput) -> f32 {
    let braid = match (input.prior_gap_ms, input.next_gap_ms) {
        (Some(prior), Some(next)) => {
            let span = prior
                .unsigned_abs()
                .saturating_add(next.unsigned_abs())
                .max(1) as f32;
            (1.0 - (prior.unsigned_abs().max(next.unsigned_abs()) as f32 / span)).clamp(0.0, 1.0)
        }
        (Some(gap), None) | (None, Some(gap)) => {
            (1.0 - gap.unsigned_abs() as f32 / 2_000.0).clamp(0.0, 1.0) * 0.5
        }
        (None, None) => 0.0,
    };
    let lineage = if input.process_distance == u8::MAX {
        0.0
    } else {
        1.0 / (f32::from(input.process_distance) + 1.0)
    };
    let contradiction_penalty = if input.contradiction { 0.25 } else { 0.0 };
    (0.72 * input.base_score + 0.18 * braid + 0.10 * lineage - contradiction_penalty)
        .clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clens_types::{EventPayload, Expectation, IoMode, TargetPattern};
    fn assertion() -> Assertion {
        Assertion {
            schema_v: 1,
            id: 7,
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
        }
    }
    fn event() -> SysEvent {
        SysEvent {
            schema_v: 1,
            seq: 9,
            ts_qpc: 1_000,
            pid: 10,
            tid: 11,
            kind: SysKind::FileRead,
            is_rundown: false,
            payload: EventPayload::File {
                path: 1,
                desired_access: 0,
                io_len: Some(1),
                write_attrs: false,
            },
        }
    }

    #[derive(serde::Deserialize)]
    struct Fixture {
        assertion: FixtureAssertion,
        candidate: FixtureCandidate,
        expected: String,
    }
    #[derive(serde::Deserialize)]
    struct FixtureAssertion {
        id: u64,
    }
    #[derive(serde::Deserialize)]
    struct FixtureCandidate {
        event_seq: u64,
        event_ts_ms: u64,
        center_ts_ms: i64,
        process_distance: u8,
        semantic: f32,
        path: f32,
    }

    #[test]
    fn twelve_blueprint_scenarios_match_expected_verdicts() {
        let fixtures = [
            include_str!("../../../fixtures/scenarios/f0001-readonly-compliant.json"),
            include_str!("../../../fixtures/scenarios/f0002-readonly-violation.json"),
            include_str!("../../../fixtures/scenarios/f0003-check-delete-positive.json"),
            include_str!("../../../fixtures/scenarios/f0004-check-delete-negative.json"),
            include_str!("../../../fixtures/scenarios/f0005-rename-chain.json"),
            include_str!("../../../fixtures/scenarios/f0006-child-shell-attribution.json"),
            include_str!("../../../fixtures/scenarios/f0007-bind-connect.json"),
            include_str!("../../../fixtures/scenarios/f0008-rundown-completion.json"),
            include_str!("../../../fixtures/scenarios/f0009-buffer-loss-weak.json"),
            include_str!("../../../fixtures/scenarios/f0010-self-noise-filter.json"),
            include_str!("../../../fixtures/scenarios/f0011-cross-tree-no-attribution.json"),
            include_str!("../../../fixtures/scenarios/f0012-longpath-registry-constraint.json"),
        ];
        for raw in fixtures {
            let fixture: Fixture = serde_json::from_str(raw).expect("valid scenario fixture");
            let a = assertion();
            let mut owned = a;
            owned.id = fixture.assertion.id;
            let mut e = event();
            e.seq = fixture.candidate.event_seq;
            e.ts_qpc = fixture.candidate.event_ts_ms;
            let matched = align_candidate(
                TimedAssertion {
                    assertion: &owned,
                    center_ts_ms: fixture.candidate.center_ts_ms,
                    window_end_ms: 2_000,
                },
                &e,
                fixture.candidate.process_distance,
                fixture.candidate.semantic,
                fixture.candidate.path,
                AlignmentProfile::default(),
            );
            let actual = match matched.state {
                VerdictState::Fulfilled => "fulfilled",
                VerdictState::Partial => "partial",
                _ => "unrelated",
            };
            assert_eq!(actual, fixture.expected);
        }
    }
    #[test]
    fn four_dimension_score_and_thresholds_are_bounded() {
        let a = assertion();
        let m = align_candidate(
            TimedAssertion {
                assertion: &a,
                center_ts_ms: 1_000,
                window_end_ms: 2_000,
            },
            &event(),
            0,
            1.0,
            1.0,
            AlignmentProfile::default(),
        );
        assert_eq!(m.state, VerdictState::Fulfilled);
        assert!((0.0..=1.0).contains(&m.score.total));
    }

    #[test]
    fn fulfilled_candidate_maps_to_public_verdict_with_evidence_edge() {
        let a = assertion();
        let matched = align_candidate(
            TimedAssertion {
                assertion: &a,
                center_ts_ms: 1_000,
                window_end_ms: 2_000,
            },
            &event(),
            0,
            1.0,
            1.0,
            AlignmentProfile::default(),
        );
        let verdict = verdict_from_match(
            &matched,
            Watermark {
                system_wall_ms: 2_000,
                now_ms: 2_000,
                disorder_ms: 0,
                assertion_lag_ms: 0,
            },
        )
        .expect("fulfilled candidate has a public verdict");
        assert_eq!(verdict.state, clens_types::Fulfilment::Fulfilled);
        assert_eq!(verdict.evidence[0].event_seq, event().seq);
    }
    #[test]
    fn watermark_is_monotonic_for_later_system_time() {
        let early = watermark_value_ms(Watermark {
            system_wall_ms: 2_000,
            now_ms: 2_100,
            disorder_ms: 100,
            assertion_lag_ms: 50,
        });
        let late = watermark_value_ms(Watermark {
            system_wall_ms: 3_000,
            now_ms: 3_100,
            disorder_ms: 100,
            assertion_lag_ms: 50,
        });
        assert!(late >= early);
        assert!(is_window_closed(
            Watermark {
                system_wall_ms: 3_000,
                now_ms: 3_100,
                disorder_ms: 100,
                assertion_lag_ms: 50
            },
            2_000
        ));
    }

    #[test]
    fn semantic_templates_cover_delete_setinfo_and_network_protocol() {
        let mut delete_event = event();
        delete_event.kind = SysKind::FileSetInfo;
        assert_eq!(
            semantic_score(
                &Assertion {
                    op: AssertionOp::Delete,
                    ..assertion()
                },
                &delete_event
            ),
            1.0
        );
        let network = SysEvent {
            kind: SysKind::NetConnect,
            payload: EventPayload::Net {
                proto: clens_types::NetProto::Tcp,
                local: clens_types::Endpoint {
                    host: "127.0.0.1".into(),
                    port: 1,
                },
                remote: clens_types::Endpoint {
                    host: "example.com".into(),
                    port: 443,
                },
            },
            ..event()
        };
        let connect = Assertion {
            op: AssertionOp::Connect,
            target: TargetPattern::Endpoint {
                host: "*.com".into(),
                port: Some(443),
                proto: clens_types::NetProto::Tcp,
            },
            ..assertion()
        };
        assert_eq!(semantic_score(&connect, &network), 1.0);
        assert_eq!(event_path_score(&connect, &network, &HashMap::new()), 0.6);
    }

    #[test]
    fn path_matching_respects_exact_case_prefix_and_glob_scores() {
        let actual = r"C:\Work\config.yaml";
        assert_eq!(
            path_match_score(&TargetPattern::Path("C:/Work/config.yaml".into()), actual),
            1.0
        );
        assert_eq!(
            path_match_score(&TargetPattern::Path("c:/work/config.yaml".into()), actual),
            0.9
        );
        assert_eq!(
            path_match_score(&TargetPattern::Path("C:/Work".into()), actual),
            0.85
        );
        assert_eq!(
            path_match_score(&TargetPattern::Path("C:/Work/*.yaml".into()), actual),
            0.6
        );
    }

    #[test]
    fn process_tree_and_window_model_are_deterministic() {
        let mut tree = ProcessTree::default();
        tree.observe_start(10, 1);
        tree.observe_start(11, 10);
        assert_eq!(tree.distance(1, 11), Some(2));
        assert_eq!(tree.nearest_common_ancestor_distance(10, 11), Some(1));
        let window = derive_time_window(1000, None, Some(5000), AlignmentProfile::default());
        assert_eq!(window.start_ms, 1000);
        assert_eq!(window.end_ms, 6000);
        let mut negative = assertion();
        negative.expect = Expectation::MustNotHappen;
        assert_eq!(
            derive_assertion_window(
                &negative,
                1000,
                Some(50),
                Some(9000),
                AlignmentProfile::default()
            )
            .end_ms,
            9000
        );
        let watermark = compute_watermark(5000, 5200, AlignmentProfile::default());
        assert_eq!(watermark_value_ms(watermark), 4200);
        let mut index = ActiveKindIndex::default();
        index.activate(10, SysKind::FileRead, 7);
        index.activate(10, SysKind::FileRead, 7);
        assert_eq!(index.candidates(10, SysKind::FileRead), &[7]);
        index.deactivate(10, SysKind::FileRead, 7);
        assert!(index.candidates(10, SysKind::FileRead).is_empty());
    }

    #[test]
    fn projected_event_time_is_used_for_real_qpc_alignment() {
        let a = assertion();
        let scored = score_candidate_at(
            TimedAssertion {
                assertion: &a,
                center_ts_ms: 10_000,
                window_end_ms: 12_000,
            },
            &event(),
            10_000,
            0,
            1.0,
            1.0,
            AlignmentProfile::default(),
        );
        assert_eq!(scored.time, 1.0);
    }

    #[test]
    fn alignment_engine_closes_windows_and_marks_constraints() {
        let mut engine = AlignmentEngine::new(AlignmentProfile::default());
        engine.activate(assertion(), 1000, 2000);
        engine.ingest_event(event(), 1000);
        assert!(engine
            .drain_updates()
            .iter()
            .any(|update| update.state == VerdictState::Fulfilled));

        let mut constraint = assertion();
        constraint.op = AssertionOp::Constraint;
        constraint.expect = Expectation::MustNotHappen;
        constraint.io_mode = IoMode::ReadOnly;
        constraint.id = 8;
        engine.activate(constraint, 1000, 2000);
        let mut write_event = event();
        write_event.kind = SysKind::FileWrite;
        engine.ingest_event(write_event, 1000);
        assert!(engine
            .drain_updates()
            .iter()
            .any(|update| update.state == VerdictState::Deviated));

        let mut unobserved = assertion();
        unobserved.id = 9;
        unobserved.op = AssertionOp::Exec;
        engine.activate(unobserved, 1000, 2000);
        engine.advance_watermark(Watermark {
            system_wall_ms: 4000,
            now_ms: 4000,
            disorder_ms: 0,
            assertion_lag_ms: 0,
        });
        assert!(engine
            .drain_updates()
            .iter()
            .any(|update| update.assertion_id == 9 && update.state == VerdictState::Unobserved));
    }

    #[test]
    fn process_attribution_uses_parent_and_token_jaccard_bands() {
        assert!(
            (token_jaccard("powershell -File app.ps1", "powershell app.ps1") - 0.667).abs() < 0.01
        );
        let assertion = Assertion {
            raw_quote: "powershell -File app.ps1".into(),
            ..assertion()
        };
        let child = SysEvent {
            kind: SysKind::ProcStart,
            pid: 20,
            payload: EventPayload::Proc {
                ppid: 10,
                image: "powershell.exe".into(),
                cmd: "powershell -File app.ps1".into(),
                exit_code: None,
            },
            ..event()
        };
        let attribution = attribute_child_process(
            10,
            &assertion,
            &child,
            1_000,
            1_000,
            &ProcessTree::default(),
        )
        .expect("child process is attributable");
        assert_eq!(attribution.state, AttributionState::Anchored);
        assert_eq!(attribution.pid, 20);
    }

    #[test]
    fn causal_weave_is_bounded_monotonic_and_penalizes_contradiction() {
        let clean = causal_weave_score(CausalWeaveInput {
            base_score: 0.9,
            prior_gap_ms: Some(10),
            next_gap_ms: Some(10),
            contradiction: false,
            process_distance: 0,
        });
        let deviated = causal_weave_score(CausalWeaveInput {
            base_score: 0.9,
            prior_gap_ms: Some(10),
            next_gap_ms: Some(10),
            contradiction: true,
            process_distance: 0,
        });
        assert!((0.0..=1.0).contains(&clean));
        assert!(clean > deviated);
        assert!(
            causal_weave_score(CausalWeaveInput {
                base_score: 0.8,
                prior_gap_ms: None,
                next_gap_ms: None,
                contradiction: false,
                process_distance: 0
            }) >= causal_weave_score(CausalWeaveInput {
                base_score: 0.4,
                prior_gap_ms: None,
                next_gap_ms: None,
                contradiction: false,
                process_distance: 0
            })
        );
    }
}
