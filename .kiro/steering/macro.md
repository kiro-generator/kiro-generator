# Macro and Mapping Pipeline

## Purpose

The `kg-macro` crate owns a small but opinionated proc-macro used to keep Kg agent schema documentation in sync with a machine-readable mapping output. The macro writes mapping metadata into Rust doc attributes, which become JSON Schema `description` fields, and `build_mappings` later parses those descriptions into the `resources/kg-helper/assets/mappings.json` payload. The mappings data is a troubleshooting aid for agents using the `kg-helper` skill (`resources/kg-helper/SKILL.md`), making it easy to discover TOML Kg fields and where they land in Kiro’s JSON agent config.

## Where It Lives

- Macro implementation: `kg-macro/src/lib.rs`
- Schema extraction: `src/schema.rs`
- Struct using the macro: `src/kg_config/agent_file.rs`
- Generated mapping output: `resources/kg-helper/assets/mappings.json`

## Macro: `#[kg_schema]`

`#[kg_schema]` is an attribute macro applied to `KgAgentFileDoc`. It does three main things.

1. It normalizes struct-level `derive` and `facet` attributes.
2. It converts each `#[kg_mapping(...)]` on fields into a single `#[doc = "..."]` string.
3. It enforces the mapping arguments and fails compilation when they are invalid.

### Struct-Level Attribute Handling

- The macro removes any existing `#[derive(...)]` and `#[facet(...)]` attributes from the struct, then re-emits them at the top.
- If there were no explicit `derive` attributes, it injects:
  `#[derive(Facet, Clone, Default)]`
- If there were no explicit `facet` attributes, it injects:
  `#[facet(deny_unknown_fields, skip_all_unless_truthy, default)]`

This guarantees a consistent default for the schema-bearing struct unless callers deliberately override it.

### Field-Level Mapping Handling

For each named field on the struct, the macro:

- Looks for `#[kg_mapping(kiro_schema_path = "...", description = "...")]`.
- Removes the `kg_mapping` attribute from the field.
- Removes any existing `#[doc = "..."]` attributes on the field.
- Appends a new doc string in the format:

```
{description} | kiro_schema_path = {kiro_schema_path}
```

The delimiter is defined in `kg-macro/src/lib.rs` as:

```
const KG_MAPPING_DELIM: &str = " | kiro_schema_path = ";
```

If multiple `kg_mapping` attributes exist or the args are malformed, the macro collects and emits a compile error rather than silently ignoring issues.

### Parsing and Errors

`kg_mapping` only accepts:

- `kiro_schema_path = "..."`
- `description = "..."`

Missing either argument or providing unknown arguments produces a compile error. Duplicate arguments also error.

## Macro Hook: `kg_mapping_delim!()`

`kg_mapping_delim!()` is a simple proc-macro function that expands to the delimiter string. `src/schema.rs` uses it to keep parsing logic in sync with the macro definition without a shared crate dependency:

```
const KG_MAPPING_DELIM: &str = kg_macro::kg_mapping_delim!();
```

## Schema to Mapping: `build_mappings`

`build_mappings` consumes a `JsonSchema` (the schema for `KgAgentFileDoc`) and produces a mapping table keyed by Kg schema path. Key behaviors:

- The schema is generated via `schema_for::<KgAgentFileDoc>()`.
- Each property is inspected for a `description` field.
- If the description is empty, the property is skipped.
- If the description does not contain the delimiter, it errors out.
- It splits the description on the delimiter, trims both sides, and stores:
  - `description` (left side)
  - `kiro_schema_path` (right side)

Each mapping entry is keyed as `#/properties/{field_name}`.

The function returns a `BTreeMap` with a single group name:

- `kg_to_kiro`: the mapping table for Kg field paths to Kiro schema paths.

## Output: `resources/kg-helper/assets/mappings.json`

`resources/kg-helper/assets/mappings.json` is the serialized output from `build_mappings` and is used by the kg-helper skill. Every mapping comes from a `#[kg_mapping]` field on `KgAgentFileDoc`.

Example (abridged):

```
{
  "kg_to_kiro": {
    "#/properties/allowedTools": {
      "kiro_schema_path": "#/properties/allowedTools",
      "description": "List of tools the agent is explicitly allowed to use"
    }
  }
}
```

## Why This Exists

The pipeline keeps a single source of truth:

- Human-readable field descriptions live on `KgAgentFileDoc` fields.
- The macro injects those descriptions into JSON Schema so they are visible to tooling.
- `build_mappings` repurposes the same descriptions to generate a deterministic Kg-to-Kiro mapping file without a second manual list.

This prevents drift between schema docs and the mapping helper consumed by the kg-helper skill.
