# Inheritance

Agents can inherit configuration from other agents using the `inherits` field.

## Basic Inheritance

```toml
[agents]
default = { inherits = [] }
rust = { inherits = ["default"] }
```

The `rust` agent inherits all configuration from `default` and can override or extend it.

## Multiple Parents

Agents can inherit from multiple parents:

```toml
[agents]
git-base = { template = true }
rust-base = { template = true }
rust-dev = { inherits = ["git-base", "rust-base"] }
```

Configuration is merged left-to-right. Later parents override earlier ones.

## Merge Behavior

### Arrays (allowedTools, resources, etc.)

Arrays are **merged** (combined):

**Parent:**
```toml
[agents.default]
allowedTools = ["read", "knowledge"]
```

**Child:**
```toml
[agents.rust]
inherits = ["default"]
allowedTools = ["@rustdocs", "@cargo"]
```

**Result:**
```toml
allowedTools = ["read", "knowledge", "@rustdocs", "@cargo"]
```

### Objects (toolsSettings, mcpServers, etc.)

Objects are **deep merged**:

**Parent:**
```toml
[agents.default.toolsSettings.shell]
allowedCommands = ["git status", "git fetch"]
autoAllowReadonly = true
```

**Child:**
```toml
[agents.rust.toolsSettings.shell]
allowedCommands = ["cargo .*"]
```

**Result:**
```toml
[toolsSettings.shell]
allowedCommands = ["git status", "git fetch", "cargo .*"]
autoAllowReadonly = true
```

### Scalars (description, timeout, etc.)

Scalars are **replaced**:

**Parent:**
```toml
description = "Default agent"
```

**Child:**
```toml
description = "Rust development agent"
```

**Result:**
```toml
description = "Rust development agent"
```

## Inheritance Chain

You can create deep inheritance hierarchies:

```toml
[agents]
base = { inherits = [] }
dev = { inherits = ["base"] }
rust-dev = { inherits = ["dev"] }
```

Configuration is merged from root to leaf: `base` → `dev` → `rust-dev`

## Force Properties

Force properties override permission restrictions in child agents. They ensure specific commands or paths are always allowed, even if a child tries to deny them.

### forceAllowedCommands

Force specific shell commands to be allowed:

```toml
[agents.git-pusher]
template = true
[agents.git-pusher.toolsSettings.shell]
forceAllowedCommands = ["git commit .*", "git push .*"]
```

Child agents cannot deny these commands:

```toml
[agents.dependabot]
inherits = ["git-pusher"]
[agents.dependabot.toolsSettings.shell]
deniedCommands = ["git push .*"]  # Ignored - git push is forced allowed
```

The forced commands are added to `allowedCommands` and removed from `deniedCommands`.

### forceAllowedPaths

Force specific paths to be readable or writable:

```toml
[agents.cargo-editor]
template = true
[agents.cargo-editor.toolsSettings.read]
forceAllowedPaths = [".*Cargo.toml.*"]
[agents.cargo-editor.toolsSettings.write]
forceAllowedPaths = [".*Cargo.toml.*"]
```

Child agents must allow access to these paths:

```toml
[agents.dependabot]
inherits = ["cargo-editor"]
[agents.dependabot.toolsSettings.write]
deniedPaths = [".*Cargo.toml.*"]  # Ignored - Cargo.toml is forced allowed
```

### Use Cases

Force properties are useful for:
- Ensuring critical permissions in specialized agents (like dependabot needing git push)
- Creating permission templates that can't be accidentally restricted
- Building agent hierarchies with guaranteed capabilities

## Validation

Use `kg validate` to see the final merged configuration:

```bash
kg validate
```

This shows exactly what each agent will generate, including inherited settings and forced permissions.
