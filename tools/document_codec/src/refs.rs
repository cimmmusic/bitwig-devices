//! ObjectRef resolution and baking.
//!
//! The Ramona binary format uses sequential depth-first IDs for ObjectRef.
//! These are a serialization artifact — they become stale after any structural
//! edit (duplicate, delete, reorder).
//!
//! Body and metadata are separate encrypted sections, each with its own
//! sequential numbering starting from 1.
//!
//! This module provides two operations:
//!
//! - **resolve**: called after loading from binary.  Assigns stable IDs to every
//!   object and rewrites ObjectRef values from sequential → stable.
//!
//! - **bake**: called before saving to binary.  Walks the tree in the same
//!   depth-first order as the deserializer, assigns fresh sequential IDs, and
//!   rewrites ObjectRef values from stable → sequential.

use std::collections::HashMap;

use crate::types::{Module, RamonaListItem, RamonaObject, RamonaValue};

// ---------------------------------------------------------------------------
// Resolve: sequential → stable  (call after decode_module)
// ---------------------------------------------------------------------------

/// Assign stable IDs to every object in the module and rewrite all ObjectRef
/// values from sequential (wire) IDs to stable IDs.
///
/// Body and metadata are resolved independently since they are separate
/// serialization sections with their own sequential numbering.
pub fn resolve_refs(module: &mut Module) {
    // Stable ID counter is shared across both sections so IDs are globally unique.
    let mut stable_counter: i32 = 0;

    // Each section has its own sequential numbering starting from 1.
    resolve_section(&mut module.body, &mut stable_counter);
    resolve_section(&mut module.metadata, &mut stable_counter);
}

fn resolve_section(root: &mut RamonaObject, stable_counter: &mut i32) {
    let mut seq_counter: i32 = 0;
    let mut seq_to_stable: HashMap<i32, i32> = HashMap::new();

    assign_stable_ids(root, &mut seq_counter, stable_counter, &mut seq_to_stable);
    rewrite_refs_obj(root, &seq_to_stable);
}

fn assign_stable_ids(
    obj: &mut RamonaObject,
    seq_counter: &mut i32,
    stable_counter: &mut i32,
    map: &mut HashMap<i32, i32>,
) {
    *seq_counter += 1;
    *stable_counter += 1;
    let seq_id = *seq_counter;
    let stable_id = *stable_counter;
    obj.stable_id = stable_id;
    map.insert(seq_id, stable_id);

    for (_key, val) in &mut obj.fields {
        assign_stable_ids_value(val, seq_counter, stable_counter, map);
    }
}

fn assign_stable_ids_value(
    val: &mut RamonaValue,
    seq_counter: &mut i32,
    stable_counter: &mut i32,
    map: &mut HashMap<i32, i32>,
) {
    match val {
        RamonaValue::Object(obj) => {
            assign_stable_ids(obj, seq_counter, stable_counter, map);
        }
        RamonaValue::RelativeRef(data) => {
            if let Some(obj) = data.as_object_mut() {
                assign_stable_ids(obj, seq_counter, stable_counter, map);
            }
        }
        RamonaValue::ObjectList(items) => {
            for item in items {
                match item {
                    RamonaListItem::Object(obj) => {
                        assign_stable_ids(obj, seq_counter, stable_counter, map);
                    }
                    RamonaListItem::RelativeRef(data) => {
                        if let Some(obj) = data.as_object_mut() {
                            assign_stable_ids(obj, seq_counter, stable_counter, map);
                        }
                    }
                    _ => {}
                }
            }
        }
        RamonaValue::StringToObjectMap(entries) => {
            for (_k, v) in entries {
                assign_stable_ids_value(v, seq_counter, stable_counter, map);
            }
        }
        _ => {}
    }
}

fn rewrite_refs_obj(obj: &mut RamonaObject, map: &HashMap<i32, i32>) {
    for (_key, val) in &mut obj.fields {
        rewrite_refs_value(val, map);
    }
}

fn rewrite_refs_value(val: &mut RamonaValue, map: &HashMap<i32, i32>) {
    match val {
        RamonaValue::ObjectRef(id) => {
            if let Some(&stable) = map.get(id) {
                *id = stable;
            }
        }
        RamonaValue::ExtObjectRef(id) => {
            if let Some(&stable) = map.get(id) {
                *id = stable;
            }
        }
        RamonaValue::Object(obj) => {
            rewrite_refs_obj(obj, map);
        }
        RamonaValue::RelativeRef(data) => {
            if let Some(obj) = data.as_object_mut() {
                rewrite_refs_obj(obj, map);
            }
        }
        RamonaValue::ObjectList(items) => {
            for item in items {
                match item {
                    RamonaListItem::Object(obj) => {
                        rewrite_refs_obj(obj, map);
                    }
                    RamonaListItem::RelativeRef(data) => {
                        if let Some(obj) = data.as_object_mut() {
                            rewrite_refs_obj(obj, map);
                        }
                    }
                    RamonaListItem::ObjectRef(id) => {
                        if let Some(&stable) = map.get(id) {
                            *id = stable;
                        }
                    }
                    RamonaListItem::ExtObjectRef(id) => {
                        if let Some(&stable) = map.get(id) {
                            *id = stable;
                        }
                    }
                    _ => {}
                }
            }
        }
        RamonaValue::StringToObjectMap(entries) => {
            for (_k, v) in entries {
                rewrite_refs_value(v, map);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Bake: stable → sequential  (call before encode_module)
// ---------------------------------------------------------------------------

/// Rewrite all ObjectRef values from stable IDs back to fresh sequential IDs
/// (depth-first order).  Call this on a **clone** before encoding to binary.
///
/// Body and metadata are baked independently (separate wire sections).
pub fn bake_refs(module: &mut Module) {
    bake_section(&mut module.body);
    bake_section(&mut module.metadata);
}

fn bake_section(root: &mut RamonaObject) {
    let mut seq_counter: i32 = 0;
    let mut stable_to_seq: HashMap<i32, i32> = HashMap::new();

    collect_seq_ids(root, &mut seq_counter, &mut stable_to_seq);
    rewrite_refs_obj(root, &stable_to_seq);
}

fn collect_seq_ids(
    obj: &RamonaObject,
    counter: &mut i32,
    map: &mut HashMap<i32, i32>,
) {
    *counter += 1;
    let seq_id = *counter;
    if obj.stable_id != 0 {
        map.insert(obj.stable_id, seq_id);
    }

    for (_key, val) in &obj.fields {
        collect_seq_ids_value(val, counter, map);
    }
}

fn collect_seq_ids_value(
    val: &RamonaValue,
    counter: &mut i32,
    map: &mut HashMap<i32, i32>,
) {
    match val {
        RamonaValue::Object(obj) => {
            collect_seq_ids(obj, counter, map);
        }
        RamonaValue::RelativeRef(data) => {
            if let Some(obj) = data.as_object() {
                collect_seq_ids(obj, counter, map);
            }
        }
        RamonaValue::ObjectList(items) => {
            for item in items {
                match item {
                    RamonaListItem::Object(obj) => {
                        collect_seq_ids(obj, counter, map);
                    }
                    RamonaListItem::RelativeRef(data) => {
                        if let Some(obj) = data.as_object() {
                            collect_seq_ids(obj, counter, map);
                        }
                    }
                    _ => {}
                }
            }
        }
        RamonaValue::StringToObjectMap(entries) => {
            for (_k, v) in entries {
                collect_seq_ids_value(v, counter, map);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Helpers for structural edits
// ---------------------------------------------------------------------------

/// Assign fresh stable IDs to all objects in a subtree.  Use after cloning
/// a subtree (duplicate/paste) so the copies get their own identities.
///
/// Returns a mapping from old_stable_id → new_stable_id for internal ref fixup.
pub fn reassign_stable_ids_obj(obj: &mut RamonaObject, counter: &mut i32) -> HashMap<i32, i32> {
    let mut remap = HashMap::new();
    reassign_walk_obj(obj, counter, &mut remap);
    remap
}

pub fn reassign_stable_ids_item(item: &mut RamonaListItem, counter: &mut i32) -> HashMap<i32, i32> {
    let mut remap = HashMap::new();
    match item {
        RamonaListItem::Object(obj) => {
            reassign_walk_obj(obj, counter, &mut remap);
        }
        RamonaListItem::RelativeRef(data) => {
            if let Some(obj) = data.as_object_mut() {
                reassign_walk_obj(obj, counter, &mut remap);
            }
        }
        _ => {}
    }
    remap
}

fn reassign_walk_obj(obj: &mut RamonaObject, counter: &mut i32, remap: &mut HashMap<i32, i32>) {
    *counter += 1;
    let old = obj.stable_id;
    let new = *counter;
    obj.stable_id = new;
    if old != 0 {
        remap.insert(old, new);
    }

    for (_key, val) in &mut obj.fields {
        reassign_walk_value(val, counter, remap);
    }
}

fn reassign_walk_value(val: &mut RamonaValue, counter: &mut i32, remap: &mut HashMap<i32, i32>) {
    match val {
        RamonaValue::Object(obj) => {
            reassign_walk_obj(obj, counter, remap);
        }
        RamonaValue::RelativeRef(data) => {
            if let Some(obj) = data.as_object_mut() {
                reassign_walk_obj(obj, counter, remap);
            }
        }
        RamonaValue::ObjectList(items) => {
            for item in items {
                match item {
                    RamonaListItem::Object(obj) => {
                        reassign_walk_obj(obj, counter, remap);
                    }
                    RamonaListItem::RelativeRef(data) => {
                        if let Some(obj) = data.as_object_mut() {
                            reassign_walk_obj(obj, counter, remap);
                        }
                    }
                    _ => {}
                }
            }
        }
        RamonaValue::StringToObjectMap(entries) => {
            for (_k, v) in entries {
                reassign_walk_value(v, counter, remap);
            }
        }
        _ => {}
    }
}

/// Find the maximum stable_id in the module (for choosing the next counter value).
pub fn max_stable_id(module: &Module) -> i32 {
    let mut max = 0i32;
    max_id_obj(&module.metadata, &mut max);
    max_id_obj(&module.body, &mut max);
    max
}

fn max_id_obj(obj: &RamonaObject, max: &mut i32) {
    *max = (*max).max(obj.stable_id);
    for (_key, val) in &obj.fields {
        max_id_value(val, max);
    }
}

fn max_id_value(val: &RamonaValue, max: &mut i32) {
    match val {
        RamonaValue::Object(obj) => max_id_obj(obj, max),
        RamonaValue::RelativeRef(data) => {
            if let Some(obj) = data.as_object() {
                max_id_obj(obj, max);
            }
        }
        RamonaValue::ObjectList(items) => {
            for item in items {
                match item {
                    RamonaListItem::Object(obj) => max_id_obj(obj, max),
                    RamonaListItem::RelativeRef(data) => {
                        if let Some(obj) = data.as_object() {
                            max_id_obj(obj, max);
                        }
                    }
                    _ => {}
                }
            }
        }
        RamonaValue::StringToObjectMap(entries) => {
            for (_k, v) in entries { max_id_value(v, max); }
        }
        _ => {}
    }
}
