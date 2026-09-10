# AAGL Technical Specification

**Standard Version:** 0.1.0  
**Status:** Draft / Architecture Specification  

---

## 1. Overview & Objectives

**AAGL (Agentic Architectural Graph Language)** is an embedded, high-performance spatial database engine implemented in Rust. It is designed to store, query, and mutate complex spatial and structural graphs (such as architectural buildings or microchip layouts) in a deterministic, version-controlled manner optimized for autonomous AI agents.

---

## 2. Container Format (`.aagl`)

An AAGL project is stored as a single portable file with the `.aagl` extension. Under the hood, it is a standard **ZIP archive** utilizing the Central Directory for O(1) random access to individual shards without full decompression.

### Directory Structure Inside `.aagl`
```text
project.aagl (ZIP Container)
├── manifest.json             # Global routing table, spatial index, and version metadata
├── schema.json               # Profile validation rules and constraints
├── chunks/                   # Sharded sub-graphs (50-100 KB chunks)
│   ├── sector_01.json
│   └── sector_02.json
└── assets/                   # Binary attachments (textures, blueprints, point clouds)
```
## 3. Global Manifest (manifest.json)
The manifest acts as the database catalog. It contains global settings, spatial routing, and version hashes for Optimistic Concurrency Control (OCC).

```json
{
  "spec_version": "0.1.0",
  "project_name": "Processor_Core_A",
  "state_hash": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "global_version": 12,
  "units": {
    "base_unit": "mm",
    "internal_resolution_to_meters": 0.001,
    "description": "1 integer unit in coordinates equals 1 millimeter"
  },
  "chunks": {
    "sector_01": {
      "path": "./chunks/sector_01.json",
      "hash": "sha256:abc123...",
      "version": 5
    }
  },
  "spatial_index": {
    "node_id_101": "sector_01",
    "node_id_102": "sector_01"
  }
}
```

> **State Hash Calculation Rule:** When computing the SHA-256 `state_hash` for `manifest.json`, the `state_hash` field itself MUST be temporarily excluded (set to an empty string `""` or omitted entirely) before hashing to prevent a circular dependency. After the digest is computed, the resulting hash is written back into the field. This is analogous to how Git commit hashes are calculated (the commit object is hashed without its own `sha` line).
>
> Example pseudo-code:
> ```
> temp_manifest = manifest.clone();
> temp_manifest.state_hash = "";
> digest = sha256(canonical_json(temp_manifest));
> manifest.state_hash = "sha256:" + hex(digest);
> ```
## 4. Coordinate System & Precision
Data Type: All geometric coordinates and dimensional values are represented as native 64-bit signed integers (i64). Floating-point types are strictly prohibited in core spatial structures to prevent precision drift.

Units Scaling:

The global scale is defined in manifest.json (units).

Individual nodes or components can override or scale units locally if required by multi-scale systems (e.g., micro-components inside a macro layout).

## 5. Shards & Chunks (50–100 KB Strategy)
Granularity: Data is partitioned into semantic chunks (chunks/) targeting 50–100 KB in size. This ensures optimal balance between Git friendliness, minimal LLM context consumption, and low file-system overhead.

Isolation: Each chunk contains an independent sub-graph of elements with internal references (parent_id, connections).

## 6. Optimistic Concurrency Control (OCC)
To allow multiple AI agents to mutate the database in parallel without locking:

Every write operation must supply the base_version (or chunk-level hash) it was built upon.

The Rust core compares the provided base_version with the current state in manifest.json.

If hashes match, the mutation is applied atomically, and the version hash increments.

If a mismatch occurs, the transaction is rejected, forcing the agent to fetch the latest state and resolve conflicts.

## 7. Mutation Protocol (JSON Patch)
Mutations are applied atomically using JSON Patch (RFC 6902) payloads sent to the Rust core.

Example patch format:

```json
[
  { "op": "replace", "path": "/chunks/sector_01/nodes/wall_42/geometry/coordinates/1", "value": 1500 },
  { "op": "add", "path": "/chunks/sector_01/nodes/door_12", "value": { "type": "Door", "parent_id": "wall_42" } }
]
```

### 7.1 Deletion and Cascading

When a delete operation targets a parent node, a sector root, or any element that owns child references, the Rust core MUST execute a **tree-like cascade delete** walk to deterministically remove all descendants before removing the target itself. This guarantees that no dangling pointers (stale `parent_id` or cross-references) remain anywhere in the graph. The cascade order is depth-first, children-first: leaves are removed first, then their parents, so every intermediate state of the graph remains referentially sound.

After every cascade (or any sequence of deletions), the core MUST inspect each affected chunk. If a chunk's JSON file becomes completely empty (i.e., it contains no nodes, edges, or remaining data entries after the operation), the physical chunk file is permanently removed from the ZIP container's `chunks/` directory. The `manifest.json` is then updated atomically to reflect this:

1. The corresponding entry under `manifest.chunks` (including its `path`, `hash`, and `version`) is deleted.
2. Any entries in `manifest.spatial_index` that pointed to nodes residing in the removed chunk are purged.
3. The `manifest.global_version` is incremented, and a new `state_hash` is recalculated (following the rule in Section 3) and written back to the manifest.

This chunk cleanup ensures the container stays lean and free of orphaned shard files, while keeping the routing table and state hash fully consistent.