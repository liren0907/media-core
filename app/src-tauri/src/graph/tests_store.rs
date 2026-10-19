//! Saved graphs — what `store` promises, run against a temporary folder.
//!
//! The one promise that cannot bend: a graph saved and opened again is the same
//! graph. The only planned difference is that undeclared parameters are dropped on
//! save, and the returned recipe says so.

use std::path::Path;
use std::time::{Duration, SystemTime};

use super::store;
use super::types::GraphRecipe;

fn recipe(value: serde_json::Value) -> GraphRecipe {
    serde_json::from_value(value).expect("test recipe should deserialize")
}

/// `GraphRecipe` has no `PartialEq`; its JSON form is what is stored anyway.
fn json(recipe: &GraphRecipe) -> serde_json::Value {
    serde_json::to_value(recipe).expect("a recipe always serializes")
}

fn runnable(label: &str) -> GraphRecipe {
    recipe(serde_json::json!({
        "label": label,
        "nodes": [
            { "nodeId": "n0", "kind": "source", "nodeRef": "file",
              "params": { "path": "/tmp/sample.mp4" } },
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_metadata" },
            { "nodeId": "n2", "kind": "method", "nodeRef": "detect_motion",
              "params": { "threshold": 30.0 } }
        ],
        "edges": [{ "from": "n0", "to": "n1" }, { "from": "n0", "to": "n2" }]
    }))
}

fn files_in(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn a_saved_graph_opens_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let original = runnable("Frames then groups");

    let saved = store::save(tmp.path(), None, original.clone()).unwrap();
    assert_eq!(saved.removed_params, 0);
    assert_eq!(json(&saved.recipe), json(&original));

    let opened = store::load(tmp.path(), &saved.id).unwrap();
    assert_eq!(json(&opened), json(&original));
}

#[test]
fn saving_without_an_id_adds_and_with_one_overwrites() {
    let tmp = tempfile::tempdir().unwrap();
    let first = store::save(tmp.path(), None, runnable("first")).unwrap();
    let second = store::save(tmp.path(), None, runnable("second")).unwrap();
    assert_ne!(first.id, second.id);
    assert_eq!(files_in(tmp.path()).len(), 2);

    let renamed = store::save(tmp.path(), Some(&first.id), runnable("first, renamed")).unwrap();
    assert_eq!(renamed.id, first.id);
    assert_eq!(
        files_in(tmp.path()).len(),
        2,
        "an overwrite must not add a file"
    );
    assert_eq!(
        store::load(tmp.path(), &first.id).unwrap().label,
        "first, renamed"
    );
}

/// Saving is the write gate only: half-drawn graphs and graphs whose method has
/// since disappeared are saved, and the list says why they cannot run.
#[test]
fn a_graph_that_cannot_run_is_still_saved() {
    let tmp = tempfile::tempdir().unwrap();
    let missing_param = recipe(serde_json::json!({
        "label": "half drawn",
        "nodes": [
            { "nodeId": "n0", "kind": "source", "nodeRef": "file",
              "params": { "path": "/tmp/sample.mp4" } },
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_frames_to_disk" }
        ],
        "edges": [{ "from": "n0", "to": "n1" }]
    }));
    let unknown_method = recipe(serde_json::json!({
        "label": "renamed method",
        "nodes": [
            { "nodeId": "n0", "kind": "source", "nodeRef": "file",
              "params": { "path": "/tmp/sample.mp4" } },
            { "nodeId": "n1", "kind": "sink", "nodeRef": "no_such_method" }
        ],
        "edges": [{ "from": "n0", "to": "n1" }]
    }));

    let a = store::save(tmp.path(), None, missing_param).unwrap();
    let b = store::save(tmp.path(), None, unknown_method.clone()).unwrap();
    assert_eq!(
        json(&store::load(tmp.path(), &b.id).unwrap()),
        json(&unknown_method)
    );

    let list = store::list(tmp.path()).unwrap();
    let row = list.iter().find(|r| r.id == a.id).unwrap();
    assert!(!row.valid);
    assert!(row.broken.is_none());
    assert!(
        row.reason
            .as_deref()
            .unwrap_or("")
            .contains("Output directory"),
        "expected the missing parameter as the reason, got {:?}",
        row.reason
    );
    let row = list.iter().find(|r| r.id == b.id).unwrap();
    assert!(!row.valid);
    assert!(row.reason.is_some());
}

#[test]
fn a_graph_that_could_not_be_read_back_is_not_written() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("graphs");
    let duplicate_ids = recipe(serde_json::json!({
        "nodes": [
            { "nodeId": "n0", "kind": "source", "nodeRef": "file" },
            { "nodeId": "n0", "kind": "method", "nodeRef": "extract_metadata" }
        ]
    }));

    let err = store::save(&dir, None, duplicate_ids).unwrap_err();
    assert!(err.contains("Duplicate node id"), "got: {err}");
    assert!(
        files_in(&dir).is_empty(),
        "nothing may be left behind, not even a .tmp"
    );
}

#[test]
fn an_unreadable_file_does_not_break_the_list() {
    let tmp = tempfile::tempdir().unwrap();
    let good = store::save(tmp.path(), None, runnable("good")).unwrap();

    let garbage = uuid::Uuid::new_v4().to_string();
    std::fs::write(tmp.path().join(format!("{garbage}.json")), "not json").unwrap();
    // Hand-edited into duplicate ids: parses, but the canvas could not hold it.
    let edited = uuid::Uuid::new_v4().to_string();
    std::fs::write(
        tmp.path().join(format!("{edited}.json")),
        r#"{ "nodes": [ { "nodeId": "n0", "kind": "source", "nodeRef": "file" },
                        { "nodeId": "n0", "kind": "source", "nodeRef": "file" } ] }"#,
    )
    .unwrap();
    // Not ours: no id to open or delete them by.
    std::fs::write(tmp.path().join("notes.json"), "{}").unwrap();
    std::fs::write(tmp.path().join(format!("{garbage}.json.tmp")), "{}").unwrap();

    let list = store::list(tmp.path()).unwrap();
    assert_eq!(list.len(), 3, "got {list:?}");
    assert!(list
        .iter()
        .any(|r| r.id == good.id && r.valid && r.broken.is_none()));
    for id in [&garbage, &edited] {
        let row = list.iter().find(|r| &r.id == id).unwrap();
        assert!(row.broken.is_some());
        assert!(!row.valid);
        assert!(store::load(tmp.path(), id).is_err());
    }
    // A broken row can still be cleared away.
    store::delete(tmp.path(), &garbage).unwrap();
}

#[test]
fn the_list_is_newest_first() {
    let tmp = tempfile::tempdir().unwrap();
    let older = store::save(tmp.path(), None, runnable("older")).unwrap();
    let newer = store::save(tmp.path(), None, runnable("newer")).unwrap();

    let set_mtime = |id: &str, secs: u64| {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(tmp.path().join(format!("{id}.json")))
            .unwrap();
        file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
            .unwrap();
    };
    set_mtime(&older.id, 1_000_000);
    set_mtime(&newer.id, 2_000_000);

    let list = store::list(tmp.path()).unwrap();
    let labels: Vec<&str> = list.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(labels, ["newer", "older"]);
    assert_eq!(list[0].modified_ms, 2_000_000_000);
    assert_eq!(list[0].node_count, 3);
}

#[test]
fn ids_that_are_not_graph_ids_are_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let saved = store::save(tmp.path(), None, runnable("kept")).unwrap();
    let upper = saved.id.to_uppercase();

    for id in ["", "../x", "x", "notes", upper.as_str()] {
        assert!(store::load(tmp.path(), id).is_err(), "load accepted {id:?}");
        assert!(
            store::delete(tmp.path(), id).is_err(),
            "delete accepted {id:?}"
        );
        assert!(
            store::save(tmp.path(), Some(id), runnable("x")).is_err(),
            "save accepted {id:?}"
        );
    }
    assert_eq!(files_in(tmp.path()).len(), 1);
}

#[test]
fn deleting_removes_the_graph_once() {
    let tmp = tempfile::tempdir().unwrap();
    let a = store::save(tmp.path(), None, runnable("a")).unwrap();
    store::save(tmp.path(), None, runnable("b")).unwrap();

    store::delete(tmp.path(), &a.id).unwrap();
    let list = store::list(tmp.path()).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].label, "b");

    assert!(store::delete(tmp.path(), &a.id).is_err());
    assert!(store::load(tmp.path(), &a.id).is_err());
}

/// The parameters the canvas can neither show nor delete are dropped, and the
/// returned recipe — what the canvas reloads from — no longer has them either.
#[test]
fn undeclared_parameters_are_dropped_on_save() {
    let tmp = tempfile::tempdir().unwrap();
    let stale = recipe(serde_json::json!({
        "label": "stale keys",
        "nodes": [
            { "nodeId": "n0", "kind": "source", "nodeRef": "file",
              "params": { "path": "/tmp/sample.mp4" } },
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_metadata",
              "params": { "windowLines": 12 } },
            { "nodeId": "n2", "kind": "method", "nodeRef": "detect_motion",
              "params": { "threshold": 30.0, "oldName": true } },
            { "nodeId": "n3", "kind": "method", "nodeRef": "no_such_method",
              "params": { "anything": 1 } }
        ],
        "edges": [
            { "from": "n0", "to": "n1" },
            { "from": "n0", "to": "n2" },
            { "from": "n0", "to": "n3" }
        ]
    }));

    let saved = store::save(tmp.path(), None, stale).unwrap();
    assert_eq!(saved.removed_params, 2);

    let opened = store::load(tmp.path(), &saved.id).unwrap();
    assert_eq!(
        json(&opened),
        json(&saved.recipe),
        "canvas and file must agree"
    );
    // Emptied by the drop: no params at all, as the canvas itself would send.
    assert!(opened.node("n1").unwrap().params.is_none());
    assert_eq!(
        json(&opened)["nodes"][2]["params"],
        serde_json::json!({ "threshold": 30.0 })
    );
    // No declaration to judge by, so nothing is dropped.
    assert_eq!(
        json(&opened)["nodes"][3]["params"],
        serde_json::json!({ "anything": 1 })
    );
}
