# Directory Structure

kg uses a hierarchical directory-based configuration system. Configuration can be defined globally, locally, or both.

## Overview

```
~/.kiro/generators/          # Global configuration
├── manifests/               # Agent declarations & inheritance
│   └── kg.toml
└── agents/                  # Agent definitions
    ├── default.toml
    └── rust.toml

.kiro/generators/            # Local (project-specific) configuration
├── manifests/
│   └── kg.toml
└── agents/
    └── project-agent.toml
```

## Directories

### manifests/

Contains agent declarations and inheritance relationships.

**Purpose:** Define which agents exist and how they relate to each other.

**Files:** TOML files (typically `kg.toml`, but can be multiple files)

**Example:**
```toml
# manifests/kg.toml
[agents]
default = { inherits = [] }
rust = { inherits = ["default"] }
```

### agents/

Contains agent configuration definitions.

**Purpose:** Define the actual agent settings (tools, resources, permissions, MCP servers).

**Files:** One TOML file per agent, named `<agent-name>.toml`

**Example:**
```toml
# agents/default.toml
description = "Default agent"
allowedTools = ["read", "knowledge", "web_search"]
resources = ["file://README.md"]

[toolsSettings.shell]
allowedCommands = ["git status", "git fetch"]
autoAllowReadonly = true
```

## Configuration Precedence

Configuration is loaded and merged in this order (least to most precedence):

1. Global manifests: `~/.kiro/generators/manifests/*.toml`
2. Global agents: `~/.kiro/generators/agents/<agent-name>.toml`
3. Local manifests: `.kiro/generators/manifests/*.toml`
4. Local agents: `.kiro/generators/agents/<agent-name>.toml`

Local settings override global settings. Use `--local` to ignore global config, or `--global` to ignore local config.

## Initialization

Create the default structure with:

```bash
kg init
```

This creates:
```
~/.kiro/generators/
├── manifests/
│   └── kg.toml
└── agents/
    └── default.toml
```

## Multiple Manifest Files

You can split agent declarations across multiple files in `manifests/`:

```
manifests/
├── kg.toml           # Core agents
├── aws.toml          # AWS-specific agents
└── dev-tools.toml    # Development tool agents
```

All files are loaded and merged together. Agent names must be unique across all files.

## Inline vs External Definitions

You can define agent configuration inline in manifests or in separate files:

**Inline (in manifests/kg.toml):**
```toml
[agents.default]
inherits = []
allowedTools = ["read", "knowledge"]
```

**External (in agents/default.toml):**
```toml
allowedTools = ["read", "knowledge"]
```

Both approaches can be mixed. External definitions take precedence over inline definitions.

### When to Use Inline vs External

**Use inline configuration** when:
- Agent config is short (< 10 lines)
- You want everything in one file for simplicity
- Prototyping or experimenting

**Use external files** when:
- Config is complex or large
- Multiple people edit the same agent
- You want to reuse configurations across projects
- Following team conventions

**Best practice**: Start with inline for simplicity, extract to external files as configs grow.

## Validation

Use `kg validate` to see the final merged configuration:

```bash
kg validate
```

This shows exactly what each agent will generate, including all merged settings from global and local sources.
