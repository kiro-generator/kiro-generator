# Configuration Schema

kg provides JSON schemas for IDE autocompletion and validation.

## kg.toml Schema

For agent declarations in `kg.toml`:

```toml
"$schema" = "https://raw.githubusercontent.com/dougEfresh/kiro-generator/refs/heads/main/schemas/manifest.json"

[agents]
default = { inherits = [] }
```

## Agent Configuration Schema

For individual agent files like `default.toml`, `rust.toml`:

```toml
"$schema" = "https://raw.githubusercontent.com/dougEfresh/kiro-generator/refs/heads/main/schemas/agent.json"

description = "Default agent"
allowedTools = ["read", "knowledge"]
```

## Generating Schemas

Generate schemas locally:

```bash
# Generate manifest schema
kg schema manifest > manifest.json

# Generate agent schema
kg schema agent > agent.json
```

## LSP Configuration

### taplo (Recommended)

Add to `.taplo.toml` in your project:

```toml
[schema]
enabled = true

[[schema.associations]]
path = "**/.kiro/generators/manifests/*.toml"
url = "https://raw.githubusercontent.com/dougEfresh/kiro-generator/refs/heads/main/schemas/manifest.json"

[[schema.associations]]
path = "**/.kiro/generators/agents/**/*.toml"
url = "https://raw.githubusercontent.com/dougEfresh/kiro-generator/refs/heads/main/schemas/agent.json"
```

### tombi

Add to `tombi.toml` in your project:

```toml
[[schemas]]
toml-version = "1.0.0"
path = "https://raw.githubusercontent.com/dougEfresh/kiro-generator/refs/heads/main/schemas/manifest.json"
include = ["**/.kiro/generators/manifests/*.toml"]

[[schemas]]
toml-version = "1.0.0"
path = "https://raw.githubusercontent.com/dougEfresh/kiro-generator/refs/heads/main/schemas/agent.json"
include = ["**/.kiro/generators/agents/**/*.toml"]
```

### VS Code

Install the [Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml) extension. It uses taplo and will automatically pick up the `$schema` field from your TOML files.

### Neovim

Using `nvim-lspconfig`:

```lua
require('lspconfig').taplo.setup({
  settings = {
    evenBetterToml = {
      schema = {
        enabled = true,
        associations = {
          ["**/.kiro/generators/manifests/*.toml"] = "https://raw.githubusercontent.com/dougEfresh/kiro-generator/refs/heads/main/schemas/manifest.json",
          ["**/.kiro/generators/agents/**/*.toml"] = "https://raw.githubusercontent.com/dougEfresh/kiro-generator/refs/heads/main/schemas/agent.json"
        }
      }
    }
  }
})
```

## Benefits

With schema validation you get:

- **Autocompletion** - Field suggestions as you type
- **Validation** - Immediate feedback on typos and invalid values
- **Documentation** - Hover tooltips explaining each field
- **Type checking** - Catch errors before running `kg validate`

## Schema Location

The schema is versioned with releases:

- **Latest:** `https://raw.githubusercontent.com/dougEfresh/kiro-generator/refs/heads/main/schemas/manifest.json`
- **Specific version:** `https://raw.githubusercontent.com/dougEfresh/kiro-generator/refs/tags/v0.1.0/schemas/manifest.json`

Pin to a specific version for stability or use `main` for latest features.
