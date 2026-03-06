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

## Lexicon

- **Manifest** — a TOML file under `manifests/`. Declares agents, their relationships, and inline config. The only place `template = true` is valid.
- **Agent file** — a TOML file under `agents/<name>.toml`. Defines configuration for a named agent. `template = true` is **not** valid here.
- **Template** — an agent declared with `template = true` in a manifest. Never produces a JSON output file. Exists only to be inherited.
- **Concrete agent** — any non-template agent. Produces a JSON file in `.kiro/agents/` or `~/.kiro/agents/`.
- **Scope** — global (`~/.kiro/generators/`) vs local (`.kiro/generators/`). Local merges on top of global.
- **Resolved chain** — the full flattened inheritance DAG in merge order, distinct from direct `inherits` (which lists only immediate parents).

## How kg Organizes Configuration

kg uses a two-layer system, but the layers are flexible:

```
manifests/          # Agent declarations — WHO exists, HOW they relate
  └── base.toml     # Filenames are arbitrary (organize however you want)

agents/             # Agent configurations — dedicated files for larger configs
  └── rust.toml     # Filename IS the agent name
```

**Manifests** declare agents. They can be minimal or contain full agent config inline:

```toml
# Minimal — just declares relationships
[agents]
rust = { inherits = ["default"] }

# Full inline config — no separate agent file needed
[agents.resources]
template = true
resources = ["file://README.md", "file://~/.config/agents/resources/me.md"]
```

**Templates** (`template = true`) are reusable configuration fragments that exist only to be inherited by other agents. They are never written to disk as JSON files — they have no output. Use them to define shared tools, resources, MCP servers, or permissions that multiple agents compose together. Template status is never inherited: if agent `rust` inherits from template `cli`, `rust` is still a concrete agent that produces a JSON file.

**Agent files** (`agents/*.toml`) are for when an agent's config is large enough to warrant its own file. The filename determines the agent name — `agents/rust.toml` configures the `rust` agent.

**Both approaches are equivalent.** kg doesn't dictate how you organize — a 3-line template can live inline in a manifest, a complex agent can get its own file. If both exist, they merge.

Both layers exist at two locations:
- `~/.kiro/generators/` -- Global (all projects)
- `.kiro/generators/` -- Local (this project)

Local configs **merge with** global configs (not replace). This is what makes kg useful for real projects -- global tooling + local project context in one generated agent.

**Merge rules** (critical — understand these before editing configs):
- **Arrays** (allowedTools, resources): Combined
- **Objects** (toolsSettings, mcpServers): Deep merged
- **Scalars** (description, model): Child replaces parent

## Scope

kg automatically determines scope based on your current directory:

- **In a project with `.kiro/generators/`**: Commands default to local scope
- **Outside a project**: Commands default to global scope
- **`--global` flag**: Forces global scope when you're in a project directory

If `kg tree <agent>` shows `"type": "global-file"` or `"type": "global-manifest"` in sources and you're in a project directory, pass `--global` to `kg diff`, `kg generate`, etc.

## Step 1: Discovery

Before reading or modifying any config files, run discovery.

### Summary view (default)

`kg tree` with no arguments shows a summary table of all agents and templates:

```bash
# Summary table of all agents + templates
kg tree

# Agents only, hide templates
kg tree --no-templates

# Machine-readable summary
kg tree -f json
```

The table shows agent name, description, and direct parents (inherits). Templates are grouped at the bottom.

### Single agent detail

`kg tree <name>` outputs full JSON detail for a specific agent, including sources, modified fields, and resolved inheritance chain:

```bash
# Single agent detail (JSON)
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

### Reverse dependency lookup

`kg tree --invert` shows what depends on a given agent or template:

```bash
# What inherits from "default" (directly or transitively)?
kg tree --invert default

# Used/unused template report (no agent name)
kg tree --invert

# Machine-readable
kg tree --invert -f json
kg tree --invert default -f json
```

`kg tree --invert <name>` shows every agent that inherits from `<name>`, with the full inheritance path. Use this to assess blast radius before modifying a template or parent agent.

`kg tree --invert` (no name) reports which templates are actively used and which are orphaned. Use this to find dead templates that can be removed.

## Step 2: Edit Agent Configs

To modify an agent's configuration:

1. Run `kg tree <agent-name>` to find which TOML files define it
2. Read the `sources` array — it tells you exactly which files to edit
3. Edit the TOML file(s) directly

Common patterns (TOML field names differ from Kiro JSON — see `references/schemas.md` for full mappings):

```toml
# Add allowed/denied shell commands
[nativeTools.shell]
allow = ["yarn install .*", "cargo .*"]
deny = ["rm -rf .*"]

# Add an MCP server
[mcpServers.my-server]
command = "npx"
args = ["-y", "@my/mcp-server"]
```

For all available fields: `kg schema agent` or load `references/schemas.md`.

## Step 3: Preview Changes

After editing TOML files, always diff before generating. `kg diff` validates the TOML and shows what will change in the generated JSON:

```bash
# Agent-friendly output (no color, dot-notation paths only) -- prefer this
kg diff --format agent

# Human-friendly with color
kg diff

# When editing global agents from within a project:
kg diff --format agent --global
```

**Output shows:**
- File paths of affected agents
- Changed paths in dot notation (e.g., `shell.denied_commands.31: + "npm publish .*"`)
- Summary of changed vs unchanged agents

**If diff shows parse errors:** Fix the TOML error at the reported file and line number, then re-run diff.

**If diff shows many changes across multiple agents:** This is normal. Common causes:
- Previous TOML edits that were never generated (accumulated drift)
- A template or parent agent was modified, and changes ripple through all inheritors via `resolved_chain`

Use `kg tree <agent>` and check `resolved_chain` + `modified_fields` to trace which ancestor introduced a change.

**If diff shows no output:** Either there are no changes, or you may be checking the wrong scope. Verify with `kg tree <agent>` that you're editing the right files and using the correct `--global` flag if needed.

## Step 4: Generate Agent Files

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

- **No agents found**: `kg tree` shows an empty table. `kg tree <name> -f json` returns `{}` (exit 0).
- **Named agent not found**: `kg tree nonexistent` produces no output in table/plain mode (exit 0). In JSON mode (`-f json`), it returns `{}`.
- **Invalid TOML**: `kg diff` reports parse errors with file path and line number

**Recovery:** If diff reports a parse error, fix the error at the reported location and re-run diff. If `kg generate` produces unexpected results, run `kg tree <agent>` to trace which source file contributes the unexpected value — check `modified_fields` on each source to pinpoint the exact file.

**If a `kg diff` path needs tracing** (e.g. you see a JSON field change and want to know which TOML field drives it), load `assets/mappings.json` to look up the TOML→Kiro JSON mapping and the jq path to inspect the generated value directly.

## Reference Documents

For detailed guidance on specific topics, load these as needed:

- **`references/migration.md`** -- Migrating from hand-written JSON agents to kg. Covers user interview framework (security posture, structure preference, scope), decision framework, and example layouts. Load when helping users set up kg for the first time or convert existing agents.
- **`references/templates.md`** -- Template categories (Permission, Capability, Context, Lifecycle), real-world examples, composition patterns, subagent templates. Load when helping users design or refactor their template hierarchy.
- **`references/schemas.md`** -- Load when writing TOML fields beyond shell rules and MCP servers (hooks, knowledge, subagents, toolSettings). Contains full TOML→JSON field mappings, jq recipes for inspecting generated output, and schema discovery commands.

## Keeping This Skill Updated

This skill is shipped with the `kg` package at `/usr/share/doc/kiro-generator/kg-helper/SKILL.md`. To update, install the latest `kg` version.
