//! Read-only, explicit multi-repository planning. No execution capability or replay.
use super::push_preview::{complete_history, remaining};
use super::*;
use crate::remote::push::*;
use std::collections::{BTreeMap, BTreeSet};
#[derive(PartialEq, Eq)]
struct EvidenceContext {
    reachable: bool,
    parent_generation: String,
    parent_config: String,
    child_generation: String,
    child_config: String,
    observed_oid: crate::domain::ObjectId,
    observed_at: u64,
}
impl From<&ChildReachability> for EvidenceContext {
    fn from(value: &ChildReachability) -> Self {
        Self {
            reachable: value.reachable,
            parent_generation: value.parent.generation.clone(),
            parent_config: value.parent.config_digest.clone(),
            child_generation: value.child_generation.clone(),
            child_config: value.child_config_digest.clone(),
            observed_oid: value.observed_child_oid.clone(),
            observed_at: value.child_observed_at_ms,
        }
    }
}
fn invalid(message: &str) -> Error {
    fail(ErrorKind::InvalidRequest, message)
}
fn stale() -> Error {
    fail(
        ErrorKind::StaleReview,
        "Push planning context changed; inspect and select again",
    )
}
fn binding_key(b: &ChildReachabilitySelection) -> (Vec<u8>, String, String) {
    (
        b.path.clone(),
        b.child_oid.hex.clone(),
        b.gitmodules_blob.hex.clone(),
    )
}
fn order(
    graph: &BTreeMap<String, BTreeSet<String>>,
) -> std::result::Result<Vec<String>, Vec<String>> {
    let mut pending = graph.clone();
    let mut result = Vec::new();
    while !pending.is_empty() {
        let Some(ready) = pending
            .iter()
            .find(|(_, children)| children.is_empty())
            .map(|(id, _)| id.clone())
        else {
            break;
        };
        pending.remove(&ready);
        for children in pending.values_mut() {
            children.remove(&ready);
        }
        result.push(ready);
    }
    if pending.is_empty() {
        return Ok(result);
    }
    // At most 32 selected nodes. A bounded DFS produces an actual closed cycle.
    fn visit(
        id: &str,
        graph: &BTreeMap<String, BTreeSet<String>>,
        done: &mut BTreeSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        if let Some(index) = path.iter().position(|v| v == id) {
            let mut cycle = path[index..].to_vec();
            cycle.push(id.into());
            return Some(cycle);
        }
        if done.contains(id) {
            return None;
        }
        path.push(id.into());
        for child in &graph[id] {
            if let Some(cycle) = visit(child, graph, done, path) {
                return Some(cycle);
            }
        }
        path.pop();
        done.insert(id.into());
        None
    }
    let mut done = BTreeSet::new();
    for id in graph.keys() {
        if let Some(cycle) = visit(id, graph, &mut done, &mut Vec::new()) {
            return Err(cycle);
        }
    }
    unreachable!("nonempty dependency graph without a ready node has a cycle")
}
impl ControlledWrites {
    pub(crate) fn push_dependency_plan(
        &mut self,
        workspace: &WorkspaceId,
        entries: &BTreeMap<String, crate::workspace::StoredWorktree>,
        selections: Vec<PushPlanningSelection>,
        options: &ReadOptions,
    ) -> Result<PushDependencyPlan> {
        let started = Instant::now();
        if selections.is_empty()
            || selections.len() > 32
            || selections.iter().map(|s| s.children.len()).sum::<usize>() > 256
        {
            return Err(invalid(
                "Choose 1–32 worktrees and at most 256 explicit child bindings",
            ));
        }
        let mut selected = BTreeMap::new();
        for s in selections {
            let id = s.worktree.clone();
            if selected.insert(id, s).is_some() {
                return Err(invalid("Duplicate selected worktree"));
            }
        }
        let mut previews = BTreeMap::new();
        let mut targets = BTreeSet::new();
        let mut issues = Vec::new();
        let mut contexts = 0;
        let mut preview_bytes = 0;
        for (id, s) in &selected {
            let entry = entries
                .get(id)
                .ok_or_else(|| invalid("Unknown selected worktree"))?;
            let p = self.push_preview(
                workspace,
                entry,
                &s.observation,
                &remaining(options, started)?,
            )?;
            if p.source_oid != s.source_oid {
                return Err(stale());
            }
            preview_bytes += serde_json::to_vec(&p)?.len();
            if preview_bytes > 2 * 1024 * 1024 {
                return Err(fail(
                    ErrorKind::OutputLimit,
                    "Selected push previews exceed the 2 MiB planning budget",
                ));
            }
            if !targets.insert((p.destination.clone(), p.destination_ref.clone())) {
                return Err(invalid(
                    "Selected worktrees must have distinct destination branches",
                ));
            }
            contexts += p
                .outgoing_gitlinks
                .iter()
                .map(|r| r.mappings.len())
                .sum::<usize>();
            if contexts > 256 {
                return Err(fail(
                    ErrorKind::OutputLimit,
                    "Push plan exceeds 256 historical dependency contexts",
                ));
            }
            if !matches!(
                p.relation,
                PushRelation::FastForward | PushRelation::UpToDate
            ) {
                issues.push(format!(
                    "{id}: source is behind or diverged; non-force planning is blocked"
                ));
            }
            if now_ms()
                .checked_sub(p.observed_at_ms)
                .is_none_or(|age| age > 300_000)
            {
                return Err(stale());
            }
            previews.insert(id.clone(), p);
        }
        let mut graph: BTreeMap<String, BTreeSet<String>> = selected
            .keys()
            .map(|id| (id.clone(), BTreeSet::new()))
            .collect();
        let mut dependencies = Vec::new();
        let mut checked = Vec::new();
        for (id, s) in &selected {
            let p = &previews[id];
            let mut bindings = BTreeMap::new();
            for b in &s.children {
                if b.parent_observation != s.observation
                    || b.source_oid != s.source_oid
                    || bindings.insert(binding_key(b), b).is_some()
                {
                    return Err(invalid("Duplicate or mismatched child binding"));
                }
            }
            for r in &p.outgoing_gitlinks {
                for mapping in &r.mappings {
                    let key = mapping
                        .gitmodules_blob
                        .as_ref()
                        .map(|blob| (r.path.clone(), r.oid.hex.clone(), blob.hex.clone()));
                    let binding = key.as_ref().and_then(|key| bindings.remove(key));
                    let mut edge = PushDependency {
                        parent_worktree: id.clone(),
                        path: r.path.clone(),
                        child_oid: r.oid.clone(),
                        gitmodules_blob: mapping.gitmodules_blob.clone(),
                        child_worktree: binding.map(|b| b.child_worktree.clone()),
                        child_selected: binding
                            .is_some_and(|b| selected.contains_key(&b.child_worktree)),
                        child_observation: binding.map(|b| b.child_observation.clone()),
                        observed_child_oid: None,
                        child_observed_at_ms: None,
                        state: DependencyState::Blocked,
                        issue: None,
                    };
                    if mapping.issue.is_some() || mapping.gitmodules_blob.is_none() {
                        edge.issue =
                            Some("Historical child destination is missing or unsupported".into());
                    } else if let Some(b) = binding {
                        let child = entries
                            .get(&b.child_worktree)
                            .ok_or_else(|| invalid("Unknown bound child worktree"))?;
                        if edge.child_selected {
                            let candidate = &previews[&b.child_worktree];
                            if candidate.observation_id != b.child_observation
                                || mapping
                                    .destination
                                    .as_ref()
                                    .is_none_or(|d| d.display != candidate.destination)
                            {
                                edge.issue=Some("Selected child must use this exact destination and observation".into());
                            } else {
                                graph
                                    .get_mut(id)
                                    .ok_or_else(stale)?
                                    .insert(b.child_worktree.clone());
                            }
                        }
                        if edge.issue.is_none() {
                            match self.child_reachability_from_preview(
                                workspace,
                                &entries[id],
                                child,
                                b.clone(),
                                p,
                                &remaining(options, started)?,
                            ) {
                                Ok(evidence) => {
                                    if evidence.parent.generation != p.generation
                                        || evidence.parent.config_digest != p.config_digest
                                    {
                                        return Err(stale());
                                    }
                                    edge.observed_child_oid =
                                        Some(evidence.observed_child_oid.clone());
                                    edge.child_observed_at_ms = Some(evidence.child_observed_at_ms);
                                    if evidence.reachable {
                                        edge.state = DependencyState::AlreadyReachable;
                                    } else if edge.child_selected {
                                        let candidate = &previews[&b.child_worktree];
                                        let contains = inspection::read(
                                            &child.metadata.root.to_path()?,
                                            &remaining(options, started)?,
                                            |session| {
                                                if session.generation != candidate.generation {
                                                    return Err(stale());
                                                }
                                                complete_history(session)?;
                                                let args = [
                                                    "merge-base",
                                                    "--is-ancestor",
                                                    &b.child_oid.hex,
                                                    &candidate.source_oid.hex,
                                                ];
                                                let output = session.output(
                                                    &args
                                                        .iter()
                                                        .map(std::ffi::OsString::from)
                                                        .collect::<Vec<_>>(),
                                                )?;
                                                match output.status.code(){Some(0)=>Ok(true),Some(1)=>Ok(false),_=>Err(fail(ErrorKind::InspectionFailed,"Cannot inspect selected child source ancestry"))}
                                            },
                                        )?;
                                        if contains {
                                            edge.state = DependencyState::SelectedChildRequired;
                                        } else {
                                            edge.issue=Some("Selected child source does not contain the required commit".into());
                                        }
                                    } else {
                                        edge.issue=Some("Unselected child is not reachable remotely; select it explicitly or obtain fresh remote evidence".into());
                                    }
                                    checked.push((
                                        id.clone(),
                                        b.clone(),
                                        EvidenceContext::from(&evidence),
                                    ));
                                }
                                Err(e)
                                    if matches!(
                                        e.kind,
                                        ErrorKind::Timeout
                                            | ErrorKind::Cancelled
                                            | ErrorKind::OutputLimit
                                            | ErrorKind::Storage
                                            | ErrorKind::Io
                                            | ErrorKind::Busy
                                            | ErrorKind::NewerSchema
                                    ) =>
                                {
                                    return Err(e)
                                }
                                Err(e) => edge.issue = Some(e.to_string()),
                            }
                        }
                    } else {
                        edge.issue=Some("Choose explicit child repository and observation evidence for this mapping".into());
                    }
                    dependencies.push(edge);
                }
            }
            if !bindings.is_empty() {
                return Err(invalid(
                    "Child binding is not a current historical requirement",
                ));
            }
        }
        for (id, b, old) in checked {
            let current = self.child_reachability_from_preview(
                workspace,
                &entries[&id],
                &entries[&b.child_worktree],
                b,
                &previews[&id],
                &remaining(options, started)?,
            )?;
            if EvidenceContext::from(&current) != old {
                return Err(stale());
            }
        }
        // Reject source/configuration drift even in selected roots that have no children.
        for (id, s) in &selected {
            let p = self.push_preview(
                workspace,
                &entries[id],
                &s.observation,
                &remaining(options, started)?,
            )?;
            let old = &previews[id];
            if p.source_oid != old.source_oid
                || p.generation != old.generation
                || p.config_digest != old.config_digest
            {
                return Err(stale());
            }
            if now_ms()
                .checked_sub(p.observed_at_ms)
                .is_none_or(|age| age > 300_000)
            {
                return Err(stale());
            }
        }
        if previews.values().any(|p| {
            now_ms()
                .checked_sub(p.observed_at_ms)
                .is_none_or(|age| age > 300_000)
        }) || dependencies
            .iter()
            .filter_map(|e| e.child_observed_at_ms)
            .any(|time| now_ms().checked_sub(time).is_none_or(|age| age > 300_000))
        {
            return Err(stale());
        }
        let (mut order, cycle) = match order(&graph) {
            Ok(order) => (order, Vec::new()),
            Err(cycle) => {
                issues.push(
                    "Cyclic selected child dependencies; no publication order is available".into(),
                );
                (Vec::new(), cycle)
            }
        };
        if dependencies
            .iter()
            .any(|e| e.state == DependencyState::Blocked)
            || !issues.is_empty()
        {
            order.clear();
        }
        let plan = PushDependencyPlan {
            generated_at_ms: now_ms(),
            items: previews
                .into_iter()
                .map(|(worktree, preview)| PlannedPush { worktree, preview })
                .collect(),
            dependencies,
            order,
            cycle,
            issues,
        };
        if serde_json::to_vec(&plan)?.len() > 2 * 1024 * 1024 {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Serialized push plan exceeds 2 MiB",
            ));
        }
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn graph(edges: &[(&str, &[&str])]) -> BTreeMap<String, BTreeSet<String>> {
        edges
            .iter()
            .map(|(id, children)| {
                (
                    id.to_string(),
                    children.iter().map(|v| v.to_string()).collect(),
                )
            })
            .collect()
    }
    #[test]
    fn cycle_witness_excludes_acyclic_dependents_and_independent_nodes() {
        let g = graph(&[
            ("a", &["b"]),
            ("b", &["c"]),
            ("c", &["b"]),
            ("d", &[]),
            ("e", &["a"]),
        ]);
        assert_eq!(order(&g).unwrap_err(), vec!["b", "c", "b"]);
        let g = graph(&[("self", &["self"])]);
        assert_eq!(order(&g).unwrap_err(), vec!["self", "self"]);
    }
    #[test]
    fn stable_order_visits_only_selected_nodes_children_first() {
        let g = graph(&[
            ("parent", &["child"]),
            ("child", &["grand"]),
            ("grand", &[]),
            ("independent", &[]),
        ]);
        let result = order(&g).unwrap();
        assert!(
            result.iter().position(|v| v == "grand") < result.iter().position(|v| v == "child")
        );
        assert!(
            result.iter().position(|v| v == "child") < result.iter().position(|v| v == "parent")
        );
        assert_eq!(result.len(), 4);
    }
}
