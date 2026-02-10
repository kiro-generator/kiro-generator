---
name: kg-helper
description: Help set up and manage kg (kiro-generator) TOML agent configurations. Use when users ask about kg, migrating JSON agents to kg, or bootstrapping a new kg setup.
license: MIT
compatibility: Requires access to user's kg configuration files in ~/.kiro/generators/ and .kiro/generators/
metadata:
  version: 0.2.0
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

## Step 3: Editing Agent Configs

To modify an agent's configuration:

1. Run `kg tree <agent-name>` to find which TOML files define it
2. Read the `sources` array -- it tells you exactly which files to edit
3. Edit the TOML file(s) directly
4. Run `kg validate` to verify the change
5. Run `kg generate` to apply

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

## Security Patterns

### Global Deny List with Per-Agent Overrides

Define safe defaults globally, override for specific agents:

```toml
# Global: ~/.kiro/generators/agents/default.toml
[toolsSettings.shell]
deniedCommands = ["rm -rf .*", "git push .*", "git commit .*", "cargo publish .*"]
autoAllowReadonly = true

# Local: .kiro/generators/agents/dependabot.toml
inherits = ["default"]
[toolsSettings.shell]
forceAllowedCommands = ["git commit .*", "git push .*"]
```

**Result**: All agents inherit the deny list, but `dependabot` can commit and push. The `force` prefix prevents child agents from accidentally denying these commands.

### Permission Template Hierarchies

Build permission tiers that compose safely:

```toml
# manifests/kg.toml
[agents.git-readonly]
template = true

[agents.git-readonly.toolsSettings.shell]
allowedCommands = ["git status", "git fetch", "git diff .*", "git log .*"]

[agents.git-write]
template = true
inherits = ["git-readonly"]

[agents.git-write.toolsSettings.shell]
forceAllowedCommands = ["git add .*", "git commit .*", "git push .*"]

# Real agents
[agents.reviewer]
inherits = ["git-readonly"]

[agents.dependabot]
inherits = ["git-write"]
```

**Result**: `reviewer` gets read-only git, `dependabot` gets read + write. The hierarchy is explicit and auditable.

### Subagent Default Template

Create a base template for all subagents with restricted permissions:

```toml
# manifests/subagents.toml
[agents.subagent-default]
template = true
model = "claude-haiku-4.5"
inherits = ["git-readonly", "cli-safe"]
tools = ["*"]

[agents.subagent-default.toolsSettings.shell]
deniedCommands = ["rm -rf .*", "git push .*", "cargo publish .*", "npm publish .*"]

# Specific subagents
[agents.gh-workflow]
inherits = ["subagent-default"]
description = "GitHub Workflow monitor"
resources = ["file://~/.config/agents/resources/gh-workflow.md"]

[agents.jina]
inherits = ["subagent-default"]
description = "HTML to markdown converter"
```

**Result**: All subagents get the same security baseline. Add specific tools/resources per subagent without duplicating the deny list.

### Auditing Permissions

Use `kg tree` and `kg validate` to audit the final permission set:

```bash
# See where permissions come from
kg tree dependabot

# Output shows inheritance chain and sources:
{
  "dependabot": {
    "inherits": ["default", "git-write"],
    "sources": [
      {"type": "global-manifest", "path": "~/.kiro/generators/manifests/kg.toml"},
      {"type": "local-file", "path": ".kiro/generators/agents/dependabot.toml"}
    ]
  }
}

# See final merged permissions
kg validate --format json | jq '.agents.dependabot.toolsSettings.shell'

# Output shows final allowedCommands and deniedCommands after merge
```

**Result**: Full visibility into where each permission comes from and what the final state is.

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

## Bootstrap: Migrating from JSON Agents

When a user runs `kg bootstrap`, kg scans their existing `~/.kiro/agents/*.json` files (global agents only) and produces `~/.kiro/skills/kg-helper/references/analysis.json`. This file is your primary input for recommending a kg configuration.

**Important:** Before generating any TOML, tell the user to back up their existing agent configs:
```bash
mv ~/.kiro/agents ~/.kiro/agents.bak
```
`kg generate` will overwrite `~/.kiro/agents/*.json`. A clean break avoids mixing hand-written JSON with generated output.

### analysis.json Schema

```json
{
  "agents": {
    "<agent-name>": {
      "source": "global|local",
      "path": "/absolute/path/to/agent.json",
      "name": "agent-name",
      "description": "Human-readable description of the agent",
      "tools": ["*"],
      "allowedTools": ["read", "grep", "@context7/*"],
      "mcpServers": {
        "context7": { "command": "context7-mcp", "args": [], "timeout": 120000 }
      },
      "toolsSettings": {
        "shell": {
          "allowedCommands": ["git status", "cargo .*"],
          "deniedCommands": ["rm -rf .*"],
          "autoAllowReadonly": true
        }
      },
      "resources": ["file://README.md", "file://AGENTS.md"],
      "knowledge": {},
      "nativeTools": {},
      "hooks": {}
    }
  },
  "overlap": {
    "allowedTools": {
      "read": ["agent-a", "agent-b", "agent-c"],
      "grep": ["agent-a", "agent-b"]
    },
    "mcpServers": {
      "context7": ["agent-a", "agent-c"]
    },
    "shellCommands": {
      "git status": ["agent-a", "agent-b", "agent-c"]
    },
    "resources": {
      "file://README.md": ["agent-a", "agent-b"]
    }
  },
  "summary": {
    "total_agents": 5,
    "total_mcp_servers": 3,
    "agents_with_shell_settings": 4,
    "agents_with_knowledge": 1
  }
}
```

### Interview the User

Before generating any TOML, ask these three questions in order:

**1. Security posture** -- "How much do you trust your tools?"

- **Permissive**: `allowedTools = ["*"]` -- no prompts, full speed. Good for personal projects.
- **Moderate**: `tools = ["*"]` but curated `allowedTools` list -- tools available but gated on dangerous ones. Most common in the wild.
- **Strict**: Explicit `tools` list, locked-down shell, deny-by-default. For teams or sensitive environments.

If analysis.json shows most agents already use curated allowedTools (not `"*"`), default to **Moderate**.
If most agents use `allowedTools = ["*"]`, default to **Permissive**.

After the user picks a posture, recommend a default deny list for shell commands.
Every posture gets a base deny list. Stricter postures add more:

**Base (all postures):**
```toml
deniedCommands = ["rm -rf .*", "chmod -R 777 .*"]
```

**Moderate adds:**
```toml
deniedCommands = ["rm -rf .*", "chmod -R 777 .*", "git push .*", "git commit .*", "cargo publish .*", "npm publish .*", "docker push .*"]
```

**Strict adds:**
```toml
deniedCommands = ["rm -rf .*", "chmod -R 777 .*", "git push .*", "git commit .*", "cargo publish .*", "npm publish .*", "docker push .*", "kubectl delete .*", "terraform destroy.*", "aws .* delete.*"]
```

Present the deny list to the user and ask if they want to adjust it before proceeding. Some users have project-specific dangerous commands (like `terraform apply`, `kubectl delete`, or deployment scripts) that should be added to the deny list.

**2. Structure preference** -- "How do you want to organize your agents?"

- **Flat**: Independent agents, no inheritance. Best for 1-3 agents with little overlap.
- **Layered**: One `default` base agent, others inherit from it. Best for 3-6 agents with shared config.
- **Composable**: Templates for git, MCP servers, permissions -- agents composed from building blocks. Best for 6+ agents or teams.

Use the `overlap` section to guide the recommendation:
- If overlap shows 3+ agents sharing the same tools/commands, suggest **Layered** or **Composable**.
- If overlap is minimal, suggest **Flat**.
- If there are clear permission tiers (readonly vs write vs admin), suggest **Composable**.

**3. Scope** -- "Global, local, or both?"

- **Global only**: Personal tooling across all projects (`~/.kiro/generators/`)
- **Local only**: This project only (`.kiro/generators/`)
- **Both**: Global base + local project augmentation -- the kg sweet spot

If agents come from `~/.kiro/agents/`, suggest **Global** or **Both**.
If agents come from `.kiro/agents/`, suggest **Local** or **Both**.

### Decision Framework

After the interview, map answers + analysis to concrete TOML:

1. **Identify the base set**: Look at `overlap.allowedTools` and `overlap.shellCommands`. Fields appearing in >50% of agents belong in a `default` agent or template.

2. **Identify template candidates**: Look for clusters in `overlap.mcpServers`. If 3+ agents share the same MCP server, that's a template (e.g., `aws-base`, `frontend-tools`).

3. **Identify permission tiers**: Look at `overlap.shellCommands`. If some agents have read-only git and others have full git, that's a `git-readonly` / `git-write` template hierarchy.

4. **Map remaining config**: Whatever isn't shared goes into individual agent files.

5. **Don't over-engineer**: If there are only 1-2 agents, skip templates entirely. A flat structure with no inheritance is fine. Don't build a 3-layer hierarchy for two agents.

### Example Layouts

#### Flat (1-3 agents, minimal overlap)

```
~/.kiro/generators/
  manifests/kg.toml
  agents/
    rust.toml
    frontend.toml
```

```toml
# manifests/kg.toml
[agents]
rust = { inherits = [] }
frontend = { inherits = [] }
```

#### Layered (3-6 agents, shared base)

```
~/.kiro/generators/
  manifests/kg.toml
  agents/
    default.toml
    rust.toml
    frontend.toml
    devops.toml
```

```toml
# manifests/kg.toml
[agents]
default = { inherits = [] }
rust = { inherits = ["default"] }
frontend = { inherits = ["default"] }
devops = { inherits = ["default"] }
```

```toml
# agents/default.toml
description = "Base agent"
allowedTools = ["read", "grep", "glob", "web_search"]
resources = ["file://README.md"]

[toolsSettings.shell]
allowedCommands = ["git status", "git fetch", "git diff .*"]
autoAllowReadonly = true
```

#### Composable (6+ agents, clear groupings)

```
~/.kiro/generators/
  manifests/
    kg.toml
    subagents.toml
  agents/
    default.toml
    rust.toml
    frontend.toml
    templates/
      git-readonly.toml
      git-write.toml
      aws-base.toml
```

Place template agent files in `agents/templates/` to keep them visually separate from real agents.

```toml
# manifests/kg.toml
[agents.git-readonly]
template = true

[agents.git-write]
template = true
inherits = ["git-readonly"]

[agents.aws-base]
template = true

[agents.default]
inherits = ["git-readonly"]

[agents.rust]
inherits = ["default"]

[agents.frontend]
inherits = ["default"]

[agents.devops]
inherits = ["default", "git-write", "aws-base"]
```

### Generating the TOML Files

After the user confirms the layout:

1. Create `~/.kiro/generators/manifests/kg.toml` with the agent declarations and inheritance
2. Create individual `~/.kiro/generators/agents/<name>.toml` files with agent-specific config
3. For each agent, only include config that ISN'T inherited from parents
4. Run `kg validate` to verify the merged output matches the original JSON agents
5. Run `kg generate` to produce the new JSON files
6. Suggest the user diff the generated JSON against their original `.kiro/agents/*.json` to verify correctness

### Edge Cases

- **No existing agents**: User is starting fresh. Skip analysis, ask what kind of project they work on, and scaffold a minimal setup based on their answers.
- **Single agent**: Don't suggest inheritance. Create a flat config with one manifest entry and one agent file.
- **Massive duplication** (20+ agents with identical allowedTools): Strongly recommend Composable layout. Point out the duplication explicitly.
- **Mixed global and local agents**: Suggest the Both scope. Global for shared tooling, local for project-specific knowledge/resources.

## Keeping This Skill Updated

This skill document may be updated as kg evolves. To get the latest version:

```bash
curl -o ~/.kiro/skills/kg-helper/SKILL.md https://kiro-generator.io/SKILL.md
```

Check for updates periodically or when kg releases a new version. The skill document version is tracked in the metadata header.
