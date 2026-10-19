//! Saved graphs — one `GraphRecipe` per JSON file in a folder.
//!
//! The file is the recipe itself, written verbatim: no wrapper, no coordinates, so
//! saving, validating and running all read the same thing. Saving passes only the
//! write gate (`validate_integrity`), so a graph that cannot run yet is still saved;
//! listing judges every graph afresh with `validate`.
//!
//! Every function takes the folder, so the tests run against a temporary one; the
//! commands in `commands/graph.rs` pass `{app_data_dir}/composer/graphs`.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::manifest;
use super::types::{GraphListItem, GraphRecipe, SavedGraph};
use super::validate;

/// Save a graph. Without an id this adds a new file under a fresh id; with one it
/// replaces that graph's file.
///
/// The file is written beside its final name and then renamed over it, so a crash
/// halfway never leaves a file that cannot be read back.
pub fn save(dir: &Path, id: Option<&str>, recipe: GraphRecipe) -> Result<SavedGraph, String> {
    validate::validate_integrity(&recipe)?;

    let id = match id {
        Some(id) => id.to_string(),
        None => uuid::Uuid::new_v4().to_string(),
    };
    let path = file_path(dir, &id)?;

    let mut recipe = recipe;
    let removed_params = drop_undeclared_params(&mut recipe);

    std::fs::create_dir_all(dir).map_err(|e| format!("Could not create the graphs folder: {e}"))?;
    let bytes = serde_json::to_vec_pretty(&recipe).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, bytes).map_err(|e| format!("Could not write the graph: {e}"))?;
    if let Err(e) = std::fs::rename(&tmp, &path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!("Could not write the graph: {e}"));
    }

    Ok(SavedGraph {
        id,
        recipe,
        removed_params,
    })
}

/// Every saved graph, newest first. A file that cannot be read back is still listed,
/// marked `broken`, so one bad file never hides the others.
pub fn list(dir: &Path) -> Result<Vec<GraphListItem>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("Could not read the graphs folder: {e}"))?;

    let mut items = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        // Only files named by an id are ours: anything else could be neither opened
        // nor deleted through an id.
        let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if file_path(dir, id).is_err() {
            continue;
        }

        let modified_ms = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        items.push(match read(&path) {
            Ok(recipe) => {
                let verdict = validate::validate(&recipe);
                GraphListItem {
                    id: id.to_string(),
                    node_count: recipe.nodes.len(),
                    label: recipe.label,
                    modified_ms,
                    valid: verdict.valid,
                    reason: verdict.reason,
                    broken: None,
                }
            }
            Err(e) => GraphListItem {
                id: id.to_string(),
                label: String::new(),
                node_count: 0,
                modified_ms,
                valid: false,
                reason: None,
                broken: Some(e),
            },
        });
    }

    items.sort_by(|a, b| {
        b.modified_ms
            .cmp(&a.modified_ms)
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(items)
}

/// Read a graph back exactly as it was saved. Whether it can run is the caller's
/// next question, through `validate`.
pub fn load(dir: &Path, id: &str) -> Result<GraphRecipe, String> {
    let path = file_path(dir, id)?;
    if !path.exists() {
        return Err("That graph no longer exists".to_string());
    }
    read(&path)
}

pub fn delete(dir: &Path, id: &str) -> Result<(), String> {
    let path = file_path(dir, id)?;
    std::fs::remove_file(&path).map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            "That graph no longer exists".to_string()
        } else {
            format!("Could not delete the graph: {e}")
        }
    })
}

/// Ids are the uuids `save` mints, in that exact spelling. Requiring it also keeps an
/// id from naming a path outside the folder (`../x`).
fn file_path(dir: &Path, id: &str) -> Result<PathBuf, String> {
    match uuid::Uuid::parse_str(id) {
        Ok(parsed) if parsed.to_string() == id => Ok(dir.join(format!("{id}.json"))),
        _ => Err(format!("'{id}' is not a saved graph id")),
    }
}

/// "Can be read back" means the same as the write gate: a file edited by hand into
/// duplicate ids or dangling edges would break the canvas, so it counts as unreadable.
fn read(path: &Path) -> Result<GraphRecipe, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Could not read the file: {e}"))?;
    let recipe: GraphRecipe =
        serde_json::from_slice(&bytes).map_err(|e| format!("Not a graph file: {e}"))?;
    validate::validate_integrity(&recipe)?;
    Ok(recipe)
}

/// Drop the parameters a node's method does not declare. They are ignored at run time
/// anyway, and the canvas can neither show nor delete them, so keeping them would
/// leave a warning nobody can clear. A node whose method is unknown keeps everything:
/// without a declaration there is no telling which keys are stale.
fn drop_undeclared_params(recipe: &mut GraphRecipe) -> usize {
    let mut removed = 0;
    for node in &mut recipe.nodes {
        let Some(meta) = manifest::find(&node.node_ref) else {
            continue;
        };
        let Some(params) = node.params.as_mut() else {
            continue;
        };
        let before = params.len();
        params.retain(|key, _| meta.param(key).is_some());
        removed += before - params.len();
        // The canvas never sends an empty map, so neither does the file.
        if params.is_empty() {
            node.params = None;
        }
    }
    removed
}
