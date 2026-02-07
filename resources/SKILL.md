---
name: kg-helper
description: Manage kiro-generator (kg) agent configurations. Discover existing agents, understand inheritance and templates, and modify TOML configs. Use when users ask about kg, need help with their agent setup, or want to create/modify agents.
license: MIT
compatibility: Requires access to user's kg configuration files in ~/.kiro/generators/ and .kiro/generators/
metadata:
  version: 0.1.0
  author: agents
---

## What is kg?

`kg` (kiro-generator) generates Kiro agent JSON files from composable TOML configurations. It solves the problem of managing agent configs by hand:

- **Composable**: Build agents from reusable templates
- **Hierarchical**: Global configs merge with local project configs
- **DRY**: Inherit and extend instead of duplicating

## Step 1: Discovery

Before reading or modifying any config files, run discovery to understand the current setup:

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
- `global-manifest` / `local-manifest` — Agent declared inline in a manifests/*.toml file
- `global-file` / `local-file` — Dedicated agent config file in agents/ directory

**Use the `sources` array to know exactly which files to read or edit.**

## Step 2: Validate and Generate

```bash
# Validate merged config (shows final state of all agents)
kg validate

# Generate .kiro/agents/*.json files from TOML configs
kg generate
```

## Core Concepts

### Two-Layer System

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

### Hierarchical Locations

```
~/.kiro/generators/     # Global (all projects)
.kiro/generators/       # Local (this project)
```

Local configs **merge with** global configs (not replace).

### Inheritance

Configuration flows from parent to child:

```toml
[agents]
base = { inherits = [] }
rust = { inherits = ["base"] }
```

**Merge rules:**
- **Arrays** (allowedTools, resources): Combined
- **Objects** (toolsSettings, mcpServers): Deep merged
- **Scalars** (description, timeout): Child replaces parent

### Templates

Templates don't generate JSON files -- they exist only for inheritance:

```toml
[agents.git-readonly]
template = true

[agents.git-readonly.toolsSettings.shell]
allowedCommands = ["git status", "git fetch"]
```

### Force Properties

Override restrictions in child agents:

```toml
[agents.git-pusher.toolsSettings.shell]
forceAllowedCommands = ["git commit .*", "git push .*"]
```

Children cannot deny forced commands/paths.

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

## Common Patterns

### Augment Global with Local

Global provides tooling, local adds project knowledge:

```toml
# Global: ~/.kiro/generators/agents/rust.toml
allowedTools = ["@rustdocs", "@cargo"]

# Local: .kiro/generators/agents/rust.toml
[knowledge.facet]
source = "file://./facet-docs"
```

### Shared Project Resources

Team shares resources, individuals customize agents:

```toml
# .kiro/generators/manifests/kg.toml (in git)
[agents.project-resources]
template = true
resources = ["file://docs/**/*.md"]

# .kiro/generators/manifests/my-agent.toml (gitignored)
[agents.rust]
inherits = ["project-resources"]
```

### Permission Levels

Build permission hierarchies:

```toml
[agents.git-readonly]
template = true
# ... read-only commands

[agents.git-write]
inherits = ["git-readonly"]
# ... add write commands
```
