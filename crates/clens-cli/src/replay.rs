use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEvent {
    pub seq: u64,
    pub assertion_id: Option<u64>,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReplayDiff {
    pub assertion_id: Option<u64>,
    pub baseline: Option<ReplayEvent>,
    pub branch: Option<ReplayEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReplayOverlay {
    pub mode: &'static str,
    pub baseline_events_reused: usize,
    pub branch_events: usize,
    pub diffs: Vec<ReplayDiff>,
    pub counterfactual_simulation: bool,
    pub note: &'static str,
}

impl ReplayOverlay {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|_| "{\"mode\":\"observe\",\"diffs\":[]}".to_owned())
    }
}

/// Read-only overlay for two observed runs. Events before the common prefix are
/// reused conceptually; post-fork events are compared by stable assertion ID and
/// fingerprint. No v2 counterfactual simulation is performed.
pub fn replay_overlay(baseline: &[ReplayEvent], branch: &[ReplayEvent]) -> ReplayOverlay {
    let prefix = common_prefix(baseline, branch);
    let mut base_by_assertion: BTreeMap<Option<u64>, ReplayEvent> = BTreeMap::new();
    let mut branch_by_assertion: BTreeMap<Option<u64>, ReplayEvent> = BTreeMap::new();
    for event in baseline.iter().skip(prefix) {
        base_by_assertion.insert(event.assertion_id, event.clone());
    }
    for event in branch.iter().skip(prefix) {
        branch_by_assertion.insert(event.assertion_id, event.clone());
    }
    let keys: BTreeSet<Option<u64>> = base_by_assertion
        .keys()
        .chain(branch_by_assertion.keys())
        .copied()
        .collect();
    let mut diffs = Vec::new();
    for key in keys {
        let left = base_by_assertion.get(&key);
        let right = branch_by_assertion.get(&key);
        if left.map(|event| &event.fingerprint) != right.map(|event| &event.fingerprint) {
            diffs.push(ReplayDiff {
                assertion_id: key,
                baseline: left.cloned(),
                branch: right.cloned(),
            });
        }
    }
    ReplayOverlay {
        mode: "observe",
        baseline_events_reused: prefix,
        branch_events: branch.len().saturating_sub(prefix),
        diffs,
        counterfactual_simulation: false,
        note: "read-only overlay; counterfactual simulation is v0.2",
    }
}

fn common_prefix(left: &[ReplayEvent], right: &[ReplayEvent]) -> usize {
    left.iter()
        .zip(right)
        .take_while(|(a, b)| a.seq == b.seq && a.fingerprint == b.fingerprint)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(seq: u64, assertion_id: Option<u64>, fingerprint: &str) -> ReplayEvent {
        ReplayEvent {
            seq,
            assertion_id,
            fingerprint: fingerprint.to_owned(),
        }
    }
    #[test]
    fn reuses_prefix_and_reports_observed_branch_diff() {
        let baseline = vec![event(1, Some(10), "read"), event(2, Some(11), "write")];
        let branch = vec![event(1, Some(10), "read"), event(2, Some(11), "delete")];
        let overlay = replay_overlay(&baseline, &branch);
        assert_eq!(overlay.baseline_events_reused, 1);
        assert_eq!(overlay.diffs.len(), 1);
        assert!(!overlay.counterfactual_simulation);
    }
    #[test]
    fn stable_assertion_ids_align_insertions() {
        let baseline = vec![event(1, Some(10), "read"), event(2, Some(11), "write")];
        let branch = vec![
            event(1, Some(10), "read"),
            event(3, Some(11), "write"),
            event(4, Some(12), "connect"),
        ];
        let overlay = replay_overlay(&baseline, &branch);
        assert_eq!(overlay.diffs.len(), 1);
        assert_eq!(overlay.diffs[0].assertion_id, Some(12));
    }
}
