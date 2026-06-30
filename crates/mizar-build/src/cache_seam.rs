use std::collections::{BTreeMap, BTreeSet};

use crate::task_graph::TaskId;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CacheSchedulingPlan {
    pub decisions: Vec<CacheTaskDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheTaskDecision {
    pub task_id: TaskId,
    pub outcome: CacheSchedulingOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheSchedulingOutcome {
    ValidatedHit(ValidatedCacheHit),
    Miss(CacheFallbackReason),
    NoKey(CacheFallbackReason),
    Unavailable(CacheFallbackReason),
    ErrorAsMiss(CacheFallbackReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CacheFallbackReason {
    Miss,
    NoKey,
    Uncacheable,
    Unavailable,
    Error,
    FutureIncompatible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedCacheHit {
    pub output_refs: Vec<CacheOutputRef>,
    pub diagnostics: Vec<CacheDiagnosticRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CacheOutputRef {
    pub identity: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CacheDiagnosticRef {
    pub source: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheSchedulingPlanDiagnostics {
    diagnostics: Vec<CacheSchedulingPlanDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheSchedulingPlanDiagnostic {
    pub task_id: TaskId,
    pub kind: CacheSchedulingPlanDiagnosticKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CacheSchedulingPlanDiagnosticKind {
    DuplicateDecision,
    UnknownTask,
}

impl CacheSchedulingPlan {
    #[must_use]
    pub fn new(decisions: Vec<CacheTaskDecision>) -> Self {
        Self { decisions }
    }

    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn decisions(&self) -> &[CacheTaskDecision] {
        &self.decisions
    }

    pub fn validated_decision_map(
        &self,
        known_tasks: &BTreeSet<TaskId>,
    ) -> Result<BTreeMap<TaskId, CacheSchedulingOutcome>, CacheSchedulingPlanDiagnostics> {
        let mut seen = BTreeSet::new();
        let mut map = BTreeMap::new();
        let mut diagnostics = Vec::new();

        for decision in &self.decisions {
            if !known_tasks.contains(&decision.task_id) {
                diagnostics.push(CacheSchedulingPlanDiagnostic {
                    task_id: decision.task_id.clone(),
                    kind: CacheSchedulingPlanDiagnosticKind::UnknownTask,
                });
                continue;
            }
            if !seen.insert(decision.task_id.clone()) {
                diagnostics.push(CacheSchedulingPlanDiagnostic {
                    task_id: decision.task_id.clone(),
                    kind: CacheSchedulingPlanDiagnosticKind::DuplicateDecision,
                });
                continue;
            }
            map.insert(decision.task_id.clone(), decision.outcome.clone());
        }

        if diagnostics.is_empty() {
            Ok(map)
        } else {
            Err(CacheSchedulingPlanDiagnostics::new(diagnostics))
        }
    }
}

impl CacheTaskDecision {
    #[must_use]
    pub fn new(task_id: TaskId, outcome: CacheSchedulingOutcome) -> Self {
        Self { task_id, outcome }
    }
}

impl ValidatedCacheHit {
    #[must_use]
    pub fn new(
        mut output_refs: Vec<CacheOutputRef>,
        mut diagnostics: Vec<CacheDiagnosticRef>,
    ) -> Self {
        output_refs.sort();
        output_refs.dedup();
        diagnostics.sort();
        diagnostics.dedup();
        Self {
            output_refs,
            diagnostics,
        }
    }
}

impl CacheOutputRef {
    #[must_use]
    pub fn new(identity: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
            content: content.into(),
        }
    }
}

impl CacheDiagnosticRef {
    #[must_use]
    pub fn new(
        source: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            source: source.into(),
            code: code.into(),
            message: message.into(),
        }
    }
}

impl CacheSchedulingPlanDiagnostics {
    #[must_use]
    pub fn new(mut diagnostics: Vec<CacheSchedulingPlanDiagnostic>) -> Self {
        diagnostics.sort_by_key(|diagnostic| (diagnostic.task_id.clone(), diagnostic.kind));
        Self { diagnostics }
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[CacheSchedulingPlanDiagnostic] {
        &self.diagnostics
    }

    #[must_use]
    pub fn into_diagnostics(self) -> Vec<CacheSchedulingPlanDiagnostic> {
        self.diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validated_hit_payload_is_canonicalized() {
        let hit = ValidatedCacheHit::new(
            vec![
                CacheOutputRef::new("b", "second"),
                CacheOutputRef::new("a", "first"),
                CacheOutputRef::new("a", "first"),
            ],
            vec![
                CacheDiagnosticRef::new("main", "W002", "later"),
                CacheDiagnosticRef::new("main", "W001", "earlier"),
                CacheDiagnosticRef::new("main", "W001", "earlier"),
            ],
        );

        assert_eq!(
            hit.output_refs,
            vec![
                CacheOutputRef::new("a", "first"),
                CacheOutputRef::new("b", "second"),
            ]
        );
        assert_eq!(
            hit.diagnostics,
            vec![
                CacheDiagnosticRef::new("main", "W001", "earlier"),
                CacheDiagnosticRef::new("main", "W002", "later"),
            ]
        );
    }

    #[test]
    fn decision_plan_rejects_duplicate_and_unknown_tasks() {
        let known = TaskId::new_for_test("known");
        let unknown = TaskId::new_for_test("unknown");
        let plan = CacheSchedulingPlan::new(vec![
            CacheTaskDecision::new(
                known.clone(),
                CacheSchedulingOutcome::Miss(CacheFallbackReason::Miss),
            ),
            CacheTaskDecision::new(
                known.clone(),
                CacheSchedulingOutcome::Unavailable(CacheFallbackReason::Unavailable),
            ),
            CacheTaskDecision::new(
                unknown.clone(),
                CacheSchedulingOutcome::NoKey(CacheFallbackReason::NoKey),
            ),
        ]);

        let diagnostics = plan
            .validated_decision_map(&BTreeSet::from([known.clone()]))
            .expect_err("invalid plan is rejected");

        assert_eq!(
            diagnostics.diagnostics(),
            &[
                CacheSchedulingPlanDiagnostic {
                    task_id: known,
                    kind: CacheSchedulingPlanDiagnosticKind::DuplicateDecision,
                },
                CacheSchedulingPlanDiagnostic {
                    task_id: unknown,
                    kind: CacheSchedulingPlanDiagnosticKind::UnknownTask,
                },
            ]
        );
    }
}
