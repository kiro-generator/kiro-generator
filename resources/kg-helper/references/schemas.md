# Schemas

kg embeds JSON schemas for validating manifest and agent TOML files. Use these to verify field names, check available options, or validate configuration before running `kg validate`.

## Available Schemas

| Schema | Validates | Command |
|--------|-----------|---------|
| manifest | `manifests/*.toml` files (agent declarations, inheritance) | `kg schema manifest` |
| agent | `agents/*.toml` files (agent configuration) | `kg schema agent` |
| kiro-agent | `~/.kiro/agents/*.json` files (output or result of the TOML configuration) | `kg schema kiro-agent` |

`kiro-agent`, which describes the **output** format — the generated Kiro agent JSON files (e.g., `~/.kiro/agents/rust.json`).
This schema is embedded in the kg binary and used automatically during `kg generate` to validate output before writing.
You don't normally need it, but it's useful for troubleshooting (see below).

## Getting the Schema

**Primary source** -- always matches your installed kg version:

```bash
# Output manifest schema to stdout
kg schema manifest

# Output agent schema to stdout
kg schema agent

# Save to a file for editor integration
kg schema manifest > manifest.schema.json
kg schema agent > agent.schema.json
```

**Fallback** -- if your kg binary is outdated and missing newer fields:

```
https://kiro-generator.io/manifest.json
https://kiro-generator.io/agent.json
https://kiro-generator.io/kiro-agent.json
```

## Usage Guidance

When helping a user add or modify a TOML field:

1. Run `kg schema manifest` or `kg schema agent` to get the current schema
2. Verify the field exists and check its type/constraints
3. If the field isn't in the schema but Kiro docs reference it, fetch the latest from the URL
4. If neither source has the field, tell the user their kg binary may need updating

The schema is also useful for discovering available fields. If a user asks "what options can I set for nativeTools?", pipe the schema through jq:

```bash
kg schema agent | jq '.properties.nativeTools'
```

## Troubleshooting with the Output Schema

`kg generate` validates all generated JSON against the embedded `kiro-agent.json` schema before writing files. If generation succeeds but Kiro still doesn't behave as expected:

1. Read the generated agent JSON (e.g., `~/.kiro/agents/rust.json`)
2. Fetch the latest Kiro agent schema from `https://kiro-generator.io/kiro-agent.json`
3. Validate the generated JSON against the latest schema — a mismatch means kg's embedded schema is stale
4. Compare the TOML input (from `kg tree` sources) against the generated JSON to identify where the translation diverged
5. If the problem is a kg bug: help the user work around it by adjusting TOML to avoid the broken path, and open a GitHub issue at `https://github.com/kiro-generator/kiro-generator` with the TOML input, expected JSON output, and actual JSON output
