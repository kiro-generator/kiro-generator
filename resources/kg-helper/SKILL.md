---
name: kg-helper
description: Help set up and manage kg (kiro-generator) TOML agent configurations. Use when users ask about kg, migrating JSON agents to kg, or bootstrapping a new kg setup.
license: MIT
compatibility: Requires access to user's kg configuration files in ~/.kiro/generators/ and .kiro/generators/
metadata:
  version: 0.3.0
  author: agents
---

## What is kg?

`kg` (kiro-generator) generates Kiro agent JSON files from composable TOML configurations. It solves the problem of managing agent configs by hand:

- **Composable**: Build agents from reusable templates
- **Hierarchical**: Global configs merge with local project configs
- **Deterministic**: Inherit and extend -- no gaps, no drift

## How kg Organizes Configuration

kg uses a two-layer system:

```
manifests/          # WHO exists and HOW they relate
  └── kg.toml       # Agent declarations + inheritance

agents/             # WHAT each agent does
  └── rust.toml     # Agent configuration
```

**Manifests** declare agents and relationships:
```toml
[agents]
default = { inherits = [] }
rust = { inherits = ["default"] }
```

**Agent files** define configuration:
```toml
# agents/rust.toml
description = "Rust development agent"
allowedTools = ["@rustdocs", "@cargo"]
```

Both layers exist at two locations:
- `~/.kiro/generators/` -- Global (all projects)
- `.kiro/generators/` -- Local (this project)

Local configs **merge with** global configs (not replace). This is what makes kg useful for real projects -- global tooling + local project context in one generated agent.

**Merge rules** (critical — understand these before editing configs):
- **Arrays** (allowedTools, resources): Combined
- **Objects** (toolsSettings, mcpServers): Deep merged
- **Scalars** (description, model): Child replaces parent

## Getting Started

If the user is new to kg, the entry point is:

```bash
kg init
```

This installs `~/.kiro/agents/kg-helper.json` (this agent). After that:

```bash
kg init --skeleton
```

Creates the TOML config scaffold at `~/.kiro/generators/`:

```
~/.kiro/generators/
├── manifests/
│   └── kg.toml
└── agents/
    └── default.toml
```

Both commands are idempotent — safe to re-run. If the user already has kg configured, skip straight to discovery.

## Step 1: Discovery

Before reading or modifying any config files, run discovery. `kg tree` shows which TOML files contribute to each agent and how they inherit:

```bash
# Show all agents
kg tree

# Single agent
kg tree rust

# Multiple agents
kg tree rust node
```

Example JSON output:
```json
{
  "rust": {
    "template": false,
    "output": ".kiro/agents/rust.json",
    "description": "Rust development agent",
    "sources": [
      {
        "type": "local-manifest",
        "path": ".kiro/generators/manifests/rust.toml",
        "modified_fields": ["description", "inherits"]
      },
      {
        "type": "global-manifest",
        "path": "/home/user/.kiro/generators/manifests/base.toml",
        "modified_fields": ["inherits"]
      },
      {
        "type": "global-file",
        "path": "/home/user/.kiro/generators/agents/rust.toml",
        "modified_fields": ["description", "prompt", "nativeTools.shell"]
      }
    ],
    "inherits": ["kg-resources", "default"],
    "resolved_chain": ["kg-resources", "cli", "resources", "knowledge", "git", "default"]
  }
}
```

**Source types:**
- `global-manifest` / `local-manifest` -- Agent declared inline in a manifests/*.toml file
- `global-file` / `local-file` -- Dedicated agent config file in agents/ directory

**`modified_fields`** lists which fields each source file contributes. Use this to go directly to the right file when a user asks "where does this shell command / MCP server / resource come from?" — no guessing required.

**`resolved_chain`** is the full flattened inheritance DAG in merge order. This differs from `inherits` (direct parents only). If `rust` inherits `["kg-resources", "default"]` but `default` itself inherits `["cli", "resources", "knowledge", "git"]`, `resolved_chain` shows all of them in the order they are merged. Use this when debugging unexpected values — find which ancestor in the chain contributes the field via `modified_fields`.

**Use the `sources` array to know exactly which files to read or edit.**

## Step 2: Edit Agent Configs

To modify an agent's configuration:

1. Run `kg tree <agent-name>` to find which TOML files define it
2. Read the `sources` array -- it tells you exactly which files to edit
3. Edit the TOML file(s) directly

For available TOML fields, see `references/schemas.md` or run `kg schema agent`.

## Step 3: Validate Changes

After editing TOML files, validate the configuration:

```bash
kg validate
```

**Scope detection:** kg automatically determines whether to operate on global or local agents based on your current directory:

- **In a project with `.kiro/generators/`**: Commands default to local scope
- **Outside a project**: Commands default to global scope
- **Force global scope**: Use `--global` flag when you're in a project but need to work with global agents

**Rule of thumb:** If `kg tree <agent>` shows `"type": "global-file"` or `"type": "global-manifest"` in sources, and you're in a project directory, use `--global`.

```bash
# When editing global agents from within a project:
kg validate --global

# When editing local agents:
kg validate
```

**Success output:**
```
✓ Validated 3 agents (2 changed, 1 unchanged)
```

If validation fails, kg reports parse errors with file path and line number.

## Step 4: Preview Changes

Always preview what will change before generating:

```bash
kg diff

# Agent-friendly output (no color, dot-notation paths only)
kg diff --format agent

# When editing global agents from within a project:
kg diff --global
```

**Output shows:**
- File paths of affected agents
- Changed paths in dot notation (e.g., `knowledge.3: +`, `description: "old" -> "new"`)
- Summary of changed vs unchanged agents

**If diff shows no output:**
Either there are no changes, or you may be checking the wrong scope. Verify with `kg tree <agent>` that you're editing the right files and using the correct `--global` flag if needed.

## Step 5: Generate Agent Files

Apply the changes:

```bash
kg generate

# When editing global agents from within a project:
kg generate --global
```

Generated files:
- Global agents: `~/.kiro/agents/<agent-name>.json`
- Local agents: `.kiro/agents/<agent-name>.json`

## Configuration Resolution Order

When resolving agent `rust`, kg searches (lowest to highest precedence):

1. `~/.kiro/generators/manifests/*.toml` - Global declarations
2. `~/.kiro/generators/agents/rust.toml` - Global config
3. `.kiro/generators/manifests/*.toml` - Local declarations
4. `.kiro/generators/agents/rust.toml` - Local config

All found configs merge together. Use `kg tree rust` to see which sources apply.

## Error States

- **No agents found**: `kg tree` returns empty JSON object `{}` (exit 0). Consumers should check for an empty object.
- **Named agent not found**: `kg tree nonexistent` returns empty JSON object `{}` (exit 0). The requested agent key will be absent from the response.
- **Invalid TOML**: `kg validate` reports parse errors with file path and line number

**Recovery:** If validation fails, fix the parse error shown in the output. If `kg generate` produces unexpected results, run `kg tree <agent>` to trace which source file contributes the unexpected value — check `modified_fields` on each source to pinpoint the exact file.

**If a `kg diff` path needs tracing** (e.g. you see a JSON field change and want to know which TOML field drives it), load `assets/mappings.json` to look up the TOML→Kiro JSON mapping and the jq path to inspect the generated value directly.

## Reference Documents

For detailed guidance on specific topics, load these as needed:

- **`references/migration.md`** -- Migrating from hand-written JSON agents to kg. Covers user interview framework (security posture, structure preference, scope), decision framework, and example layouts. Load when helping users set up kg for the first time or convert existing agents.
- **`references/templates.md`** -- Template categories (Permission, Capability, Context, Lifecycle), real-world examples, composition patterns, subagent templates. Load when helping users design or refactor their template hierarchy.
- **`references/schemas.md`** -- TOML to JSON field mappings, schema files in assets/, jq recipes for inspecting validated output. Load when verifying field names or exploring available configuration options. To trace a specific diff path back to its TOML field, use `assets/mappings.json` for machine-readable lookup.

## Keeping This Skill Updated

This skill is shipped with the `kg` package at `/usr/share/doc/kiro-generator/kg-helper/SKILL.md`. To update, install the latest `kg` version.
