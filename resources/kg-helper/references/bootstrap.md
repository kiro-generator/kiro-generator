# Bootstrap: Migrating from JSON Agents

When a user runs `kg bootstrap`, kg scans their existing `~/.kiro/agents/*.json` files (global agents only) and produces `~/.kiro/skills/kg-helper/references/analysis.json`. This file is your primary input for recommending a kg configuration.

**Important:** Before generating any TOML, tell the user to back up their existing agent configs:
```bash
mv ~/.kiro/agents ~/.kiro/agents.bak
```
`kg generate` will overwrite `~/.kiro/agents/*.json`. A clean break avoids mixing hand-written JSON with generated output.

## analysis.json Schema

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

## Interview the User

Before generating any TOML, ask these three questions in order:

### 1. Security posture -- "How much do you trust your tools?"

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

### 2. Structure preference -- "How do you want to organize your agents?"

- **Flat**: Independent agents, no inheritance. Best for 1-3 agents with little overlap.
- **Layered**: One `default` base agent, others inherit from it. Best for 3-6 agents with shared config.
- **Composable**: Templates for git, MCP servers, permissions -- agents composed from building blocks. Best for 6+ agents or teams.

Use the `overlap` section to guide the recommendation:
- If overlap shows 3+ agents sharing the same tools/commands, suggest **Layered** or **Composable**.
- If overlap is minimal, suggest **Flat**.
- If there are clear permission tiers (readonly vs write vs admin), suggest **Composable**.

### 3. Scope -- "Global, local, or both?"

- **Global only**: Personal tooling across all projects (`~/.kiro/generators/`)
- **Local only**: This project only (`.kiro/generators/`)
- **Both**: Global base + local project augmentation -- the kg sweet spot

If agents come from `~/.kiro/agents/`, suggest **Global** or **Both**.
If agents come from `.kiro/agents/`, suggest **Local** or **Both**.

## Decision Framework

After the interview, map answers + analysis to concrete TOML:

1. **Identify the base set**: Look at `overlap.allowedTools` and `overlap.shellCommands`. Fields appearing in >50% of agents belong in a `default` agent or template.

2. **Identify template candidates**: Look for clusters in `overlap.mcpServers`. If 3+ agents share the same MCP server, that's a template (e.g., `aws-base`, `frontend-tools`).

3. **Identify permission tiers**: Look at `overlap.shellCommands`. If some agents have read-only git and others have full git, that's a `git-readonly` / `git-write` template hierarchy.

4. **Map remaining config**: Whatever isn't shared goes into individual agent files.

5. **Don't over-engineer**: If there are only 1-2 agents, skip templates entirely. A flat structure with no inheritance is fine. Don't build a 3-layer hierarchy for two agents.

## Example Layouts

### Flat (1-3 agents, minimal overlap)

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

### Layered (3-6 agents, shared base)

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

### Composable (6+ agents, clear groupings)

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
      git.toml
      cli.toml
      resources.toml
```

Place template agent files in `agents/templates/` to keep them visually separate from real agents.

```toml
# manifests/kg.toml
[agents]
git = { template = true }
cli = { template = true }
resources = { template = true }
k8s = { template = true }

default = { inherits = ["git", "resources", "cli"] }
rust = { inherits = ["default"] }
frontend = { inherits = ["default"] }
devops = { inherits = ["default", "k8s"] }
```

## Generating the TOML Files

After the user confirms the layout:

1. Create `~/.kiro/generators/manifests/kg.toml` with the agent declarations and inheritance
2. Create individual `~/.kiro/generators/agents/<name>.toml` files with agent-specific config
3. For each agent, only include config that ISN'T inherited from parents
4. Run `kg validate` to verify the merged output matches the original JSON agents
5. Run `kg generate` to produce the new JSON files
6. Suggest the user diff the generated JSON against their original `.kiro/agents/*.json` to verify correctness

## Edge Cases

- **No existing agents**: User is starting fresh. Skip analysis, ask what kind of project they work on, and scaffold a minimal setup based on their answers.
- **Single agent**: Don't suggest inheritance. Create a flat config with one manifest entry and one agent file.
- **Massive duplication** (20+ agents with identical allowedTools): Strongly recommend Composable layout. Point out the duplication explicitly.
- **Mixed global and local agents**: Suggest the Both scope. Global for shared tooling, local for project-specific knowledge/resources.
