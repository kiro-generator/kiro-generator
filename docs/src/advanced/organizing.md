# Organizing Agents

For complex projects with many agents, you can organize agent definitions into subdirectories.

## Subdirectories in agents/

Agent definition files can be nested in subdirectories up to 5 levels deep:

```
~/.kiro/generators/agents/
├── default.toml
├── rust.toml
└── aws-mcps/
    ├── eks.toml
    ├── iam.toml
    └── s3.toml
```

kg recursively searches for `<agent-name>.toml` in the `agents/` directory tree.

## Example: AWS MCP Agents

Organize related agents by category:

**Directory structure:**
```
.kiro/generators/
├── manifests/
│   └── kg.toml
└── agents/
    └── aws-mcps/
        ├── eks.toml
        └── iam.toml
```

**manifests/kg.toml:**
```toml
[agents]
eks = { template = true }
iam = { template = true }
aws-secops = { inherits = ["eks", "iam"] }
```

**agents/aws-mcps/eks.toml:**
```toml
description = "AWS EKS MCP Server agent"
allowedTools = ["@aws-eks"]

[mcpServers.aws-eks]
command = "uvx"
args = ["awslabs.aws-eks-mcp-server@latest"]
```

**agents/aws-mcps/iam.toml:**
```toml
description = "AWS IAM MCP Server agent"
allowedTools = ["@aws-iam"]

[mcpServers.aws-iam]
command = "uvx"
args = ["awslabs.aws-iam-mcp-server@latest"]
```

The `aws-secops` agent inherits from both `eks` and `iam`, combining their MCP servers and tools.

## Validation

kg validates that agent names are unique across all subdirectories:

```bash
kg validate
```

If duplicate names are found, you'll get an error:

```
Error: Duplicate agent 'eks' found in local agents:
  - .kiro/generators/agents/aws-mcps/eks.toml
  - .kiro/generators/agents/aws/eks.toml
```

## Best Practices

- **Group by domain**: `aws-mcps/`, `dev-tools/`, `security/`
- **Keep depth shallow**: 1-2 levels is usually enough
- **Use descriptive names**: `aws-mcps/eks.toml` not `a/e.toml`
- **Combine with templates**: Use subdirectories for template components

## Limits

- Maximum depth: 5 levels
- Agent names must be unique across all subdirectories
- Subdirectory names don't affect agent names (only the filename matters)
