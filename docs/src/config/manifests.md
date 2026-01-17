# Manifests

Manifests declare agents and their relationships. They live in the `manifests/` directory.

## Purpose

Manifests answer: "What agents exist and how do they relate to each other?"

They don't contain the actual configuration - that's in `agents/`. Think of manifests as the index or table of contents.

## Location

- **Global:** `~/.kiro/generators/manifests/*.toml`
- **Local:** `.kiro/generators/manifests/*.toml`

## Basic Structure

```toml
# manifests/kg.toml
[agents]
default = { inherits = [] }
rust = { inherits = ["default"] }
python = { inherits = ["default"] }
```

## Agent Declaration

Each entry in `[agents]` declares an agent:

```toml
[agents]
agent-name = { inherits = ["parent1", "parent2"] }
```

The key (`agent-name`) is the agent's name. This tells kg to look for:
- `agents/agent-name.toml` for configuration
- Or inline configuration in the manifest itself

## Inheritance

Use `inherits` to build agent hierarchies:

```toml
[agents]
base = { inherits = [] }
dev = { inherits = ["base"] }
rust-dev = { inherits = ["dev"] }
```

Configuration flows from parent to child. See [Inheritance](./inheritance.md) for details.

## Templates

Mark agents as templates (non-generating) with `template = true`:

```toml
[agents]
git-base = { template = true }
rust = { inherits = ["git-base"] }
```

Templates provide reusable configuration but don't generate JSON files. See [Templates](./templates.md) for details.

## Inline Configuration

You can define agent configuration directly in manifests:

```toml
[agents.default]
inherits = []
allowedTools = ["read", "knowledge"]
resources = ["file://README.md"]

[agents.default.toolsSettings.shell]
allowedCommands = ["git status", "git fetch"]
```

This is useful for simple agents or when you want everything in one file.

## Multiple Manifest Files

Split declarations across multiple files for organization:

```
manifests/
├── kg.toml           # Core agents
├── aws.toml          # AWS-specific agents
└── dev-tools.toml    # Development tool agents
```

All files are loaded and merged. Agent names must be unique across all files.

**Example:**

```toml
# manifests/kg.toml
[agents]
default = { inherits = [] }

# manifests/aws.toml
[agents]
aws-base = { template = true }
secops = { inherits = ["default", "aws-base"] }
```

## Validation

Agent names must be unique across all manifest files. kg will error if duplicates are found:

```
Error: Duplicate agent 'rust' found in:
  - ~/.kiro/generators/manifests/kg.toml
  - ~/.kiro/generators/manifests/dev.toml
```

## Best Practices

- Keep manifests focused on relationships, not configuration
- Use descriptive agent names (`rust-dev`, not `r1`)
- Group related agents in the same manifest file
- Use templates for reusable components
- Document complex inheritance chains with comments

## Example: Complete Manifest

```toml
# manifests/kg.toml

# Base agents (templates)
[agents.git-base]
template = true

[agents.mcp-base]
template = true

# Real agents
[agents.default]
inherits = ["git-base"]

[agents.rust]
inherits = ["default", "mcp-base"]

[agents.python]
inherits = ["default", "mcp-base"]

[agents.secops]
inherits = ["default"]
```
