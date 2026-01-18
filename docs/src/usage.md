# Usage

## Setup 

1. Create global config structure

```bash
kg init
```

This creates:
```
~/.kiro/generators/
├── manifests/
│   └── kg.toml
└── agents/
    └── default.toml
```

2. Add your first agent in `~/.kiro/generators/manifests/kg.toml`

```toml
[agents]
default = { inherits = [] }
```

3. Configure the agent in `~/.kiro/generators/agents/default.toml`

```toml
description = "Default agent"
allowedTools = ["read", "knowledge"]
resources = ["file://README.md"]
```

4. Verify

```bash
kg validate
```

5. Generate

```bash
kg generate
```

## Manifests

`manifests/*.toml` files define the relationship or inheritance of your agents. You can define the relationship between agents using the `inherits` field. For example, if you have an agent named `default` and another agent named `rust`, you can define the relationship between them as follows:

```toml
[agents]
default = { inherits = [] }
rust = { inherits = ["default"] }
```

`kg` will then look for agent configuration files in the following order:

- `~/.kiro/generators/agents/rust.toml`
- `.kiro/generators/agents/rust.toml`

Both can be present and will be merged together.

### "Inline" Agent Configuration

You can define agent properties "inline" in manifest files: 

```toml
[agents.default]
inherits = []
allowedTools = ["read", "knowledge"]

[agents.rust]
inherits = ["default"]
allowedTools = ["@rustdocs", "@cargo"]
```

In this example, the `allowedTools` field is inherited from the `default` agent, and the `rust` agent adds two additional tools: `@rustdocs` and `@cargo`.

### Templates

`Templates` are like agent templates. `kg` will skip generating agent `JSON` files. You can use them as building blocks or components to derive other real agents.

`manifests/kg.toml`: 

```toml
[agents.default]
inherits = ["git"]
allowedTools = ["read", "knowledge"]

[agents.git]
template = true  # do not generate JSON agent config

[agents.git.toolsSettings.shell]
allowedCommands = ["git status .*", "git fetch .*", "git diff .*", "git log .*"]

[agents.git-write]
template = true

[agents.git-write.toolsSettings.shell]
allowedCommands = ["git .*"]

[agents.rust]
inherits = ["default"]
allowedTools = ["@rustdocs", "@cargo"]

[agents.dependabot]
inherits = ["rust", "git-write"]
```

The `dependabot` agent will be able to use any git command.
