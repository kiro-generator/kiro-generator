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
    "description": "Rust development agent",
    "sources": [
      {"type": "local-manifest", "path": ".kiro/generators/manifests/kg.toml"},
      {"type": "global-manifest", "path": "/home/user/.kiro/generators/manifests/base.toml"},
      {"type": "global-file", "path": "/home/user/.kiro/generators/agents/lang/rust.toml"}
    ],
    "inherits": ["default", "kg-resources"]
  }
}
```

**Source types:**
- `global-manifest` / `local-manifest` -- Agent declared inline in a manifests/*.toml file
- `global-file` / `local-file` -- Dedicated agent config file in agents/ directory

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

**Recovery:** If validation fails, fix the parse error shown in the output. If `kg generate` produces unexpected results, run `kg tree <agent>` to trace which source file contributes the unexpected value.

## Reference Documents

For detailed guidance on specific topics, load these as needed:

- **`references/templates.md`** -- Template categories (Permission, Capability, Context, Lifecycle), real-world examples, composition patterns, subagent templates. Load when helping users design or refactor their template hierarchy.
- **`references/bootstrap.md`** -- Migration from hand-written JSON agents. Covers analysis.json schema, user interview framework, decision framework, example layouts. Load when helping users run `kg bootstrap` or set up kg for the first time.
- **`references/schemas.md`** -- TOML to JSON field mappings, schema files in assets/, jq recipes for inspecting validated output. Load when verifying field names or exploring available configuration options.

## Keeping This Skill Updated

This skill is embedded in the `kg` binary and installed by `kg bootstrap`. To update, install the latest `kg` version and re-run `kg bootstrap`.
