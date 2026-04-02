# Directory Structure

`kg` keeps source configuration and generated output separate.

That is the main idea to keep in your head:

- you edit TOML in `generators/`
- `kg` generates JSON into `agents/`
- global and local config can both exist at the same time

## Overview

```text
~/.kiro/generators/          # Global configuration
├── manifests/               # Agent declarations & inheritance
│   └── kg.toml
└── agents/                  # Agent definitions
    ├── default.toml
    └── rust.toml

.kiro/generators/            # Local (project-specific) configuration
└── agents/
    └── rust.toml            # override global rust.toml

~/.kiro/agents/              # Generated global Kiro JSON
└── *.json

.kiro/agents/                # Generated local Kiro JSON
└── *.json
```

## Source Directories

### manifests/

`manifests/` declares which agents exist and how they relate to each other.

Any `.toml` file in this directory is loaded. `kg.toml` is common, but not special.

**Example:**
```toml
[agents]
default = { inherits = [] }
rust = { inherits = ["default"] }
```

### agents/

`agents/` contains the actual configuration for an agent.

**Example:**
```toml
# ~/.kiro/generators/agents/rust.toml
description = "Rust development agent"
tools = ["*"]
allowedTools = ["read", "knowledge", "web_search", "@rustdocs"]

[mcpServers.rustdocs]
command = "rust-docs-mcp"
timeout = 1000
```

A project can then extend that same agent locally:

```toml
# .kiro/generators/agents/rust.toml
description = "Rust development agent for this project"

[resources.project]
locations = ["README.md", "Cargo.toml"]
```

## Configuration Precedence

Configuration is loaded and merged in this order:

1. Global manifests: `~/.kiro/generators/manifests/*.toml`
2. Global agents: `~/.kiro/generators/agents/<agent-name>.toml`
3. Local manifests: `.kiro/generators/manifests/*.toml`
4. Local agents: `.kiro/generators/agents/<agent-name>.toml`

Local config merges on top of global config.

That means a global `rust.toml` can define your usual tools and MCP servers, while a local `rust.toml` adds project-specific resources or small overrides.

Use:

- `--global` to ignore local config
- `--local` to ignore global config

## Generated Output

The TOML files under `generators/` are the source of truth.

Generated JSON is written to:

- `~/.kiro/agents/` for global agents
- `.kiro/agents/` for local agents

So for the `rust` example above, `kg generate` would write either:

- `~/.kiro/agents/rust.json`
- `.kiro/agents/rust.json`

depending on which scope is active.

## Initialization

There are two different init paths:

```bash
kg init
```

This installs the `kg-helper` agent to:

```text
~/.kiro/agents/kg-helper.json
```

If you want starter TOML files instead, use:

```bash
kg init --skeleton
```

That creates:

```text
~/.kiro/generators/
├── manifests/
│   └── kg.toml
└── agents/
    ├── default.toml
    └── git.toml
```

## Multiple Manifest Files

You can split agent declarations across multiple files in `manifests/`:

```text
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
[agents.rust]
inherits = ["default"]
description = "Rust development agent"
```

**External (in agents/rust.toml):**
```toml
description = "Rust development agent"
allowedTools = ["read", "knowledge", "web_search", "@rustdocs"]
```

Both approaches can be mixed. External definitions take precedence over inline definitions.

### When to Use Inline vs External

Use inline configuration when:
- Agent config is short (< 10 lines)
- You want everything in one file for simplicity
- Prototyping or experimenting

Use external files when:
- Config is complex or large
- Multiple people edit the same agent
- You want to reuse configurations across projects
- Following team conventions

Start simple. Extract config into separate files when it stops being simple.

## Validation

Use these commands to inspect what `kg` sees:

```bash
kg validate
kg tree summary
kg tree details rust
```

If you are inside a project and want to inspect only the global config, add `--global`.
