# kg Introspection Skill Design

## Problem Statement

Agents need to understand and navigate `kg` configuration without reading source code or manually tracing TOML files. Currently, to find where `" rm .*"` is defined for the `rust` agent, an agent must:

1. Know that `.kiro/agents/rust.json` is generated
2. Understand the inheritance model
3. Manually grep through `~/.kiro/generators/agents/` and `.kiro/generators/agents/`
4. Trace parent relationships in manifests
5. Understand merge semantics

This is fragile and requires intimate knowledge of `kg`'s internals.

## Real-World Example: Finding `" rm .*"`

**Current workflow (manual):**
```bash
# Agent sees this in generated file
jq .toolsSettings.shell.deniedCommands .kiro/agents/rust.json
# Output includes: " rm .*"

# Agent must manually trace:
grep -r "rm \.\*" ~/.kiro/generators/agents/ .kiro/generators/agents/
# Finds: ~/.kiro/generators/agents/default.toml

# Agent must check inheritance:
grep -r "rust.*inherits" ~/.kiro/generators/manifests/
# Discovers: rust inherits from default

# Conclusion: " rm .*" comes from default.toml
```

**Desired workflow (introspection):**
```bash
kg which " rm .*" --agent rust --tool shell --field deniedCommands
# Output:
# Source: ~/.kiro/generators/agents/default.toml
# Line: 42
# Inherited by: rust (via default)
```

## Proposed Commands

### 1. `kg show <agent> [OPTIONS]`

Show agent configuration with source attribution.

```bash
# Show full agent config with sources
kg show rust

# Show inheritance chain
kg show rust --inherits
# Output:
# rust
#   └─ default (~/.kiro/generators/agents/default.toml)
#      └─ git-base (~/.kiro/generators/agents/git-base.toml) [template]

# Show specific field
kg show rust --field toolsSettings.shell.deniedCommands
# Output (with sources):
# deniedCommands:
#   - " rm .*"           [default.toml:42]
#   - ".*kill.*"         [default.toml:43]
#   - "cargo publish .*" [rust.toml:28]

# Show merged vs source
kg show rust --merged
# Shows final merged config (what gets generated)

kg show rust --source
# Shows only what's defined in rust.toml (not inherited)
```

### 2. `kg which <pattern> [OPTIONS]`

Find where a configuration value is defined.

```bash
# Find where a pattern is defined
kg which " rm .*" --agent rust
# Output:
# Pattern: " rm .*"
# Agent: rust
# Field: toolsSettings.shell.deniedCommands
# Source: ~/.kiro/generators/agents/default.toml:42
# Inherited: yes (via default)

# Find all agents using a pattern
kg which "cargo publish" --all
# Output:
# Pattern: "cargo publish"
# Found in:
#   - rust (toolsSettings.shell.deniedCommands) [rust.toml:28]
#   - rust-ci (toolsSettings.shell.deniedCommands) [rust-ci.toml:15]

# Find by tool and field
kg which --tool shell --field allowedCommands --value "cargo .+" --agent rust
# Output:
# Value: "cargo .+"
# Agent: rust
# Tool: shell
# Field: allowedCommands
# Source: ~/.kiro/generators/agents/rust.toml:18
```

### 3. `kg trace <agent>`

Show detailed resolution and merge process.

```bash
# Trace agent resolution
kg trace rust
# Output:
# Resolving agent: rust
# 1. Loading manifest: .kiro/generators/manifests/kg.toml
#    - Found: rust { inherits = ["default"] }
# 2. Resolving parent: default
#    - Loading: ~/.kiro/generators/agents/default.toml
#    - Merged: toolsSettings.shell.deniedCommands (3 entries)
# 3. Loading agent: rust
#    - Loading: ~/.kiro/generators/agents/rust.toml
#    - Merged: toolsSettings.shell.allowedCommands (5 entries)
#    - Added: mcpServers.context7
# 4. Final agent: rust
#    - Location: .kiro/agents/rust.json
#    - Templates: 0
#    - MCP Servers: 1

# Trace specific field
kg trace rust --field toolsSettings.shell.deniedCommands
# Output:
# Field: toolsSettings.shell.deniedCommands
# Resolution order:
# 1. default.toml: [" rm .*", ".*kill.*", "pulumi up.*"]
# 2. rust.toml: [" rm .*", ".*kill.*", "pulumi up.*", "cargo publish .*"]
#    ^ Added: "cargo publish .*"
# Final: 4 entries
```

### 4. `kg list [OPTIONS]`

List agents, templates, and configuration.

```bash
# List all agents
kg list

# List templates only
kg list --templates

# List agents inheriting from a template
kg list --inherits-from default

# List agents with specific tool
kg list --has-tool shell

# List agents with MCP servers
kg list --has-mcp
```

### 5. `kg inspect <file>`

Inspect a TOML file without generating.

```bash
# Validate TOML syntax
kg inspect ~/.kiro/generators/agents/rust.toml

# Show what would be generated
kg inspect ~/.kiro/generators/agents/rust.toml --preview

# Check for issues
kg inspect ~/.kiro/generators/agents/rust.toml --lint
```

## SKILL.md Structure

```markdown
# kg - Kiro Agent Generator

## Overview
kg generates Kiro agent JSON files from composable TOML configurations.

## Quick Reference

### Generate agents
kg generate                    # Generate all agents
kg generate --local            # Only local agents
kg generate --global           # Only global agents

### Validate configuration
kg validate                    # Validate all
kg validate --debug            # Show debug output
kg validate --trace rust       # Trace specific agent

### Introspection
kg show <agent>                # Show agent config
kg show <agent> --inherits     # Show inheritance chain
kg which <pattern> --agent <agent>  # Find pattern source
kg trace <agent>               # Trace resolution process
kg list                        # List all agents

### Debugging
kg diff                        # Show changes vs existing
kg validate --trace all        # Trace all agents (verbose)

## Configuration Structure

Global: ~/.kiro/generators/
  ├── manifests/*.toml         # Agent declarations
  └── agents/*.toml            # Agent definitions

Local: .kiro/generators/
  ├── manifests/*.toml
  └── agents/*.toml

Generated: .kiro/agents/*.json (local) or ~/.kiro/agents/*.json (global)

## Inheritance Model

Agents inherit from parent agents via `inherits = ["parent1", "parent2"]`.
Configuration is deep-merged (arrays concatenate, objects merge).

Templates (`template = true`) provide reusable config but don't generate files.

## Common Workflows

### Find where a setting comes from
kg which "<pattern>" --agent <agent>

### Understand inheritance
kg show <agent> --inherits

### Debug generation issues
kg validate --trace <agent>

### Preview changes
kg diff

### Add permissions to an agent
**User request:** "Add path `~/.config/agents/**` to my default agent so that everyone can read this path"

**Workflow:**
```bash
# 1. Find where default agent is configured
kg show default --source
# Output shows: ~/.config/agents/generators/agents/default.toml

# 2. Check current read permissions
kg show default --field nativeTools.read
# Output:
# nativeTools.read:
#   allow:
#     - "~/.config/agents/scratchpad/**"  [default.toml:32]

# 3. Edit the file
$EDITOR ~/.config/agents/generators/agents/default.toml
# Add to [nativeTools.read] allow array: "~/.config/agents/**"

# 4. Validate changes
kg validate --trace default
# Verify the new path appears in merged config

# 5. Generate updated agent
kg generate
```

**Result:** All agents inheriting from `default` now have read access to `~/.config/agents/**`

## Examples

[Real-world examples of common tasks]
```

## Implementation Notes

### Data Structures Needed

```rust
// Source attribution for config values
struct ConfigSource {
    file: PathBuf,
    line: usize,
    inherited: bool,
    parent: Option<String>,
}

// Traced resolution step
struct ResolutionStep {
    agent: String,
    action: String,  // "loaded", "merged", "added"
    field: String,
    value: Value,
    source: ConfigSource,
}
```

### Commands to Implement

1. **`kg show`** - Requires tracking sources during merge
2. **`kg which`** - Requires reverse lookup from value to source
3. **`kg trace`** - Requires recording resolution steps
4. **`kg list`** - Already have most data, just format differently
5. **`kg inspect`** - Parse and validate single file

### Challenges

- **Source tracking**: Need to preserve file/line info through merge process
- **Reverse lookup**: Need index from value → source
- **Performance**: Tracing adds overhead, make it opt-in
- **Output format**: Balance human-readable vs machine-parseable

## Benefits for Agents

1. **Discoverability**: Agents can explore config without reading code
2. **Debugging**: Quickly find why a setting has a specific value
3. **Validation**: Verify changes before generating
4. **Learning**: Understand inheritance and merge semantics through examples
5. **Automation**: Machine-readable output for scripting


## User Questions and Requests

* How many agents use the context7 MCP server?
* Who inherits the "default" agent?
* How many template agents do I have?
* Which agents refers (uses) resource ~/.config/agents/resources/me.md ?
* Who has access to kill "firecrawl" ?
* Add `git commit.*` to my default's agent shell nativeTool deny list
* Create a new agent name "rust-tester" and have this inherit the "subagent-default" agent

## New Users

Examine my current kiro JSON config and recommend a new config layout/structure for `kg` that is flexible and easy to understand.

## Next Steps

1. Design source tracking in merge logic
2. Implement `kg show` with basic source attribution
3. Add `kg which` for pattern search
4. Create SKILL.md with examples
5. Test with agents who don't know kg internals


