# CLI Commands

## validate

Validate configuration without generating files.

```bash
kg validate [OPTIONS]
```

**Options:**
- `--local` - Use only local configuration (ignore global `~/.kiro/generators/`)
- `-g, --global` - Use only global configuration (ignore local `.kiro/generators/`)
- `--show-templates` - Show template agents in output
- `-d, --debug` - Enable debug output
- `-t, --trace <AGENT_NAME>` - Enable trace level debug for an agent (use `all` for everything, very verbose)
- `-c, --color <WHEN>` - When to show color: `always`, `auto`, `never` (default: `auto`)
- `-f, --format <FORMAT>` - Format of console output: `table`, `json` (default: `table`)

**Output:**

Table view shows:
- Agent name
- Location (📁 local, 🏠 global, 🌍 both)
- MCP servers configured
- Override (Allowed) Permissions

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

## generate

Generate agent JSON files for kiro-cli.

```bash
kg generate [OPTIONS]
```

**Options:**
- `--local` - Use only local configuration (ignore global `~/.kiro/generators/`)
- `-g, --global` - Use only global configuration (ignore local `.kiro/generators/`)
- `--show-templates` - Show template agents in output
- `-d, --debug` - Enable debug output
- `-t, --trace <AGENT_NAME>` - Enable trace level debug for an agent (use `all` for everything, very verbose)
- `-c, --color <WHEN>` - When to show color: `always`, `auto`, `never` (default: `auto`)
- `-f, --format <FORMAT>` - Format of console output: `table`, `json` (default: `table`)

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
```

## diff

Show differences between generated and existing agent files.

```bash
kg diff [OPTIONS]
```

**Options:**
- `-d, --debug` - Enable debug output
- `-t, --trace <AGENT_NAME>` - Enable trace level debug for an agent (use `all` for everything, very verbose)
- `-c, --color <WHEN>` - When to show color: `always`, `auto`, `never` (default: `auto`)
- `-f, --format <FORMAT>` - Format of console output: `table`, `json` (default: `table`)

**Output:**

Shows element-level differences between what would be generated and what currently exists:

```
Agent: rust
  resources.[3]: - "file://dummy.md"
  toolsSettings.shell.allowedCommands.[2]: + "cargo test .*"
  knowledge.facet.description: "old description" → "new description"
```

**Stability:**

The diff output is deterministic - running it multiple times on unchanged files produces identical results. This is achieved by normalizing both generated and existing agents before comparison.

**Examples:**

```bash
# Diff all agents
kg diff

# JSON output for scripting
kg diff --format json
```

**Use Cases:**

- Preview changes before running `kg generate`
- Verify configuration changes have expected effects
- Debug inheritance and merge behavior
- CI/CD validation (exit code 0 = no changes, 1 = differences found)

## schema

Generate JSON schemas for IDE autocompletion and validation.

See [Configuration Schema](./schema.md) for details.

## Output Formats

### Table (default)

Human-readable table with agent details:

```
╭────────────────────┬─────┬──────────┬────────────────────────────────────╮
│ Agent 🤖 (PREVIEW) ┆ Loc ┆ MCP 💻   ┆    Override (Allowed) Permissions  │
╞════════════════════╪═════╪══════════╪════════════════════════════════════╡
│ rust               ┆ 📁  ┆ context7 ┆                                    │
╰────────────────────┴─────┴──────────┴────────────────────────────────────╯
```

### JSON

Machine-readable output for scripting:

```bash
kg validate --format json | jq '.agents[] | select(.name == "rust")'
```
