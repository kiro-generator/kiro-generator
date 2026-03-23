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

If `kg tree details <agent>` shows `"source_type": "global-file"` or `"source_type": "global-manifest"` in sources and you're in a project directory, pass `--global` to `kg diff`, `kg generate`, etc.

## Step 1: Discovery

Before reading or modifying any config files, run discovery.

### Summary view (default)

`kg tree summary` shows a summary table of all agents and templates:

```bash
# Summary table of all agents + templates
kg tree summary

# Agents only, hide templates
kg tree summary --no-templates

# Add a lookup table with source file locations
kg tree summary --locations

# Machine-readable summary
kg tree summary -f json
```

The table shows agent name, description, and direct parents (`inherits`). Templates are grouped at the bottom.

`--locations` adds a third table that shows the global/local manifest and agent-file locations for every discovered agent and template.

JSON summary output always includes top-level `agents` and `templates` objects. Each entry includes `description`, `inherits`, and `locations`. With `--no-templates -f json`, the `templates` object is currently emitted as `{}` rather than being omitted.

### Single agent detail

`kg tree details <name>` outputs full JSON detail for one or more agents, including sources, modified fields, and the ordered transitive ancestor chain:

```bash
# Single agent detail (JSON)
kg tree details rust

# Multiple agents
kg tree details rust node

# Alias also accepted
kg tree detail rust
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
        "source_type": "global-file",
        "path": "/home/user/.kiro/generators/agents/rust.toml",
        "modified_fields": ["description", "prompt", "nativeTools.shell"]
      },
      {
        "source_type": "global-manifest",
        "path": "/home/user/.kiro/generators/manifests/base.toml",
        "modified_fields": ["inherits"]
      },
      {
        "source_type": "local-manifest",
        "path": ".kiro/generators/manifests/rust.toml",
        "modified_fields": ["description", "inherits"]
      }
    ],
    "inherits": ["default", "kg-resources"],
    "resolved_ancestors": ["kg-resources", "cli", "cli-compress-decompress", "git", "cli-systemd", "knowledge", "resources", "default"]
  }
}
```

**Source types:**
- `global-manifest` / `local-manifest` -- Agent declared inline in a manifests/*.toml file
- `global-file` / `local-file` -- Dedicated agent config file in agents/ directory

**`modified_fields`** lists which fields each source file contributes. Use this to go directly to the right file when a user asks "where does this shell command / MCP server / resource come from?" — no guessing required.

**`resolved_ancestors`** is the full transitive ancestor chain for the agent in merge order. This differs from `inherits` (direct parents only). Use it when debugging unexpected values to see which ancestors are applied first, then use `modified_fields` on `sources` to find the contributing file.

**Use the `sources` array to know exactly which files to read or edit.**

### Search across agents

`kg tree search <pattern>` finds which agents reference a given string, which fields matched, and where the config files live:

```bash
# Case-insensitive search (default)
kg tree search yarn

# Scope to a specific field prefix
kg tree search "git push" --field nativeTools.shell

# Case-sensitive
kg tree search MyServer --case-sensitive
```

Output is JSON with the search parameters echoed back and a `results` object keyed by agent name. Each result includes:
- **`match.fields`** — list of matched field paths (e.g. `nativeTools.shell`, `mcpServers.git`, `resources.docs`)
- **`match.locations`** — array of source file paths where matches were found (e.g. `["global-file://...", "local-manifest://..."]`)
- **`summary`** — agent description, direct parents, and source file locations

The `--field` filter accepts a field path or prefix. `--field nativeTools` matches `nativeTools.shell`, `nativeTools.aws`, etc. `--field nativeTools.shell` matches only that exact field.

Use this to answer "which agents reference X?" without manually grepping TOML files. Combine with `kg tree details <agent>` to trace the exact source file via `modified_fields`.

### Reverse dependency lookup

`kg tree dependents <name>` shows what depends on a given agent or template:

```bash
# What inherits from "default" (directly or transitively)?
kg tree dependents default

# Multiple lookups at once
kg tree dependents default aws

# Aliases also accepted
kg tree invert default
kg tree i default
```

`kg tree dependents <name>` returns a JSON object keyed by each requested name. Each value is the sorted set of concrete agents whose `resolved_ancestors` includes that name. Use this to assess blast radius before modifying a template or parent agent.

This command currently requires at least one name and does not emit inheritance paths or an orphaned-template report.

## Step 2: Edit Agent Configs

To modify an agent's configuration:

1. Run `kg tree details <agent-name>` to find which TOML files define it
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

# Only diff specific agents
kg diff --format agent rust node

# Human-friendly with color
kg diff

# When editing global agents from within a project:
kg diff --format agent --global
```

**Output shows:**
- File paths of affected agents
- Changed paths in dot notation (e.g., `shell.denied_commands.31: + "npm publish .*"`)
- Summary of changed vs unchanged agents
- If you pass one or more agent names, only those agents are diffed

**If diff shows parse errors:** Fix the TOML error at the reported file and line number, then re-run diff.

**If diff shows many changes across multiple agents:** This is normal. Common causes:
- Previous TOML edits that were never generated (accumulated drift)
- A template or parent agent was modified, and changes ripple through all inheritors via `resolved_ancestors`

Use `kg tree details <agent>` and check `resolved_ancestors` + `modified_fields` to trace which ancestor introduced a change.

**If diff shows `No changes (...)`:** Either there are no changes, or you may be checking the wrong scope. Verify with `kg tree details <agent>` that you're editing the right files and using the correct `--global` flag if needed.

**If diff says `agent not found in current scope`:** The requested agent name does not exist in the active diff scope. Common causes are a typo, or looking for a global agent while running inside a project without `--global`.

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

All found configs merge together. Use `kg tree details rust` to see which sources apply.

## Error States

- **No agents found**: `kg tree summary` shows an empty summary table.
- **Named agent not found**: `kg tree details nonexistent` returns `{}` (exit 0).
- **Invalid TOML**: `kg diff` reports parse errors with file path and line number
- **Diff agent not found in current scope**: `kg diff missing-agent` ends with `No changes (0 agents checked); agent not found in current scope: missing-agent`

**Recovery:** If diff reports a parse error, fix the error at the reported location and re-run diff. If `kg generate` produces unexpected results, run `kg tree details <agent>` to trace which source file contributes the unexpected value — check `modified_fields` on each source to pinpoint the exact file.

**If a `kg diff` path needs tracing** (e.g. you see a JSON field change and want to know which TOML field drives it), load `assets/mappings.json` to look up the TOML→Kiro JSON mapping and the jq path to inspect the generated value directly.

## Reference Documents

For detailed guidance on specific topics, load these as needed:

- **`references/migration.md`** -- Migrating from hand-written JSON agents to kg. Covers user interview framework (security posture, structure preference, scope), decision framework, and example layouts. Load when helping users set up kg for the first time or convert existing agents.
- **`references/templates.md`** -- Template categories (Permission, Capability, Context, Lifecycle), real-world examples, composition patterns, subagent templates. Load when helping users design or refactor their template hierarchy.
- **`references/schemas.md`** -- Load when writing TOML fields beyond shell rules and MCP servers (hooks, knowledge, subagents, toolSettings). Contains full TOML→JSON field mappings, jq recipes for inspecting generated output, and schema discovery commands.

## Keeping This Skill Updated

This skill is shipped with the `kg` package at `/usr/share/doc/kiro-generator/kg-helper/SKILL.md`. To update, install the latest `kg` version.
