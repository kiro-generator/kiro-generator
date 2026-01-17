# Quick Start 

1. Define your agents in `~/.kiro/generators/manifests/kg.toml`

```toml
[agents]
default = { inherits = [] }
rust = { inherits = ["default"] }
```

2. Define your agent configurations in `~/.kiro/generators/agents/<agent-name>.toml`

`~/.kiro/generators/agents/default.toml`:
```toml
description = "Default agent"
tools = ["*"]
allowedTools = ["read", "knowledge", "web_search"]
resources = ["file://README.md", "file://AGENTS.md"]

[toolsSettings.shell]
allowedCommands = ["git status", "git fetch", "git diff .*"]
deniedCommands = ["git commit .*", "git push .*"]
autoAllowReadonly = true
```

`~/.kiro/generators/agents/rust.toml`:
```toml
description = "General Rust agent"
resources = ["file://RUST.md"]
allowedTools = ["@rustdocs", "@cargo"]

[mcpServers.rustdocs]
command = "rust-docs-mcp"
timeout = 1000

[mcpServers.cargo]
command = "cargo-mcp"
timeout = 1200

[toolsSettings.shell]
allowedCommands = ["cargo .+"]
deniedCommands = ["cargo publish .*"]
```

3. Validate

```shell
$ kg validate 
╭────────────────────┬─────┬─────────────────┬────────────────────────────────────────────────┬────────────────────┬────────┬────────┬────────╮
│ Agent 🤖 (PREVIEW) ┆ Loc ┆ MCP 💻          ┆ Allowed Tools ⚙️                               ┆ Resources 📋       ┆    Forced Permissions    │
╞════════════════════╪═════╪═════════════════╪════════════════════════════════════════════════╪════════════════════╪══════════════════════════╡
│ default            ┆ 📁  ┆                 ┆ knowledge, read, web_search                    ┆ - file://README.md ┆                          │
│                    ┆     ┆                 ┆                                                ┆ - file://AGENTS.md ┆                          │
│                    ┆     ┆                 ┆                                                ┆                    ┆                          │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ rust               ┆ 📁  ┆ cargo, rustdocs ┆ @cargo, @rustdocs, knowledge, read, web_search ┆ - file://README.md ┆                          │
│                    ┆     ┆                 ┆                                                ┆ - file://AGENTS.md ┆                          │
│                    ┆     ┆                 ┆                                                ┆ - file://RUST.md   ┆                          │
│                    ┆     ┆                 ┆                                                ┆                    ┆                          │
╰────────────────────┴─────┴─────────────────┴────────────────────────────────────────────────┴────────────────────┴──────────────────────────╯

🎉 Config is valid
→ Run kg generate to generate agent files
```

4. Generate

```shell
$ kg generate
```
