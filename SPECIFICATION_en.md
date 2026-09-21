# AAGL Technical Specification — Container Layer

**Standard Version:** 0.1.0
**Status:** Draft / Storage-System Specification
**Scope:** This document defines ONLY the physical file-layout, routing, hashing, chunking, graph-structural invariants, mutation mechanics, and view-resolution MECHANISM of the `.aagl` container format. It explicitly does **not** define:
- What semantic "types" a node may have;
- What keys a node's `metadata` object may or must contain;
- What keys a style rule may accept, or what those keys mean to a renderer.

Those concerns belong to a **profile** (also called a *protocol* or *domain schema*) referenced from `manifest.json`. Multiple higher-level protocols (architectural floor plans, PCB layouts, network topologies, mechanical assemblies, …) can share this one container format by plugging different profile definitions.

---

## 1. Overview & Objectives

**AAGL (Agentic Architectural Graph Language)** is an embedded, high-performance spatial graph storage engine implemented in Rust. It stores, queries, and mutates complex spatial/structural graphs deterministically, with version-control semantics optimised for concurrent autonomous AI agents.

Two non-negotiable layering principles:
1. **Format/protocol split.** The container format (this document) is type-blind. Semantic contracts for `metadata` and `view`-style field dictionaries are declared in a **profile** chosen by the author and routed through `manifest.profile`.
2. **Data/view split.** All files under `chunks/` carry the graph model and nothing else. All presentation strategies live under `views/` and are opaque to the core engine. Cosmetic edits must never dirty data hashes or trigger OCC conflicts.

---

## 2. Container Format (`.aagl`)

An AAGL project is a single portable file with extension `.aagl`. Internally it is a standard **ZIP archive**; readers MUST use the ZIP Central Directory for O(1) random access to individual files without streaming the entire archive.

### Directory Layout (required)
```text
project.aagl (ZIP)
├── manifest.json             # Routing table + version anchors + profile declaration (REQUIRED)
├── schema.json               # Profile-owned node/metadata validation rules (REQUIRED, opaque to core)
├── chunks/                   # Sharded sub-graphs — graph data ONLY (REQUIRED, one entry at minimum)
│   └── chunk_genesis.json    # Mandatory identity-root chunk (REQUIRED)
├── views/                    # Presentation strategies — NEVER mutate graph data (OPTIONAL)
│   └── <theme_id>.json       # Each theme is a standalone file
└── assets/                   # Binary attachments (textures, point clouds, rasters) (OPTIONAL)
    └── ...
```

> **Invariant:** Nothing under `chunks/` is allowed to reference or embed view data. Nothing under `views/` is allowed to duplicate or override any of the four structural node fields (`parent_id`, `coords`, `relations`, `metadata`).

---

## 3. Global Manifest (`manifest.json`)

`manifest.json` is the authoritative catalog for the whole container. Every field in this section is part of the container-layer contract (and therefore known to the Rust core). Semantic fields — anything about node types, style dictionaries, coordinate layout interpretations — live inside `manifest.profile` and are treated as an opaque blob by the core engine.

```json
{
  "spec_version": "0.1.0",
  "project_name": "sample_project",
  "state_hash": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "global_version": 12,
  "profile": {
    "id": "my_domain_protocol_v3",
    "schema_path": "./schema.json",
    "view_type_key": "metadata.profile_key",
    "view_style_schema": {
      "_comment": "Opaque profile-defined contract describing which style keys a renderer should expect in by_type/by_id/fallback. The container layer does not parse this — it only stores and hashes it."
    },
    "core_hints": {
      "relation_codes": ["directed", "symmetric", "control", "dependent"],
      "coords_layout_registry": {
        "_comment": "Opaque to core; used by profile-level validators to assign semantic interpretation to the integer coords array per node."
      }
    },
    "extensions": {}
  },
  "units": {
    "base_unit": "mm",
    "internal_resolution_to_meters": 0.001
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
  "views": {
    "active_theme": "default_minimal",
    "registered": {
      "default_minimal": {
        "path": "./views/default_theme.json",
        "hash": "sha256:7a9f3e...",
        "version": 2
      }
    }
  },
  "spatial_index": {
    "node_0001": "chunk_genesis",
    "node_0002": "chunk_genesis"
  }
}
```

### 3.1 `manifest.profile` — where ALL type/style knowledge lives

This is the single author-configurable entry point that answers every semantic question about the project. The AAGL container format itself knows nothing about any of this.

| Field                | Type     | Meaning (profile-owned — core treats as opaque blob except for path routing) |
|----------------------|----------|-------------------------------------------------------------------------------|
| `id`                 | string   | Stable identifier of the higher-level protocol in use, e.g. `arch_floorplan_v2`, `pcb_oenull_v1`, `generic_spatial_v0`. Readers can look up the protocol definition outside the container if desired. |
| `schema_path`        | string   | Path (inside the ZIP) to the profile-owned JSON Schema file that validates node shapes, `metadata` contents, `coords` layout rules, allowed `relations` — everything semantic. The Rust core itself does **not** run this schema; the profile loader does. |
| `view_type_key`      | string   | **Which dotted field-path on a node provides the discriminator used as the key in `views/<theme>.json#/by_type`.** Example: `"metadata.profile_key"` means `by_type[node.metadata.profile_key]`. By making this configurable in the manifest, the container format never hard-codes a `type` field name — protocols can use `"class"`, `"kind"`, `"layer_code"`, compound discriminators, anything. |
| `view_style_schema`  | object   | Profile-defined contract describing *which keys a style rule object may contain* and their semantics (colour formats, hatch codes, label-placement enums, renderer hints). The container layer never reads this; it is honoured by renderers/profile-loaders only. |
| `core_hints`         | object   | A narrow, optional set of overrides the container core MAY read if present. Currently only `relation_codes` is standardised (list of allowed `rel_type` strings; defaults to `[directed,symmetric,control,dependent]` if absent). |
| `extensions`         | object   | Free-form bag for future profile-level knobs without breaking container layout. Core MUST ignore everything in this object. |

### 3.2 State Hash / OCC Anchor
When computing `state_hash` the `state_hash` field itself MUST be temporarily set to `""` (or omitted) before hashing, then the resulting digest written back, per:
```
temp = manifest.clone();
temp.state_hash = "";
digest = sha256(canonical_json(temp));
manifest.state_hash = "sha256:" + hex(digest);
```

### 3.3 View Registration
- `manifest.views.registered` entries follow the same `{path, hash, version}` triple as chunks for bookkeeping consistency.
- **View version bumps do NOT increment `manifest.global_version` and do NOT change `state_hash`.** This is required so that theming tweaks (selection highlight, accent swap, print vs screen) do not invalidate ongoing concurrent OCC writes to the data graph.
- `active_theme` is a renderer hint. The core never reads, enforces, or mutates it.

---

## 4. Coordinate System (container-level)

- `coords` arrays in every node MUST contain only native 64-bit signed integers. Floating-point values are forbidden anywhere in a `coords` array at the container layer (profiles MAY of course embed float metadata *inside* the `metadata` object if a use case demands it).
- Global scale/unit semantics are declared in `manifest.units` and are otherwise opaque to the core.

---

## 5. Chunks (`chunks/*.json`)

### 5.1 Granularity & Isolation
- Data is split into chunks targeting 50–100 KB each.
- Each chunk is an independent sub-graph with internal `parent_id` tree and cross-reference pointers through `relations`.
- Each chunk JSON has exactly three structural top-level fields: `chunk_id` (string, matches manifest key), `version` (monotonic integer ≥ 1), and `nodes` (object of `node_id → node`). `bounds` (6-element `[x_min,y_min,z_min,x_max,y_max,z_max]` i64 array) is OPTIONAL on any chunk but REQUIRED on `chunk_genesis` (see 5.2).

### 5.2 Genesis Chunk
- Every project must contain exactly one chunk registered under id `chunk_genesis` at path `./chunks/chunk_genesis.json`.
- It MUST declare a 6-element `bounds` field defining the global spatial envelope of the project.
- It MUST contain at least one root node whose `parent_id` equals the literal string `"genesis"`.
- Its `chunk_id` identity is immutable: it can never be deleted, renamed, or unregistered from `manifest.chunks` (though its `nodes` contents may evolve).

### 5.3 Node Contract (container layer — four required keys only)

Every node, regardless of profile, MUST expose exactly four required structural fields. **These four fields (and only these four) are known to the Rust core.** Any other top-level keys on a node are profile-defined extensions and MUST be ignored by the core.

| Field       | Type              | Container-layer semantics (profile-blind)                                                                 |
|-------------|-------------------|------------------------------------------------------------------------------------------------------------|
| `parent_id` | `String`          | Hierarchical ownership pointer. Root nodes use the literal `"genesis"` string (**never** `null`). The core walks this tree for cascade-delete. Semantic "meaning" is profile-defined. |
| `coords`    | `Vec<i64>`        | Flat array of 64-bit signed integers. Core treats it as an opaque, order-preserving blob. Length, layout, and interpretation are solely defined by `manifest.profile`. |
| `relations` | `Vec<Relation>`   | Flat array of cross-node links. See 5.4. Core enforces only structural invariants of Relation entries, not their semantics. |
| `metadata`  | `JSON Object`     | Fully opaque profile-owned object. **Container layer requires zero keys and forbids zero keys here.** All shape rules (requiring a discriminator field, enforcing field types, even banning visual keys from metadata — should a profile choose to do so) come from `manifest.profile.schema_path`. |

> **No hardcoded `type` field required.** The container does not need a node to carry a `metadata.type` key. Where the view layer's `by_type` map looks up its discriminator is entirely determined by the dotted JSON path in `manifest.profile.view_type_key`.

### 5.4 Relation Contract (container layer — three required keys only)

Each entry in `relations` MUST have exactly these three fields. Their semantics are profile-defined; the core only validates structural presence and can route via `target_chunk`.

| Field          | Type     | Container-layer meaning |
|----------------|----------|-------------------------|
| `target`       | `String` | `node_id` of the destination. |
| `target_chunk` | `String` | `chunk_id` hosting the destination — enables cross-chunk routing via `manifest.spatial_index`. |
| `rel_type`     | `String` | One of a set of allowed codes. The container default set is `["directed", "symmetric", "control", "dependent"]`, but a profile may override this list by setting `manifest.profile.core_hints.relation_codes` to a different string array. |

### 5.5 Reference (Profile-Blind) Genesis Chunk

```json
{
  "chunk_id": "chunk_genesis",
  "version": 1,
  "bounds": [0, 0, 0, 10000, 5000, 3000],
  "nodes": {
    "node_0001": {
      "parent_id": "genesis",
      "coords": [0, 0, 0, 10000, 5000, 3000],
      "relations": [],
      "metadata": {
        "profile_key": "container_root",
        "label": "Global Envelope",
        "custom": { "foo": "bar" }
      }
    },
    "node_0002": {
      "parent_id": "node_0001",
      "coords": [100, 200, 0, 1100, 200, 0, 1100, 200, 2800, 100, 200, 2800],
      "relations": [
        { "target": "node_0009", "target_chunk": "sector_02", "rel_type": "dependent" },
        { "target": "node_0003", "target_chunk": "chunk_genesis", "rel_type": "symmetric" },
        { "target": "node_0042", "target_chunk": "sector_02", "rel_type": "control" },
        { "target": "node_0017", "target_chunk": "sector_01", "rel_type": "directed" }
      ],
      "metadata": {
        "profile_key": "linear_feature_A",
        "props": { "p1": 200, "p2": "sample" }
      }
    }
  }
}
```

Notice there are no references to `Wall`, `Door`, `fill_color`, `hatch` anywhere. Those are not part of the container format.

---

## 6. Optimistic Concurrency Control (OCC) — Container Layer

1. Every core write supplies the `base_version` / `base_chunk_hash` it was built on.
2. Core compares supplied values to the current manifest/chunk anchors.
3. On match → mutation applied atomically, chunk `version` / `manifest.global_version` increment, `state_hash` recomputed.
4. On mismatch → transaction rejected; caller must rebase.
5. View theme edits (under `views/`) are **excluded** from OCC conflict detection on the data graph: renderers replace them freely and update `manifest.views.registered[theme_id].{hash,version}` without touching `global_version` or `state_hash`.

---

## 7. Mutation Protocol: JSON Patch

Mutations to the data graph are applied as atomic RFC 6902 JSON Patch sequences.

Scope rule (core API): the core's `apply_patch` entry point accepts patches targeting **only** data-bearing paths (`/chunks/*`, `/spatial_index/*`, `/units/*`, `/profile/core_hints/relation_codes` if profile allows). Patches to `/views/*` or `/profile/view_style_schema` are client-side profile operations; they are applied and registered through the view/theme manager, **not** through the core data-mutation endpoint — precisely so they cannot trigger data-level OCC conflicts.

### 7.1 Cascade Delete
When a node is removed, the core performs a deterministic depth-first, children-first walk of `parent_id` descendants, removing all transitive children before the parent. If a chunk file ends completely empty after deletion, it is physically removed from the ZIP and its entry is purged from `manifest.chunks` and `manifest.spatial_index`; `global_version` increments; `state_hash` recomputes.

---

## 8. View Layer: Mechanism Only

The container standard defines the **mechanism** of view theming (file placement, registration in manifest, three-tier resolution order, per-field shallow merge semantics). It does **not** define which keys live inside a style rule, or the allowed values of any style enum.

### 8.1 Resolution Mechanism — Three Tiers, Per-Key Shallow Merge

Every theme JSON under `views/` is required to have the following top-level structural keys (from `schemas/view-theme.json`): `theme_id`, `theme_name`, `spec_version`. It additionally declares zero or more of: `fallback`, `by_type`, `by_id`.

When a renderer resolves the style of a node, it performs:

```
         Highest priority
               ▼
         ┌─────────────┐
         │   by_id     │   Keyed by exact node_id string.
         └──────┬──────┘
                │  key missing? fall through
                ▼
         ┌─────────────┐
         │   by_type   │   Keyed by discriminator extracted using
         └──────┬──────┘   manifest.profile.view_type_key dotted path.
                │  key missing? fall through
                ▼
         ┌─────────────┐
         │  fallback   │   theme-wide defaults.
         └──────┬──────┘
                │  key missing? fall through
                ▼
         Renderer's own hardcoded ultimate defaults.
```

Merging is **shallow and per-key**: if a higher-priority rule object defines key K, it wins; if it omits key K, the value from the next tier is used. Renderers MUST NOT replace the entire lower-priority rule object wholesale (which would throw away unspecified but meaningful inherited keys).

### 8.2 Keys inside a style rule object — 100 % profile-owned

The names, types, and semantics of individual keys inside `by_type[*]`, `by_id[*]`, and `fallback` are fully determined by `manifest.profile.view_style_schema`. The container layer does not constrain or enumerate them in this document. Examples which a profile might define (non-normative — purely illustrative):
- profile `arch_floorplan_v2` could define `{stroke_color_hex, line_dash_code, hatch_ordinal, label_text_path, label_anchor_ordinal}`
- profile `pcb_oenull_v1` could define `{pad_mask_layer, copper_net_class_code, silk_opacity, drill_plating_bit}`
- profile `network_topology_v4` could define `{bg_gradient_id, stroke_net_color, icon_glyph_id, tooltip_path_expression}`

### 8.3 Reference Theme Skeleton (mechanism only)

A minimal `views/default_theme.json` skeleton is shipped with the reference implementation solely to demonstrate the three-tier structural shape. Its style-rule keys are placeholders only; a real profile replaces them with real profile-defined contracts.

---

## 9. Layering Boundary — Summary Checklist

To verify an implementation respects the type-blind/storage-only scope of THIS spec:

| Concern                                           | Decided by                   | Decided where                    |
|---------------------------------------------------|------------------------------|----------------------------------|
| ZIP layout, `chunks/`, `views/`, `assets/`        | Container format             | This document, § 2               |
| 4 required node fields, 3 required relation fields | Container format           | § 5.3, § 5.4                     |
| OCC, hashing, chunk cascade delete                | Container format             | § 3.2, § 6, § 7.1                |
| 3-tier view resolution structure                  | Container format             | § 8.1                            |
| **What fields go in `metadata`**                  | **Profile / Protocol**       | `manifest.profile.schema_path` → `/schema.json` |
| **What a "node type" is, and its discriminator name** | **Profile / Protocol**  | `manifest.profile.view_type_key` + schema |
| **Names/meanings of style-rule keys**             | **Profile / Protocol**       | `manifest.profile.view_style_schema` |
| Coords array layout per discriminator             | Profile / Protocol           | schema + `profile.core_hints` (opaque) |
| Allowed `rel_type` codes override                 | Profile / Protocol (hint)    | `manifest.profile.core_hints.relation_codes` |
