# Diff Pipeline

## Overview

`kg diff` compares generated agent config (from TOML sources) against existing Kiro agent JSON files on disk. It normalizes both sides into `NormalizedAgent` for stable, deterministic comparison using `rediff::FacetDiff`.

## Data Flow

1. `diff_agents()` iterates non-template agents from `merge()`
2. For each agent, `compute_diff()` in `src/generator/mod.rs`:
   - Reads existing JSON from the destination path (`.kiro/agents/` or `~/.kiro/agents/`)
   - Parses it into `KiroAgent` via `facet_json`
   - Calls `.normalize()` on both existing and generated `KiroAgent`
   - Runs `rediff::FacetDiff` on the two `NormalizedAgent` structs
3. Output is formatted per `DiffFormatArg` (Full, Compact, Plain, Agent)

## NormalizedAgent (src/kiro/diff.rs)

Converts `KiroAgent` into a struct optimized for stable diffing:

- **Sorts** all collections: `tools`, `allowed_tools`, `resources`, `knowledge`, `other_tools`
- **Deduplicates** resources via `HashSet` before sorting
- **Deserializes** known native tools (`shell`, `aws`, `read`, `write`, `subagent`) into concrete typed structs for field-level diffs
- **Collects** unknown tool names into `other_tools` (presence only, no settings)

## Known Gaps

These `KiroAgent` fields are NOT included in `NormalizedAgent` and therefore invisible to diff:

_None — all fields are now covered._

Changes to any of these fields will produce "No changes" from `kg diff`.

## Silent Failures

- If a native tool setting (e.g. `shell`) fails to deserialize into its concrete type, it silently becomes `None` — no warning logged. Only malformed knowledge resources log a `tracing::warn!`.
- The `_filter` parameter in `diff_agents()` is accepted but unused — agent name filtering is not implemented.

## Format Modes

| DiffFormatArg | Colors | Style | Use case |
|---|---|---|---|
| `Full` | yes | structural, all fields shown | human review |
| `Compact` | yes | dot-notation, changes only | default / generate --diff |
| `Plain` | no | structural, all fields shown | piping to files |
| `Agent` | no | dot-notation, changes only | agent consumption / scripts |

## Key Types

- `KiroAgent` — raw agent struct matching Kiro JSON schema (`src/kiro/mod.rs`)
- `NormalizedAgent` — diff-optimized representation (`src/kiro/diff.rs`)
- `AgentDiff` — enum: `New`, `Changed(String)`, `Same` (`src/generator/mod.rs`)
- `DiffFormatArg` — output format enum (`src/output.rs`)
