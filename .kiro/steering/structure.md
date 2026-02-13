# Project Structure

## Source Layout

```
src/
  main.rs              -- Entry point, module declarations, Result type alias
  commands/
    mod.rs             -- Clap CLI definition (Cli, Command, all *Args structs)
    execute.rs         -- Command dispatch and execution logic
    bootstrap.rs       -- `kg bootstrap` implementation
    runtime.rs         -- Runtime environment setup
    tree.rs            -- `kg tree` implementation
    watch_linux.rs     -- `kg watch` systemd watcher (Linux)
    watch_peasants.rs  -- `kg watch` stub (non-Linux)
  generator/
    mod.rs             -- Generator orchestration (validate/generate pipeline)
    discover.rs        -- Manifest and agent file discovery across config locations
    config_location.rs -- ConfigLocation enum (Global, Local) and path resolution
    merge.rs           -- Agent inheritance merging logic
  kg_config/
    mod.rs             -- GeneratorConfig (top-level TOML structure), toml_parse helpers
    manifest.rs        -- Manifest struct (per-agent TOML definition)
    agent_file.rs      -- KgAgentFileDoc (standalone agent file format)
    native.rs          -- Native tool permission structs (glob, grep, web_fetch, etc.)
    merge.rs           -- Field-level merge logic for inheritance
    knowledge.rs       -- KgKnowledge config
    subagent.rs        -- SubagentConfig
  kiro/
    mod.rs             -- Kiro agent JSON output types
    diff.rs            -- Diff rendering between generated and existing agents
    tools.rs           -- Kiro tool definitions
    hook.rs            -- Kiro hook definitions
    custom_tool.rs     -- Custom MCP tool definitions
  os/
    mod.rs             -- OS abstraction re-exports
    systemd.rs         -- Systemd unit management
    fs/
      mod.rs           -- Fs enum: Real (production) or Chroot (tests)
      unix.rs          -- Unix-specific filesystem helpers
      windows.rs       -- Windows-specific filesystem helpers
  output.rs            -- Table/JSON/plain output formatting, color handling
  source.rs            -- Source file tracking
  schema.rs            -- JSON schema output command
  schema_optional.rs   -- Optional field schema generation
  tracing_init.rs      -- Tracing subscriber setup
  util.rs              -- Small shared utilities
```

## Data Flow

1. `commands/mod.rs` parses CLI args via clap
2. `commands/execute.rs` dispatches to the appropriate command
3. `generator/discover.rs` finds all manifest TOML files across config locations
4. `kg_config/` deserializes TOML into Rust structs via facet
5. `generator/merge.rs` resolves inheritance chains
6. `kiro/` serializes the merged result to Kiro agent JSON
7. `output.rs` formats results for the terminal

## Configuration Locations

- Global: `~/.kiro/generators/manifests/*.toml` and `~/.kiro/generators/agents/*.toml`
- Local: `.kiro/generators/manifests/*.toml` and `.kiro/generators/agents/*.toml`
- Output: `~/.kiro/agents/*.json` (global) and `.kiro/agents/*.json` (local)

## Test Data

- `data/` -- TOML fixtures for deserialization and merge tests
- `data/kiro/generators/` -- Fixture config copied into test chroot as local config
- `data/kiro/global/` -- Fixture config copied into test chroot as `~/.kiro/generators/`

## Other Directories

- `docs/` -- mdbook documentation (source in `docs/src/`, built to `docs/book/`)
- `docs/kiro/` -- Scraped Kiro reference docs (agent config spec, tools, hooks, etc.)
- `schemas/` -- JSON schema files for manifest and agent validation
- `resources/` -- Systemd unit files, config examples, and the kg-helper skill
  - `resources/kg-helper/` -- Agent skill package (installed by `kg bootstrap`)
    - `SKILL.md` -- Operational guide for agents helping users with kg
    - `references/` -- Detailed docs loaded on demand (templates, bootstrap, schemas)
- `scripts/` -- CI, formatting, coverage scripts
- `.kiro/generators/` -- This project's own kg manifests (dogfooding)
