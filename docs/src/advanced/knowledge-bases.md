# Knowledge Bases

Project-specific knowledge bases are a powerful pattern for augmenting global agents with local context.

## The Problem

In standard kiro-cli, local agent files completely override global ones. If you have a global `~/.kiro/agents/rust.json` with cargo tooling, and you create a local `.kiro/agents/rust.json` with project-specific knowledge, you lose all the global configuration.

You'd have to duplicate the cargo commands, shell permissions, and resources in every project.

## The kg Solution

kg merges global and local configurations during generation. This lets you:
- Keep general tooling in global config
- Add project-specific knowledge locally
- Get both in the final generated agent

## Real-World Example

**Global Rust agent** (`~/.kiro/generators/agents/lang/rust.toml`):
```toml
description = "General Rust agent"
resources = [
  "file://~/.config/agent/resources/rust.md",
  "file://docs/src/**/*.md"
]

[nativeTools.shell]
allow = ["cargo .+"]
deny = ["cargo publish .*"]
```

**Local project config** (`.kiro/generators/agents/rust.toml`):
```toml
[knowledge.facet]
source = "file://./facet-docs"
description = "information about the Rust crates facet-json, facet-toml, facet-diff and other facet libraries"
autoUpdate = true
indexType = "best"
```

**Generated result** (`.kiro/agents/rust.json`):
```json
{
  "description": "General Rust agent",
  "resources": [
    "file://~/.config/agent/resources/rust.md",
    "file://docs/src/**/*.md"
  ],
  "nativeTools": {
    "shell": {
      "allow": ["cargo .+"],
      "deny": ["cargo publish .*"]
    }
  },
  "knowledge": {
    "facet": {
      "source": "file://./facet-docs",
      "description": "information about the Rust crates facet-json, facet-toml, facet-diff and other facet libraries",
      "autoUpdate": true,
      "indexType": "best"
    }
  }
}
```

## Why This Works

- **Global config** provides cargo tooling and general Rust resources
- **Local config** adds project-specific facet documentation knowledge base
- **kg merges** both during generation, so the final agent has everything
- **No duplication** - cargo commands aren't repeated in every project

## Pattern: Shared Project Resources

For team projects, create a reusable template that defines project-specific resources:

**`.kiro/generators/manifests/kg.toml`** (checked into git):
```toml
[agents.kg-resources]
template = true
resources = [
  "file://docs/src/**/*.md",
  "file://AGENTS.md",
  "file://README.md",
]

[agents.kg-resources.knowledge.facet]
source = "file://./facet-docs"
description = "Information about the Rust crates facet-json, facet-toml, facet-diff and other facet libraries"
autoUpdate = true
indexType = "best"

[agents.kg-resources.knowledge.kiro]
source = "file://./docs/kiro"
description = "Kiro CLI configuration reference for JSON agents"
autoUpdate = true
indexType = "best"
```

**`.kiro/generators/manifests/rust.toml`** (gitignored, personal):
```toml
[agents.rust]
inherits = ["kg-resources"]
```

**`.kiro/generators/manifests/super-rust.toml`** (teammate's personal file):
```toml
[agents.super-rust]
inherits = ["kg-resources"]
```

### Why This Works

- `kg-resources` is a template (doesn't generate an agent file)
- Each developer creates their own manifest file with their preferred agent name
- Everyone inherits from the shared `kg-resources` template
- No conflicts: different agent names, different files
- Project context is shared, personal tooling is individual

### Gitignore Setup

```gitignore
# Ignore personal agent manifests
.kiro/generators/manifests/*.toml
# Keep shared project resources
!.kiro/generators/manifests/kg.toml

# Ignore generated agent files
.kiro/agents/
```

### Result

- You run `kg generate` → `.kiro/agents/rust.json` (global rust tooling + project resources)
- Teammate runs `kg generate` → `.kiro/agents/super-rust.json` (their global config + project resources)
- Both agents have the same project knowledge bases and resources
- No duplication, no conflicts, pure composition

## Pattern: Augment, Don't Replace

This pattern works for any project-specific context:
- Documentation knowledge bases
- Project-specific MCP servers
- Additional resources or tools
- Extended permissions for project needs

The key insight: local config augments global config rather than replacing it entirely.

## Comparison to Native kiro-cli

**Without kg:**
```
~/.kiro/agents/rust.json          ← Has cargo tooling
.kiro/agents/rust.json            ← Overrides completely, loses cargo tooling
```

**With kg:**
```
~/.kiro/generators/agents/lang/rust.toml    ← Global cargo tooling
.kiro/generators/agents/rust.toml           ← Local knowledge base
                    ↓
            kg generate
                    ↓
.kiro/agents/rust.json                      ← Merged: both cargo + knowledge
```

This is the composability that makes kg valuable for real projects.
