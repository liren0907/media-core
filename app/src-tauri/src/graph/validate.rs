//! Graph validation, in two layers.
//!
//! `validate_integrity` is the WRITE gate: it only asks "can this data be read back".
//! It deliberately does not check that `node_ref` exists in the manifest, so a graph
//! can always be stored — including the edit that removes a node the manifest no
//! longer knows about.
//!
//! `validate` is the RUN judge (G1–G7). Slot supply is judged on ANCESTRY, not on
//! "everything that happens to run earlier": the execution order is a topological
//! linearisation of the picture, and the picture is what the user drew. Anything
//! ancestry allows, execution order also allows, so this is the stricter and more
//! honest of the two readings.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use super::manifest::{self, MethodMeta, ParamKind, Slot, SourceKind, WriteMode};
use super::types::{GraphRecipe, GraphVerdict, NodeParams};

const KINDS: [&str; 3] = ["source", "method", "sink"];

// ============================================================================
// LAYER 1 — INTEGRITY (the write gate)
// ============================================================================

pub fn validate_integrity(recipe: &GraphRecipe) -> Result<(), String> {
    // G1 — node identity
    let mut seen: HashSet<&str> = HashSet::new();
    for node in &recipe.nodes {
        if node.node_id.trim().is_empty() {
            return Err("A node has an empty id".to_string());
        }
        if !seen.insert(node.node_id.as_str()) {
            return Err(format!("Duplicate node id '{}'", node.node_id));
        }
        if node.node_ref.trim().is_empty() {
            return Err(format!("Node '{}' has an empty method reference", node.node_id));
        }
        if !KINDS.contains(&node.kind.as_str()) {
            return Err(format!(
                "Node '{}' has an unknown kind '{}'",
                node.node_id, node.kind
            ));
        }
    }

    // G2 — edge integrity
    let mut edge_seen: HashSet<(&str, &str)> = HashSet::new();
    for edge in &recipe.edges {
        if edge.from == edge.to {
            return Err(format!("Node '{}' is connected to itself", edge.from));
        }
        if recipe.node(&edge.from).is_none() {
            return Err(format!("Edge starts at unknown node '{}'", edge.from));
        }
        if recipe.node(&edge.to).is_none() {
            return Err(format!("Edge ends at unknown node '{}'", edge.to));
        }
        if !edge_seen.insert((edge.from.as_str(), edge.to.as_str())) {
            return Err(format!(
                "Duplicate connection from '{}' to '{}'",
                edge.from, edge.to
            ));
        }
    }

    Ok(())
}

// ============================================================================
// TOPOLOGICAL ORDER (G5)
// ============================================================================

/// Kahn's algorithm. Ties are broken by declaration order so the same graph always
/// executes in the same order — otherwise multi-writer slots would give drifting
/// results between runs.
pub fn topo_order(recipe: &GraphRecipe) -> Result<Vec<String>, String> {
    let mut in_degree: HashMap<&str, usize> = recipe
        .nodes
        .iter()
        .map(|n| (n.node_id.as_str(), 0usize))
        .collect();

    for edge in &recipe.edges {
        if let Some(d) = in_degree.get_mut(edge.to.as_str()) {
            *d += 1;
        }
    }

    let mut order: Vec<String> = Vec::with_capacity(recipe.nodes.len());
    let mut ready: Vec<&str> = recipe
        .nodes
        .iter()
        .filter(|n| in_degree[n.node_id.as_str()] == 0)
        .map(|n| n.node_id.as_str())
        .collect();

    while let Some(id) = ready.first().copied() {
        ready.remove(0);
        order.push(id.to_string());

        // Follow outgoing edges in declaration order, then re-sort the ready set by
        // declaration order so the whole traversal is deterministic.
        let mut freed: Vec<&str> = Vec::new();
        for edge in recipe.edges.iter().filter(|e| e.from == id) {
            if let Some(d) = in_degree.get_mut(edge.to.as_str()) {
                *d -= 1;
                if *d == 0 {
                    freed.push(edge.to.as_str());
                }
            }
        }
        ready.extend(freed);
        ready.sort_by_key(|id| {
            recipe
                .nodes
                .iter()
                .position(|n| n.node_id == *id)
                .unwrap_or(usize::MAX)
        });
    }

    if order.len() != recipe.nodes.len() {
        return Err("The graph contains a cycle".to_string());
    }
    Ok(order)
}

// ============================================================================
// LAYER 2 — RUNNABILITY (the run judge)
// ============================================================================

pub fn validate(recipe: &GraphRecipe) -> GraphVerdict {
    if let Err(reason) = validate_integrity(recipe) {
        return GraphVerdict::invalid(reason);
    }

    if recipe.nodes.is_empty() {
        return GraphVerdict::invalid("The graph is empty");
    }

    // ---- G3: every node resolves to a manifest entry, params are well formed ----
    let mut metas: HashMap<&str, &'static MethodMeta> = HashMap::new();
    for node in &recipe.nodes {
        let Some(meta) = manifest::find(&node.node_ref) else {
            return GraphVerdict::invalid(format!(
                "Node '{}' refers to an unknown method '{}'",
                node.node_id, node.node_ref
            ));
        };
        if meta.kind.as_str() != node.kind {
            return GraphVerdict::invalid(format!(
                "Node '{}' is stored as '{}' but '{}' is a {}",
                node.node_id,
                node.kind,
                node.node_ref,
                meta.kind.as_str()
            ));
        }
        if let Err(reason) = check_params(meta, &node.params()) {
            return GraphVerdict::invalid(reason);
        }
        metas.insert(node.node_id.as_str(), meta);
    }

    // ---- G4: exactly one source; sources have no input, sinks have no output ----
    let sources: Vec<&str> = recipe
        .nodes
        .iter()
        .filter(|n| n.kind == "source")
        .map(|n| n.node_id.as_str())
        .collect();
    match sources.len() {
        0 => return GraphVerdict::invalid("The graph has no source node"),
        1 => {}
        n => {
            return GraphVerdict::invalid(format!(
                "The graph has {} source nodes; exactly one is allowed",
                n
            ))
        }
    }
    for node in &recipe.nodes {
        if node.kind == "source" && recipe.in_degree(&node.node_id) > 0 {
            return GraphVerdict::invalid(format!(
                "Source node '{}' cannot have an input",
                node.node_id
            ));
        }
        if node.kind == "sink" && recipe.out_degree(&node.node_id) > 0 {
            return GraphVerdict::invalid(format!(
                "Sink node '{}' cannot have an output",
                node.node_id
            ));
        }
    }

    // ---- G7: one node per method -------------------------------------------
    // The blackboard holds one value per slot, so a second copy of the same method
    // would silently overwrite the first. A v1 restriction, recorded as such.
    let mut refs: HashSet<&str> = HashSet::new();
    for node in &recipe.nodes {
        if !refs.insert(node.node_ref.as_str()) {
            return GraphVerdict::invalid(format!(
                "'{}' appears more than once; each method can only be used once per graph",
                node.node_ref
            ));
        }
    }

    // ---- G5: acyclic --------------------------------------------------------
    let order = match topo_order(recipe) {
        Ok(order) => order,
        Err(reason) => return GraphVerdict::invalid(reason),
    };

    // ---- G6: slot supply along ancestry -------------------------------------
    let source_node = recipe.source_node().expect("checked above");
    let source_kind = manifest::source_kind_of(&source_node.node_ref);

    // available[n] = union over direct upstreams u of (available[u] ∪ writes(u))
    let mut available: HashMap<&str, HashSet<Slot>> = HashMap::new();
    for id in &order {
        let mut avail: HashSet<Slot> = HashSet::new();
        for up in recipe.upstream_of(id) {
            if let Some(inherited) = available.get(up) {
                avail.extend(inherited.iter().copied());
            }
            if let Some(meta) = metas.get(up) {
                avail.extend(meta.writes.iter().map(|w| w.slot));
            }
        }
        let key = recipe
            .node(id)
            .map(|n| n.node_id.as_str())
            .expect("id came from the recipe");
        available.insert(key, avail);
    }

    for node in &recipe.nodes {
        let meta = metas[node.node_id.as_str()];
        let params = node.params();
        let avail = &available[node.node_id.as_str()];

        for read in meta.reads {
            let satisfied_by_param = read
                .satisfied_by_param
                .map(|key| is_set(&params, key))
                .unwrap_or(false);

            if !read.optional && !satisfied_by_param && !avail.contains(&read.slot) {
                return GraphVerdict::invalid(match read.satisfied_by_param {
                    Some(key) => format!(
                        "'{}' needs {} — connect a step that produces it, or fill in '{}'",
                        meta.label,
                        read.slot.label(),
                        key
                    ),
                    None => format!(
                        "'{}' needs {}, which nothing upstream produces",
                        meta.label,
                        read.slot.label()
                    ),
                });
            }

            // The three MediaSource variants are not interchangeable: as_path_str()
            // returns None for Stream and Camera, so file-based steps cannot run.
            if read.slot == Slot::Source && !read.accepts.is_empty() && !satisfied_by_param {
                if let Some(kind) = source_kind {
                    if !read.accepts.contains(&kind) {
                        return GraphVerdict::invalid(format!(
                            "'{}' cannot read from a {} source",
                            meta.label,
                            source_label(kind)
                        ));
                    }
                }
            }
        }
    }

    GraphVerdict {
        valid: true,
        reason: None,
        warnings: collect_warnings(recipe, &metas, &available),
        order,
    }
}

// ============================================================================
// WARNINGS — findings that do not block the run
// ============================================================================

fn collect_warnings(
    recipe: &GraphRecipe,
    metas: &HashMap<&str, &'static MethodMeta>,
    available: &HashMap<&str, HashSet<Slot>>,
) -> Vec<String> {
    let mut warnings = Vec::new();

    // A slot written by several steps where at least one overwrites wholesale.
    // detect_motion is the real case: it assigns context.analysis outright and so
    // discards the similarity groups the other analysis steps merged in.
    let mut writers: HashMap<Slot, Vec<(&str, WriteMode)>> = HashMap::new();
    for node in &recipe.nodes {
        if let Some(meta) = metas.get(node.node_id.as_str()) {
            for write in meta.writes {
                writers
                    .entry(write.slot)
                    .or_default()
                    .push((meta.label, write.mode));
            }
        }
    }
    for (slot, list) in &writers {
        if list.len() > 1 && list.iter().any(|(_, mode)| *mode == WriteMode::Replace) {
            let names: Vec<&str> = list.iter().map(|(label, _)| *label).collect();
            warnings.push(format!(
                "{} is written by {} — one of them replaces the whole value, so earlier results may be lost",
                slot.label(),
                names.join(", ")
            ));
        }
    }

    // ffmpeg is resolved through PATH. A bundled .app launched from Finder inherits
    // launchd's PATH, which does not include Homebrew.
    if recipe
        .nodes
        .iter()
        .any(|n| metas.get(n.node_id.as_str()).is_some_and(|m| m.requires_ffmpeg))
        && !ffmpeg_available()
    {
        warnings.push("ffmpeg was not found on PATH; steps that need it will fail".to_string());
    }

    // Disconnected nodes: not a broken graph, just an unfinished one.
    if recipe.nodes.len() > 1 {
        for node in &recipe.nodes {
            if recipe.in_degree(&node.node_id) == 0 && recipe.out_degree(&node.node_id) == 0 {
                let label = metas
                    .get(node.node_id.as_str())
                    .map(|m| m.label)
                    .unwrap_or(node.node_ref.as_str());
                warnings.push(format!("'{}' is not connected to anything", label));
            }
        }
    }

    // A sink whose optional inputs are all missing will fail at run time.
    for node in &recipe.nodes {
        let Some(meta) = metas.get(node.node_id.as_str()) else {
            continue;
        };
        if meta.kind != manifest::NodeKind::Sink || meta.reads.is_empty() {
            continue;
        }
        let avail = &available[node.node_id.as_str()];
        if !meta.reads.iter().any(|r| avail.contains(&r.slot)) {
            warnings.push(format!(
                "'{}' has no input to report on; it will fail when the graph runs",
                meta.label
            ));
        }
    }

    warnings
}

// ============================================================================
// HELPERS
// ============================================================================

fn is_set(params: &NodeParams, key: &str) -> bool {
    match params.get(key) {
        None | Some(serde_json::Value::Null) => false,
        Some(serde_json::Value::String(s)) => !s.trim().is_empty(),
        Some(_) => true,
    }
}

fn source_label(kind: SourceKind) -> &'static str {
    match kind {
        SourceKind::File => "file",
        SourceKind::Stream => "stream",
        SourceKind::Camera => "camera",
    }
}

fn check_params(meta: &MethodMeta, params: &NodeParams) -> Result<(), String> {
    for key in params.keys() {
        if meta.param(key).is_none() {
            return Err(format!("'{}' has no parameter called '{}'", meta.label, key));
        }
    }

    for spec in meta.params {
        if !is_set(params, spec.key) {
            if spec.required {
                return Err(format!("'{}' needs '{}' to be set", meta.label, spec.label));
            }
            continue;
        }
        let value = &params[spec.key];

        match spec.kind {
            ParamKind::Bool => {
                if !value.is_boolean() {
                    return Err(type_error(meta, spec.label, "true or false"));
                }
            }
            ParamKind::Int | ParamKind::Float => {
                let Some(n) = value.as_f64() else {
                    return Err(type_error(meta, spec.label, "a number"));
                };
                if let Some(min) = spec.min {
                    if n < min {
                        return Err(format!(
                            "'{}': {} must be at least {}",
                            meta.label, spec.label, min
                        ));
                    }
                }
                if let Some(max) = spec.max {
                    if n > max {
                        return Err(format!(
                            "'{}': {} must be at most {}",
                            meta.label, spec.label, max
                        ));
                    }
                }
            }
            ParamKind::Str | ParamKind::Path | ParamKind::Dir => {
                if !value.is_string() {
                    return Err(type_error(meta, spec.label, "text"));
                }
            }
            ParamKind::Enum => {
                let Some(s) = value.as_str() else {
                    return Err(type_error(meta, spec.label, "one of the listed options"));
                };
                if !spec.options.iter().any(|o| o.value == s) {
                    return Err(format!(
                        "'{}': '{}' is not a valid {}",
                        meta.label, s, spec.label
                    ));
                }
            }
        }
    }

    Ok(())
}

fn type_error(meta: &MethodMeta, label: &str, expected: &str) -> String {
    format!("'{}': {} must be {}", meta.label, label, expected)
}

fn ffmpeg_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        std::process::Command::new("ffmpeg")
            .arg("-version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    })
}
