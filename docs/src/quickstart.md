# Quick Start

This guide starts with the `--skeleton` path on purpose.

It is the clearest way to learn how `kg` works: which files it creates, what each file is for, and how the pieces fit together. Once that clicks, the regular `kg init` flow makes a lot more sense too.

## Start With The Skeleton

Create the starter configuration:

```bash
kg init --skeleton
```

This creates:

```text
~/.kiro/generators/
├── manifests/
│   └── kg.toml
└── agents/
    ├── default.toml
    └── git.toml
```

That is the core `kg` layout:

- `manifests/` declares which agents exist and how they inherit from each other
- `agents/` contains the actual configuration for each agent

## Look At The Starter Files

`kg init --skeleton` gives you a small but working example.

`~/.kiro/generators/manifests/kg.toml`:

```toml
"$schema" = "https://kiro-generator.io/manifest.json"

[agents]
git = { template = true }
default = { inherits = ["git"] }
```

This manifest says:

- `git` is a template agent, so it is reusable but does not generate a final JSON file
- `default` is a real agent and inherits from `git`

`~/.kiro/generators/agents/git.toml`:

```toml
"$schema" = "https://kiro-generator.io/agent.json"

[nativeTools.shell]
allow = ["git status", "git fetch", "git diff .*", "git log .*"]
deny = ["git commit .*", "git push .*"]
```

This is the reusable git policy. It gives child agents read-ish git access, while keeping write operations on a shorter leash.

`~/.kiro/generators/agents/default.toml`:

```toml
"$schema" = "https://kiro-generator.io/agent.json"

description = "Default agent"
tools = ["*"]
allowedTools = ["read", "knowledge", "web_search"]

[resources.default]
locations = ["README.md"]
```

This is the actual agent definition. When `kg` resolves inheritance, `default` gets its own settings plus the shell rules from `git`.

## Validate The Configuration

Before generating anything, validate it:

```bash
kg validate --global
```

Use `--global` here to keep the example focused on `~/.kiro/generators/`, even if you run the command from a project that has local `.kiro/generators/` files.

Validation shows the final merged agent configuration that `kg` will generate.

## Generate The Agent JSON

When validation looks good, generate the Kiro agent files:

```bash
kg generate --global
```

This writes the generated agent JSON files to:

```text
~/.kiro/agents/
```

In the starter skeleton, `git` is a template, so it is not generated. `default` is the real agent, so that is the one you should expect to see.

## Make Your First Change

Now edit the starter files and repeat the loop:

1. Change `~/.kiro/generators/manifests/kg.toml` if you want to add agents or change inheritance
2. Change files in `~/.kiro/generators/agents/` if you want to change tools, resources, permissions, or MCP configuration
3. Run `kg validate --global`
4. Run `kg generate --global`

That is the whole workflow.

Not glamorous. Very effective.

## Next

Now that the file layout is familiar, you can keep going with:

- [Recursive Self-Improvement](guided-migration.md) for the guided `kg-helper` migration path
- [Directory Structure](config/structure.md)
- [Defining Agents](config/defining-agents.md)
- [Inheritance](config/inheritance.md)
- [Templates](config/templates.md)
