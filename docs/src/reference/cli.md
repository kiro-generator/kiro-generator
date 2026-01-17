# CLI Commands

## Global Options

```
kg [OPTIONS] <COMMAND>
```

**Options:**
- `-d, --debug` - Enable debug output
- `-t, --trace <AGENT_NAME>` - Enable trace logging for specific agent (use `all` for everything)
- `-c, --color <WHEN>` - Control color output: `always`, `auto`, `never` (default: `auto`)
- `-f, --format <FORMAT>` - Output format: `table`, `json` (default: `table`)
- `-h, --help` - Show help
- `-V, --version` - Show version

## Commands

### validate

Validate configuration without generating files.

```bash
kg validate [OPTIONS]
```

**Options:**
- `--local` - Ignore global `~/.kiro/generators/` config
- `--global` - Ignore local `.kiro/generators/` config
- `--show-templates` - Include template agents in output

**Output:**

Table view shows:
- Agent name
- Location (📁 local, 🏠 global, 🌍 both)
- MCP servers configured
- Allowed tools
- Resources
- Forced permissions

**Examples:**

```bash
# Validate all agents
kg validate

# Only validate local agents
kg validate --local

# Show templates in output
kg validate --show-templates

# JSON output for scripting
kg validate --format json
```

### generate

Generate agent JSON files for kiro-cli.

```bash
kg generate [OPTIONS]
```

**Options:**
- `--local` - Ignore global config, only generate local agents
- `--global` - Ignore local config, only generate global agents
- `--show-templates` - Include template agents in output

**Output:**

Generates JSON files:
- Global agents → `~/.kiro/agents/<agent-name>.json`
- Local agents → `.kiro/agents/<agent-name>.json`

Templates are never generated.

**Examples:**

```bash
# Generate all agents
kg generate

# Only generate global agents
kg generate --global

# Only generate local agents
kg generate --local

# Debug specific agent generation
kg generate --trace rust
```

### version

Display version information.

```bash
kg version
```

Shows kg version and build info.

## Output Formats

### Table (default)

Human-readable table with agent details:

```
╭────────────────────┬─────┬─────────────────┬────────────────────────────────────────────────╮
│ Agent 🤖 (PREVIEW) ┆ Loc ┆ MCP 💻          ┆ Allowed Tools ⚙️                               │
╞════════════════════╪═════╪═════════════════╪════════════════════════════════════════════════╡
│ rust               ┆ 📁  ┆ cargo, rustdocs ┆ @cargo, @rustdocs, knowledge, read, web_search │
╰────────────────────┴─────┴─────────────────┴────────────────────────────────────────────────╯
```

### JSON

Machine-readable output for scripting:

```bash
kg validate --format json | jq '.agents[] | select(.name == "rust")'
```

## Debugging

Use trace logging to debug configuration issues:

```bash
# Trace specific agent
kg validate --trace rust

# Trace all agents (very verbose)
kg validate --trace all

# Debug mode (less verbose than trace)
kg validate --debug
```

Trace output shows:
- Configuration file loading
- Inheritance resolution
- Merge operations
- Force property application
