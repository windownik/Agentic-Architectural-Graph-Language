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
│   ├── chunk_genesis.json    # Mandatory genesis chunk (global bounds + root space)
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
    "chunk_genesis": {
      "path": "./chunks/chunk_genesis.json",
      "hash": "sha256:def456...",
      "version": 1
    },
    "sector_01": {
      "path": "./chunks/sector_01.json",
      "hash": "sha256:abc123...",
      "version": 5
    }
  },
  "spatial_index": {
    "node_root_space": "chunk_genesis",
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

Granularity: Data is partitioned into semantic chunks (`chunks/`) targeting 50–100 KB in size. This ensures optimal balance between Git friendliness, minimal LLM context consumption, and low file-system overhead.

Isolation: Each chunk contains an independent sub-graph of elements with internal references (`parent_id`, `relations`).

### 5.1 Genesis Chunk & Global Bounding Volume

The very first chunk in every AAGL project MUST be a **Genesis Chunk** with the reserved `chunk_id: "chunk_genesis"`, stored at `./chunks/chunk_genesis.json`. This chunk is immutable in its identity — it can never be deleted, renamed, or unregistered from the manifest, even if its contents evolve.

The Genesis Chunk defines the project's global spatial envelope as an **Axis-Aligned Bounding Box (AABB)**, encoded as a parallelepiped in the chunk's top-level `bounds` field. The `bounds` value is a flat 6-element array of `i64` integers in the form `[x_min, y_min, z_min, x_max, y_max, z_max]`, expressed in the project's base units declared in `manifest.json`.

Example:
```
"bounds": [0, 0, 0, 10000, 5000, 3000]
```
represents a volume from origin (0,0,0) to (10000, 5000, 3000), e.g. a 10m × 5m × 3m volume at millimetre resolution.

All coordinates and node placements in the project SHOULD fall within this volume. The Genesis Chunk also hosts the root node of the ownership tree (the node whose `parent_id` is the literal string `"genesis"`); see Section 5.2.

### 5.2 Core-Agnostic Node Structure

Every node inside any chunk conforms to the same four-field core data model. The Rust core understands only these fields; everything else is treated as opaque and delegated to `schema.json` for external semantic validation.

| Field         | Type                        | Description |
|---------------|-----------------------------|-------------|
| `parent_id`   | `String`                    | Hierarchical ownership pointer. Root-level elements (direct children of the Genesis Chunk's root space) MUST use the reserved literal string `"genesis"` instead of `null`. All other nodes reference their owning parent's `node_id`. This establishes a strict tree used for cascade deletion and traversal. |
| `coords`      | `Vec<i64>`                  | Spatial geometry expressed as a flat list of native 64-bit signed integers. Floating-point types are forbidden here. The exact interpretation (points, AABB min/max, polyline vertices, packed parallelepiped corners, etc.) is defined by `schema.json` based on `metadata.type`; the Rust core treats this array as an opaque but order-preserving integer blob. |
| `relations`   | `[Relation]`                | Array of cross-node graph pointers, independent from the `parent_id` ownership tree. See Section 5.3 for the full taxonomy. |
| `metadata`    | `serde_json::Value` (JSON object) | Engine-agnostic, opaque JSON object carrying domain-specific parameters, visual properties, material assignments, and — crucially — the node's semantic `type` (e.g. `"Wall"`, `"Door"`, `"RootSpace"`). The Rust core never inspects this object's contents; validation of `type`, required fields, and invariants is the exclusive responsibility of `schema.json`. |

This strict four-field separation keeps the Rust core minimal, deterministic, and reusable across any spatial domain (architecture, microchip layouts, mechanical assemblies) while pushing domain complexity into declarative schema rules.

### 5.3 Universal Relations Taxonomy (`relations`)

The `relations` array on a node establishes a **non-hierarchical graph layer** that complements the ownership tree defined by `parent_id`. Relations are treated by the Rust core as lightweight, soft pointers: the core enforces only minimal structural invariants (presence of mandatory fields, integerity of cross-chunk routing) and performs no semantic validation of relationship meaning — that is deferred to `schema.json`.

Every entry in the `relations` array MUST contain exactly the following three mandatory fields:

| Field           | Type     | Description |
|-----------------|----------|-------------|
| `target`        | `String` | The `node_id` of the relation's destination node. |
| `target_chunk`  | `String` | The `chunk_id` of the chunk hosting the `target` node, enabling the core to resolve cross-chunk references via the manifest's routing table. |
| `rel_type`      | `String` | One of four universal, domain-agnostic relation codes. Custom or domain-specific relation subtypes MUST be encoded inside `metadata` on either endpoint, not by extending this enum. |

The four universal values of `rel_type` are:

- **`"directed"`** — A one-way, directional vector, flow, or link from the source node to the target node. Examples: signal propagation, a one-way portal, a supply line *from* A *to* B, heat flow. The asymmetry is semantically meaningful: reversing the direction changes the relation's meaning.
- **`"symmetric"`** — An equal, bidirectional topological relationship between two nodes. Examples: two zones that are adjacent, two walls that physically touch, mirrored elements. If node X has a `symmetric` relation to Y, a fully consistent graph SHOULD also declare the inverse relation on Y, though the core does not enforce this at write time (schema rules may).
- **`"control"`** — A trigger or activation link where the source node exerts control over the target node's state or behaviour. Examples: a light switch controlling a lamp, a sensor triggering a valve, a schedule controlling a thermostat. This is inherently one-way and implies agency, distinct from the generic `directed` flow.
- **`"dependent"`** — A structural or logical attachment/dependency where the target node is required for the source node to be valid or complete. Examples: a window mounted *on* a wall, a column resting *on* a slab, a fixture that inherits a material assignment from its host. Deleting the target may require the source to be flagged as orphaned or itself deleted (schema-level policy; the core's cascade-delete tree operates strictly on `parent_id`).

Relations are intentionally "soft" to preserve domain neutrality. Integrity checks such as target-existence validation, inverse-symmetry enforcement, forbidden cross-type relations, and cardinality limits are handled exclusively by `schema.json` profile rules.

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
  { "op": "replace", "path": "/chunks/sector_01/nodes/wall_42/coords/1", "value": 1500 },
  {
    "op": "add",
    "path": "/chunks/sector_01/nodes/door_12",
    "value": {
      "parent_id": "wall_42",
      "coords": [2000, 0, 0, 3000, 0, 0, 3000, 0, 2400, 2000, 0, 2400],
      "relations": [
        {
          "target": "wall_42",
          "target_chunk": "sector_01",
          "rel_type": "dependent"
        }
      ],
      "metadata": {
        "type": "Door",
        "material": "wood",
        "width_mm": 900,
        "swing": "right"
      }
    }
  }
]
```

### 7.1 Deletion and Cascading

When a delete operation targets a parent node, a sector root, or any element that owns child references, the Rust core MUST execute a **tree-like cascade delete** walk to deterministically remove all descendants before removing the target itself. This guarantees that no dangling pointers (stale `parent_id` or cross-references) remain anywhere in the graph. The cascade order is depth-first, children-first: leaves are removed first, then their parents, so every intermediate state of the graph remains referentially sound.

After every cascade (or any sequence of deletions), the core MUST inspect each affected chunk. If a chunk's JSON file becomes completely empty (i.e., it contains no nodes, edges, or remaining data entries after the operation), the physical chunk file is permanently removed from the ZIP container's `chunks/` directory. The `manifest.json` is then updated atomically to reflect this:

1. The corresponding entry under `manifest.chunks` (including its `path`, `hash`, and `version`) is deleted.
2. Any entries in `manifest.spatial_index` that pointed to nodes residing in the removed chunk are purged.
3. The `manifest.global_version` is incremented, and a new `state_hash` is recalculated (following the rule in Section 3) and written back to the manifest.

This chunk cleanup ensures the container stays lean and free of orphaned shard files, while keeping the routing table and state hash fully consistent.

## 8. Reference Chunk Example (Genesis + Child Node)

The following JSON shows a complete, conformant `chunk_genesis.json` file. It declares the global AABB bounds, a root space node whose `parent_id` is the reserved literal `"genesis"`, and a child wall node that demonstrates cross-chunk relations using all three mandatory relation fields (`target`, `target_chunk`, `rel_type`) and domain-specific metadata validated by `schema.json`.

```json
{
  "chunk_id": "chunk_genesis",
  "version": 1,
  "bounds": [0, 0, 0, 10000, 5000, 3000],
  "nodes": {
    "node_root_space": {
      "parent_id": "genesis",
      "coords": [0, 0, 0, 10000, 5000, 3000],
      "relations": [],
      "metadata": {
        "type": "RootSpace",
        "name": "Main Building Volume",
        "classification": "occupied"
      }
    },
    "node_wall_01": {
      "parent_id": "node_root_space",
      "coords": [100, 200, 0, 1100, 200, 0, 1100, 200, 2800, 100, 200, 2800],
      "relations": [
        {
          "target": "node_door_05",
          "target_chunk": "sector_02",
          "rel_type": "dependent"
        },
        {
          "target": "node_wall_02",
          "target_chunk": "chunk_genesis",
          "rel_type": "symmetric"
        },
        {
          "target": "node_switch_03",
          "target_chunk": "sector_02",
          "rel_type": "control"
        },
        {
          "target": "node_zone_circulation",
          "target_chunk": "sector_01",
          "rel_type": "directed"
        }
      ],
      "metadata": {
        "type": "Wall",
        "material": "concrete",
        "thickness_mm": 200,
        "fire_rating": "REI-120",
        "color": "#e8e4de"
      }
    }
  }
}
```

This example encodes four universal relation types on `node_wall_01`: a `dependent` attachment to a door it hosts, a `symmetric` adjacency to a neighbouring wall in the same chunk, a `control` link to a light switch in another chunk, and a `directed` entry/exit flow toward a circulation zone in a third chunk. The Rust core stores and routes these links faithfully, while `schema.json` decides whether a wall is *allowed* to depend on a door, control a switch, etc.
