# Agent Definitions

Agent definitions contain the actual configuration for your agents. They live in the `agents/` directory.

## Purpose

Agent definitions answer: "What does this agent do? What tools can it use? What resources does it have?"

They contain all the configuration that gets compiled into the final Kiro agent JSON file.

## Location

- **Global:** `~/.kiro/generators/agents/<agent-name>.toml`
- **Local:** `.kiro/generators/agents/<agent-name>.toml`

## Basic Structure

```toml
# agents/default.toml
description = "Default agent"
tools = ["*"]
allowedTools = ["read", "knowledge", "web_search"]
resources = ["file://README.md", "file://AGENTS.md"]

[toolsSettings.shell]
allowedCommands = ["git status", "git fetch", "git diff .*"]
autoAllowReadonly = true
```

## Configuration Fields

### Core Fields

**description** - Human-readable description of the agent

**tools** - Array of tools available to the agent. Use `["*"]` for all tools.

**allowedTools** - Tools that don't require user permission prompts. Supports wildcards and MCP patterns.

**resources** - Files or URLs to include as context. Supports `file://` and `https://` schemes.

### MCP Servers

Define MCP servers the agent can use:

```toml
[mcpServers.rustdocs]
command = "rust-docs-mcp"
timeout = 1000

[mcpServers.cargo]
command = "cargo-mcp"
timeout = 1200
```

### Tool Settings

Configure permissions for specific tools:

```toml
[toolsSettings.shell]
allowedCommands = ["git status", "git fetch"]
deniedCommands = ["git push .*", "rm -rf .*"]
autoAllowReadonly = true

[toolsSettings.read]
allowedPaths = [".*\\.rs$", ".*Cargo\\.toml$"]
deniedPaths = [".*/target/.*"]

[toolsSettings.write]
allowedPaths = [".*\\.rs$"]
deniedPaths = [".*Cargo\\.lock$"]
```

### Native Tools

Configure native shell access:

```toml
[nativeTools.shell]
allow = ["git .*", "cargo .*"]
```

### Hooks

Define lifecycle hooks:

```toml
[hooks.pre_chat]
type = "shell"
command = "git fetch"

[hooks.post_chat]
type = "shell"
command = "git status"
```

## Inline vs External

You can define configuration inline in manifests or in separate files:

**External (recommended):**
```toml
# manifests/kg.toml
[agents]
default = { inherits = [] }

# agents/default.toml
allowedTools = ["read", "knowledge"]
```

**Inline:**
```toml
# manifests/kg.toml
[agents.default]
inherits = []
allowedTools = ["read", "knowledge"]
```

External definitions take precedence over inline definitions.

## Inheritance

Agent definitions are merged via inheritance. See [Inheritance](./inheritance.md) for details.

## Validation

Use `kg validate` to see the final merged configuration:

```bash
kg validate
```

This shows exactly what each agent will generate, including all inherited settings.

## Example: Complete Agent

```toml
# agents/rust.toml
description = "Rust development agent"
tools = ["*"]
allowedTools = ["@rustdocs", "@cargo", "read", "knowledge", "web_search"]
resources = [
    "file://README.md",
    "file://RUST.md",
    "file://Cargo.toml"
]

[mcpServers.rustdocs]
command = "rust-docs-mcp"
timeout = 1000

[mcpServers.cargo]
command = "cargo-mcp"
timeout = 1200

[toolsSettings.shell]
allowedCommands = ["cargo .*", "git status", "git fetch"]
deniedCommands = ["cargo publish .*"]
autoAllowReadonly = true

[toolsSettings.read]
allowedPaths = [".*\\.rs$", ".*Cargo\\.toml$"]

[toolsSettings.write]
allowedPaths = [".*\\.rs$"]
deniedPaths = [".*Cargo\\.lock$"]
```
