//! Phase 0 acceptance tests — see docs/PLAN_composer_phase0_backend.md §8.
//!
//! These cover graph shape, not media processing: they need no fixture files and
//! run in milliseconds. Actually executing steps is verified separately.

use super::manifest::{self, ParamKind, METHODS};
use super::types::GraphRecipe;
use super::validate;

fn recipe(value: serde_json::Value) -> GraphRecipe {
    serde_json::from_value(value).expect("test recipe should deserialize")
}

fn file_source(path: &str) -> serde_json::Value {
    serde_json::json!({
        "nodeId": "n0", "kind": "source", "nodeRef": "file",
        "params": { "path": path }
    })
}

// ============================================================================
// MANIFEST SELF-CONSISTENCY
// ============================================================================

#[test]
fn manifest_ids_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for m in METHODS {
        assert!(seen.insert(m.id), "duplicate manifest id '{}'", m.id);
    }
}

#[test]
fn manifest_param_keys_are_unique_per_method() {
    for m in METHODS {
        let mut seen = std::collections::HashSet::new();
        for p in m.params {
            assert!(
                seen.insert(p.key),
                "'{}' declares '{}' more than once",
                m.id,
                p.key
            );
        }
    }
}

#[test]
fn enum_params_declare_options_and_a_valid_default() {
    for m in METHODS {
        for p in m.params {
            if p.kind != ParamKind::Enum {
                continue;
            }
            assert!(
                !p.options.is_empty(),
                "'{}.{}' is an enum with no options",
                m.id,
                p.key
            );
            if let Some(super::manifest::DefaultValue::Str(d)) = p.default {
                assert!(
                    p.options.iter().any(|o| o.value == d),
                    "'{}.{}' defaults to '{}', which is not one of its options",
                    m.id,
                    p.key,
                    d
                );
            }
        }
    }
}

#[test]
fn every_satisfied_by_param_names_a_real_param() {
    for m in METHODS {
        for r in m.reads {
            if let Some(key) = r.satisfied_by_param {
                assert!(
                    m.param(key).is_some(),
                    "'{}' says '{}' can satisfy a port, but has no such parameter",
                    m.id,
                    key
                );
            }
        }
    }
}

#[test]
fn source_kinds_resolve_for_every_source_entry() {
    for m in METHODS.iter().filter(|m| m.kind == manifest::NodeKind::Source) {
        assert!(
            manifest::source_kind_of(m.id).is_some(),
            "source '{}' has no MediaSource mapping",
            m.id
        );
    }
}

// ============================================================================
// VALID GRAPHS
// ============================================================================

#[test]
fn fan_out_graph_is_valid() {
    let g = recipe(serde_json::json!({
        "label": "fan",
        "nodes": [
            file_source("/tmp/sample.mp4"),
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_metadata" },
            { "nodeId": "n2", "kind": "method", "nodeRef": "detect_motion",
              "params": { "threshold": 30.0 } }
        ],
        "edges": [{ "from": "n0", "to": "n1" }, { "from": "n0", "to": "n2" }]
    }));

    let v = validate::validate(&g);
    assert!(v.valid, "expected valid, got: {:?}", v.reason);
    assert_eq!(v.order, vec!["n0", "n1", "n2"]);
}

/// The core Phase 0 claim: a blank `inputDir` is satisfied by an upstream step that
/// writes the frames-on-disk slot. Without this the graph is only ever one level deep.
#[test]
fn chain_inherits_the_upstream_directory() {
    let g = recipe(serde_json::json!({
        "label": "chain",
        "nodes": [
            file_source("/tmp/sample.mp4"),
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_frames_to_disk",
              "params": { "outputDir": "/tmp/frames", "interval": 30 } },
            { "nodeId": "n2", "kind": "method", "nodeRef": "group_similar_images",
              "params": { "outputDir": "/tmp/groups" } }
        ],
        "edges": [{ "from": "n0", "to": "n1" }, { "from": "n1", "to": "n2" }]
    }));

    let v = validate::validate(&g);
    assert!(v.valid, "expected valid, got: {:?}", v.reason);
    assert_eq!(v.order, vec!["n0", "n1", "n2"]);
}

#[test]
fn sink_may_take_two_inputs() {
    // Fan-in is supported here, unlike the reference implementation this was modelled
    // on: a step reading two slots simply takes two incoming edges.
    let g = recipe(serde_json::json!({
        "label": "fan-in",
        "nodes": [
            file_source("/tmp/sample.mp4"),
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_metadata" },
            { "nodeId": "n2", "kind": "method", "nodeRef": "detect_motion" },
            { "nodeId": "n3", "kind": "sink", "nodeRef": "save_report",
              "params": { "outputPath": "/tmp/report.json" } }
        ],
        "edges": [
            { "from": "n0", "to": "n1" }, { "from": "n0", "to": "n2" },
            { "from": "n1", "to": "n3" }, { "from": "n2", "to": "n3" }
        ]
    }));

    let v = validate::validate(&g);
    assert!(v.valid, "expected valid, got: {:?}", v.reason);
    assert_eq!(v.order.last().unwrap(), "n3");
}

#[test]
fn camera_source_accepts_capture_frame() {
    let g = recipe(serde_json::json!({
        "nodes": [
            { "nodeId": "n0", "kind": "source", "nodeRef": "camera", "params": { "cameraId": 0 } },
            { "nodeId": "n1", "kind": "method", "nodeRef": "capture_frame" }
        ],
        "edges": [{ "from": "n0", "to": "n1" }]
    }));

    let v = validate::validate(&g);
    assert!(v.valid, "expected valid, got: {:?}", v.reason);
}

// ============================================================================
// REJECTED GRAPHS
// ============================================================================

#[test]
fn chain_without_an_upstream_directory_is_rejected() {
    let g = recipe(serde_json::json!({
        "nodes": [
            file_source("/tmp/sample.mp4"),
            { "nodeId": "n1", "kind": "method", "nodeRef": "group_similar_images",
              "params": { "outputDir": "/tmp/groups" } }
        ],
        "edges": [{ "from": "n0", "to": "n1" }]
    }));

    let v = validate::validate(&g);
    assert!(!v.valid);
    assert!(
        v.reason.as_deref().unwrap_or("").contains("inputDir"),
        "the reason should name the parameter that would fix it: {:?}",
        v.reason
    );
}

#[test]
fn cycles_are_rejected() {
    let g = recipe(serde_json::json!({
        "nodes": [
            file_source("/tmp/sample.mp4"),
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_metadata" },
            { "nodeId": "n2", "kind": "method", "nodeRef": "detect_motion" }
        ],
        "edges": [
            { "from": "n0", "to": "n1" },
            { "from": "n1", "to": "n2" },
            { "from": "n2", "to": "n1" }
        ]
    }));

    let v = validate::validate(&g);
    assert!(!v.valid);
    assert!(v.reason.as_deref().unwrap_or("").contains("cycle"));
}

#[test]
fn two_sources_are_rejected() {
    let g = recipe(serde_json::json!({
        "nodes": [
            file_source("/tmp/a.mp4"),
            { "nodeId": "n1", "kind": "source", "nodeRef": "stream",
              "params": { "url": "rtsp://example/live" } }
        ],
        "edges": []
    }));

    let v = validate::validate(&g);
    assert!(!v.valid);
    assert!(v.reason.as_deref().unwrap_or("").contains("source"));
}

#[test]
fn file_only_steps_reject_a_camera_source() {
    // MediaSource::as_path_str() returns None for Camera, so this would fail at run
    // time with a configuration error. Better to say so before running.
    let g = recipe(serde_json::json!({
        "nodes": [
            { "nodeId": "n0", "kind": "source", "nodeRef": "camera", "params": { "cameraId": 0 } },
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_metadata" }
        ],
        "edges": [{ "from": "n0", "to": "n1" }]
    }));

    let v = validate::validate(&g);
    assert!(!v.valid);
    assert!(v.reason.as_deref().unwrap_or("").contains("camera"));
}

#[test]
fn the_same_method_twice_is_rejected() {
    // The blackboard holds one value per slot, so the second copy would silently
    // overwrite the first.
    let g = recipe(serde_json::json!({
        "nodes": [
            file_source("/tmp/sample.mp4"),
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_metadata" },
            { "nodeId": "n2", "kind": "method", "nodeRef": "extract_metadata" }
        ],
        "edges": [{ "from": "n0", "to": "n1" }, { "from": "n0", "to": "n2" }]
    }));

    let v = validate::validate(&g);
    assert!(!v.valid);
    assert!(v.reason.as_deref().unwrap_or("").contains("more than once"));
}

#[test]
fn a_missing_required_parameter_is_rejected() {
    let g = recipe(serde_json::json!({
        "nodes": [
            file_source("/tmp/sample.mp4"),
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_frames_to_disk" }
        ],
        "edges": [{ "from": "n0", "to": "n1" }]
    }));

    let v = validate::validate(&g);
    assert!(!v.valid);
    assert!(v.reason.as_deref().unwrap_or("").contains("Output directory"));
}

#[test]
fn an_undeclared_parameter_is_rejected() {
    let g = recipe(serde_json::json!({
        "nodes": [
            file_source("/tmp/sample.mp4"),
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_metadata",
              "params": { "windowLines": 12 } }
        ],
        "edges": [{ "from": "n0", "to": "n1" }]
    }));

    let v = validate::validate(&g);
    assert!(!v.valid);
    assert!(v.reason.as_deref().unwrap_or("").contains("windowLines"));
}

#[test]
fn an_out_of_range_enum_value_is_rejected() {
    let g = recipe(serde_json::json!({
        "nodes": [
            file_source("/tmp/sample.mp4"),
            { "nodeId": "n1", "kind": "method", "nodeRef": "detect_motion",
              "params": { "algorithm": "wavelet" } }
        ],
        "edges": [{ "from": "n0", "to": "n1" }]
    }));

    let v = validate::validate(&g);
    assert!(!v.valid);
    assert!(v.reason.as_deref().unwrap_or("").contains("wavelet"));
}

#[test]
fn a_value_below_the_declared_minimum_is_rejected() {
    let g = recipe(serde_json::json!({
        "nodes": [
            file_source("/tmp/sample.mp4"),
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_frames_to_disk",
              "params": { "outputDir": "/tmp/frames", "interval": 0 } }
        ],
        "edges": [{ "from": "n0", "to": "n1" }]
    }));

    let v = validate::validate(&g);
    assert!(!v.valid);
}

// ============================================================================
// WARNINGS
// ============================================================================

#[test]
fn clobbering_writers_produce_a_warning() {
    // detect_motion assigns context.analysis outright, discarding the similarity
    // groups group_similar_images merged in. Legal, but worth saying out loud.
    let g = recipe(serde_json::json!({
        "nodes": [
            file_source("/tmp/sample.mp4"),
            { "nodeId": "n1", "kind": "method", "nodeRef": "detect_motion" },
            { "nodeId": "n2", "kind": "method", "nodeRef": "extract_frames_to_disk",
              "params": { "outputDir": "/tmp/frames" } },
            { "nodeId": "n3", "kind": "method", "nodeRef": "group_similar_images",
              "params": { "outputDir": "/tmp/groups" } }
        ],
        "edges": [
            { "from": "n0", "to": "n1" },
            { "from": "n0", "to": "n2" },
            { "from": "n2", "to": "n3" }
        ]
    }));

    let v = validate::validate(&g);
    assert!(v.valid, "expected valid, got: {:?}", v.reason);
    assert!(
        v.warnings.iter().any(|w| w.contains("replaces the whole value")),
        "expected a clobbering warning, got: {:?}",
        v.warnings
    );
}

#[test]
fn a_disconnected_node_produces_a_warning() {
    let g = recipe(serde_json::json!({
        "nodes": [
            file_source("/tmp/sample.mp4"),
            { "nodeId": "n1", "kind": "method", "nodeRef": "compare_images",
              "params": { "image1": "/tmp/a.png", "image2": "/tmp/b.png" } }
        ],
        "edges": []
    }));

    let v = validate::validate(&g);
    assert!(v.valid, "expected valid, got: {:?}", v.reason);
    assert!(
        v.warnings.iter().any(|w| w.contains("not connected")),
        "got: {:?}",
        v.warnings
    );
}

#[test]
fn a_sink_with_nothing_to_report_produces_a_warning() {
    let g = recipe(serde_json::json!({
        "nodes": [
            file_source("/tmp/sample.mp4"),
            { "nodeId": "n1", "kind": "method", "nodeRef": "convert_to_hls",
              "params": { "outputDir": "/tmp/hls" } },
            { "nodeId": "n2", "kind": "sink", "nodeRef": "save_report",
              "params": { "outputPath": "/tmp/report.json" } }
        ],
        "edges": [{ "from": "n0", "to": "n1" }, { "from": "n1", "to": "n2" }]
    }));

    let v = validate::validate(&g);
    assert!(v.valid, "expected valid, got: {:?}", v.reason);
    assert!(
        v.warnings.iter().any(|w| w.contains("no input to report on")),
        "got: {:?}",
        v.warnings
    );
}

// ============================================================================
// EXECUTION ORDER
// ============================================================================

#[test]
fn topological_order_is_stable_across_runs() {
    let g = recipe(serde_json::json!({
        "nodes": [
            file_source("/tmp/sample.mp4"),
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_metadata" },
            { "nodeId": "n2", "kind": "method", "nodeRef": "detect_motion" },
            { "nodeId": "n3", "kind": "sink", "nodeRef": "save_report",
              "params": { "outputPath": "/tmp/report.json" } }
        ],
        "edges": [
            { "from": "n0", "to": "n1" }, { "from": "n0", "to": "n2" },
            { "from": "n1", "to": "n3" }, { "from": "n2", "to": "n3" }
        ]
    }));

    let first = validate::topo_order(&g).expect("acyclic");
    for _ in 0..20 {
        assert_eq!(validate::topo_order(&g).expect("acyclic"), first);
    }
    // Every edge must point forwards in the order.
    for edge in &g.edges {
        let from = first.iter().position(|id| *id == edge.from).unwrap();
        let to = first.iter().position(|id| *id == edge.to).unwrap();
        assert!(from < to, "edge {} -> {} runs backwards", edge.from, edge.to);
    }
}
