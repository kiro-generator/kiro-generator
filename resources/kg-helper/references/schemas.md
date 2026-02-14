# Schemas

kg provides JSON schemas for validating configuration and inspecting available fields. Schemas are installed locally by `kg bootstrap` — these paths won't exist until bootstrap has run. Schemas can be updated on demand.

## Available Schemas

| Schema | Describes | Local Path |
|--------|-----------|------------|
| `kiro-agent` | Final Kiro agent JSON format (what kg generates) | `~/.kiro/skills/kg-helper/assets/kiro-agent.json` |
| `kg-manifest` | kg manifest TOML files (`manifests/*.toml`) | `~/.kiro/skills/kg-helper/assets/kg-manifest.json` |
| `kg-agent` | kg agent TOML files (`agents/*.toml`) | `~/.kiro/skills/kg-helper/assets/kg-agent.json` |

**Note:** Local schemas may be outdated if kg has been updated since bootstrap. See "Updating Schemas" below.

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

Read local schemas to see what fields are valid:

```bash
# List all top-level fields for Kiro agents
jq '.properties | keys' ~/.kiro/skills/kg-helper/assets/kiro-agent.json

# Show nativeTools structure (TOML)
jq '.properties.nativeTools' ~/.kiro/skills/kg-helper/assets/kg-agent.json

# Show toolsSettings options (JSON output)
jq '.properties.toolsSettings.properties | keys' ~/.kiro/skills/kg-helper/assets/kiro-agent.json
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

If local schemas are outdated (missing fields you know exist):

```bash
# Update specific schema
kg schema kiro-agent > ~/.kiro/skills/kg-helper/assets/kiro-agent.json
kg schema manifest > ~/.kiro/skills/kg-helper/assets/kg-manifest.json
kg schema agent > ~/.kiro/skills/kg-helper/assets/kg-agent.json
```

**Fallback URLs** (if `kg schema` output still doesn't match Kiro docs):

```
https://kiro-generator.io/kiro-agent.json
https://kiro-generator.io/kg-manifest.json
https://kiro-generator.io/kg-agent.json
```

## Workflow for Adding/Modifying Fields

When helping a user add or modify a TOML field:

1. Check local schema first: `jq '.properties.FIELD_NAME' ~/.kiro/skills/kg-helper/assets/kg-agent.json`
2. If field is missing, update schema: `kg schema agent > ~/.kiro/skills/kg-helper/assets/kg-agent.json`
3. If field doesn't exist after updating, tell the user their kg binary may need updating

## Troubleshooting Generation Issues

If `kg generate` succeeds but Kiro doesn't behave as expected:

1. Read the generated agent JSON: `cat ~/.kiro/agents/rust.json`
2. Update local kiro-agent schema: `kg schema kiro-agent > ~/.kiro/skills/kg-helper/assets/kiro-agent.json`
3. Compare TOML input (from `kg tree rust`) against generated JSON
4. Check field mappings above - ensure you're looking at the right JSON field
5. If the problem is a kg bug, open an issue at `https://github.com/kiro-generator/kiro-generator` with:
   - TOML input (from `kg tree rust`)
   - Expected JSON output
   - Actual JSON output
