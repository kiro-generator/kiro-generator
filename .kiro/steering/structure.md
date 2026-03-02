# Project Structure

## Data Flow

1. `commands/mod.rs` parses CLI args via clap
2. `commands/execute.rs` dispatches to the appropriate command
3. `generator/discover.rs` finds all manifest TOML files across config locations
4. `kg_config/` deserializes TOML into Rust structs via facet
5. `generator/merge.rs` resolves inheritance chains
6. `kiro/` serializes the merged result to Kiro agent JSON
7. `output.rs` formats results for the terminal

## Application Configuration Locations

- Global: `~/.kiro/generators/manifests/*.toml` and `~/.kiro/generators/agents/*.toml`
- Local: `.kiro/generators/manifests/*.toml` and `.kiro/generators/agents/*.toml`
- Output: `~/.kiro/agents/*.json` (global) and `.kiro/agents/*.json` (local)

## Test Data

- `fixtures/` -- TOML fixtures for deserialization and merge tests
- `fixtures/kiro/generators/` -- Fixture config copied into test chroot as local config
- `fixtures/kiro/global/` -- Fixture config copied into test chroot as `~/.kiro/generators/`

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
