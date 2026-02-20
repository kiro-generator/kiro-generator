# Schemas

kg provides JSON schemas for validating configuration and inspecting available fields.

## Available Schemas

| Schema | Describes | Filename |
|--------|-----------|----------|
| `kiro-agent` | Final Kiro agent JSON format (what kg generates) | `kiro-agent.json` |
| `kg-manifest` | kg manifest TOML files (`manifests/*.toml`) | `manifest.json` |
| `kg-agent` | kg agent TOML files (`agents/*.toml`) | `agent.json` |

## Getting Schemas

### kg schema (preferred)

Generate schemas from the running binary -- always matches your installed version:

```bash
kg schema kiro-agent   # Kiro agent JSON format
kg schema manifest     # kg manifest TOML format
kg schema agent        # kg agent TOML format
```

### System packages

If kg was installed via a package manager, schemas are on disk:

| Platform | Path |
|----------|------|
| Debian/Ubuntu | `/usr/share/doc/kiro-generator/schemas/` |
| Arch Linux | `/usr/share/doc/kiro-generator/schemas/` |
| Homebrew (macOS) | `$(brew --prefix)/share/kiro-generator/schemas/` |

### Fallback URLs

If neither `kg` nor system packages are available:

```
https://kiro-generator.io/kiro-agent.json
https://kiro-generator.io/manifest.json
https://kiro-generator.io/agent.json
```

## TOML to JSON Field Mappings

kg's TOML format uses different field names than Kiro's JSON format. Key mappings:

| TOML Field (kg) | JSON Field (Kiro) | Notes |
|-----------------|-------------------|-------|
| `nativeTools.*` | `toolsSettings.*` | Native tool permissions (shell, read, write, glob, grep, web-fetch, aws) |
| `nativeTools.*.allow` | `toolsSettings.*.allowedCommands` or `allowedPaths` | Depends on tool type |
| `nativeTools.*.deny` | `toolsSettings.*.deniedCommands` or `deniedPaths` | Depends on tool type |
| `nativeTools.*.forceAllow` | `toolsSettings.*.forceAllowedCommands` or `forceAllowedPaths` | Depends on tool type |
| `knowledge` | `resources` | Knowledge bases are merged into resources array as objects |
| `toolSettings` | `toolsSettings` | Additional tool settings (merged with nativeTools) |
| `subagents.allow` | `toolsSettings.subagent.allowedAgents` | Subagent permissions |
| `mcpServers.*.state` | `mcpServers.*.disabled` | "enabled"/"disabled" string becomes boolean (true if disabled) |

### nativeTools → toolsSettings

**TOML:**
```toml
[nativeTools.shell]
allow = ["git .*", "cargo .*"]
deny = ["rm -rf .*"]
denyByDefault = true

[nativeTools.write]
allow = ["./src/**"]
deny = ["Cargo.lock"]
```

**JSON:**
```json
{
  "toolsSettings": {
    "shell": {
      "allowedCommands": ["git .*", "cargo .*"],
      "deniedCommands": ["rm -rf .*"],
      "denyByDefault": true
    },
    "write": {
      "allowedPaths": ["./src/**"],
      "deniedPaths": ["Cargo.lock"]
    }
  }
}
```

### knowledge → resources

**TOML:**
```toml
resources = ["file://README.md"]

[knowledge.project-docs]
source = "file://docs/"
description = "Project documentation"
```

**JSON:**
```json
{
  "resources": [
    "file://README.md",
    {
      "name": "project-docs",
      "type": "knowledgeBase",
      "source": "file://docs/",
      "description": "Project documentation"
    }
  ]
}
```

Note: `resources` array contains both simple strings and knowledge base objects.

### subagents → toolsSettings.subagent

**TOML:**
```toml
[subagents]
allow = ["pr-review", "code-gen"]
deny = ["admin"]
```

**JSON:**
```json
{
  "toolsSettings": {
    "subagent": {
      "allowedAgents": ["pr-review", "code-gen"]
    }
  }
}
```

Note: `deny` is applied during conversion - only the final allowed list appears in JSON.

### hooks

**TOML:**
```toml
[hooks.agentSpawn.spawn]
command = "git fetch"
timeout_ms = 5000

[hooks.preToolUse.pre]
command = "echo before"
matcher = "git.*"
```

**JSON:**
```json
{
  "hooks": {
    "agentSpawn": [
      {
        "command": "git fetch",
        "timeout_ms": 5000
      }
    ],
    "preToolUse": [
      {
        "command": "echo before",
        "matcher": "git.*"
      }
    ]
  }
}
```

Note: TOML uses nested maps (`hooks.TYPE.NAME`), JSON uses arrays (`hooks.TYPE[]`). The NAME key is discarded.

## Inspecting Agent Configuration

Use `kg validate --format json` with jq to query agent configuration:

```bash
# List all agents
kg validate --format json | jq '[.[].name]'

# Show specific agent
kg validate --format json | jq '.[] | select(.name == "rust")'

# Find agents with shell permissions
kg validate --format json | jq '[.[] | select(.toolsSettings.shell) | .name]'

# List all MCP servers across agents
kg validate --format json | jq '[.[] | .mcpServers | keys] | flatten | unique'

# Check if agent has specific tool
kg validate --format json | jq '.[] | select(.name == "rust") | .allowedTools | contains(["@cargo"])'

# Show shell commands for an agent
kg validate --format json | jq '.[] | select(.name == "rust") | .toolsSettings.shell'
```

## Discovering Available Fields

Query schemas to see what fields are valid:

```bash
# List all top-level fields for Kiro agents
kg schema kiro-agent | jq '.properties | keys'

# Show nativeTools structure (TOML)
kg schema agent | jq '.properties.nativeTools'

# Show toolsSettings options (JSON output)
kg schema kiro-agent | jq '.properties.toolsSettings.properties | keys'
```

## Verifying Mappings

To see how kg translates your TOML to JSON:

```bash
# Show TOML sources for an agent
kg tree rust

# Show resulting JSON
kg validate --format json | jq '.[] | select(.name == "rust")'

# Compare specific field transformation
kg validate --format json | jq '.[] | select(.name == "rust") | .toolsSettings'
```

## Updating Schemas

`kg schema` always reflects the installed binary. To update system-installed schemas, update the package:

```bash
# Arch
paru -Syu kiro-generator-git

# Debian
sudo apt upgrade kiro-generator

# Homebrew
brew upgrade kiro-generator

# cargo
cargo install kiro-generator
```

## Workflow for Adding/Modifying Fields

When helping a user add or modify a TOML field:

1. Check the schema: `kg schema agent | jq '.properties.FIELD_NAME'`
2. If the field doesn't exist, the user's kg binary may need updating

## Troubleshooting Generation Issues

If `kg generate` succeeds but Kiro doesn't behave as expected:

1. Read the generated agent JSON: `cat ~/.kiro/agents/rust.json`
2. Compare against the schema: `kg schema kiro-agent | jq '.properties'`
3. Compare TOML input (from `kg tree rust`) against generated JSON
4. Check field mappings above - ensure you're looking at the right JSON field
5. If the problem is a kg bug, open an issue at `https://github.com/kiro-generator/kiro-generator` with:
   - TOML input (from `kg tree rust`)
   - Expected JSON output
   - Actual JSON output
