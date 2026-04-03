# Defining Agents

In `kg`, an agent is usually defined across one or more TOML sources.

Those sources can live in:

- `manifests/*.toml`
- `agents/<name>.toml`
- global config
- local project config

That is why splitting "manifests" and "agent files" into separate mental buckets can be misleading. They are not rival systems. They are two ways to contribute configuration to the same final agent.

## The Two Building Blocks

`manifests/` is where you declare agents, templates, and inheritance.

`agents/` is where you usually put the heavier per-agent configuration.

But manifests are not limited to declarations. They can also define agent config inline.

That means all of these are valid:

- declare an agent in a manifest and keep the rest in `agents/rust.toml`
- define the whole agent inline in a manifest
- mix both approaches and let them merge

## A Real Example: `rust`

This repo already has a real `rust` agent, and it is a good example of how `kg` is actually used.

Running:

```bash
kg tree details rust
```

shows that `rust` is built from three sources:

- a global agent file: `/home/user/.kiro/generators/agents/rust.toml`
- a global manifest: `/home/user/.kiro/generators/manifests/base.toml`
- a local manifest: `.kiro/generators/manifests/rust.toml`

It also shows that the local manifest contributes:

- `description`
- `inherits`
- `skills.kg-helper`

That is the useful part to notice: the project-specific override for `rust` lives inline in a manifest, not in `.kiro/generators/agents/rust.toml`.

## What That Looks Like

The local project defines a reusable template in `.kiro/generators/manifests/project-resources.toml`:

```toml
[agents.project-resources]
template = true

[agents.project-resources.resources.rust]
locations = [
  "docs/src/SUMMARY.md",
  "docs/src/config/*.md",
  "docs/kiro/configuration-reference.md"
]
```

Then it extends `rust` inline in `.kiro/generators/manifests/rust.toml`:

```toml
[agents.rust]
description = "kiro-generator specific rust agent"
inherits = ["project-resources"]

[agents.rust.skills.kg-helper]
disabled = true
```

So the final `rust` agent is not defined in just one place:

- the global config provides the baseline Rust agent
- the local template adds project-specific resources and knowledge
- the local manifest augments `rust` to inherit from that template

That is a very normal `kg` pattern.

## When To Use A Manifest

Use a manifest when you want to:

- declare an agent or template
- define inheritance with `inherits`
- keep short inline config close to the declaration
- add small project-specific overrides without creating another file

This is especially handy for local project config, where one small inline override is often easier to understand than another `agents/<name>.toml` file.

## When To Use An Agent File

Use `agents/<name>.toml` when:

- the agent has a lot of configuration
- you want one file per agent
- the config includes larger MCP, shell, hook, or resource sections
- you want a clean separation between declaration and implementation

If the agent is getting long, move the heavier parts into `agents/<name>.toml`.

## Inline And External Merge Together

This is the part that matters most:

- manifests can define config inline
- agent files can define config externally
- global and local config can both contribute
- `kg` merges all of it into the final agent

So think less in terms of "where is the one true definition?" and more in terms of "which sources contribute to this agent?"

When you need that answer, use:

```bash
kg tree details <agent-name>
```

It will tell you exactly which files contribute and which fields each one modifies.

## A Minimal Pattern

If you want a simple starting point, this is still a good baseline:

```toml
# manifests/kg.toml
[agents]
default = { inherits = [] }
rust = { inherits = ["default"] }
```

```toml
# agents/rust.toml
description = "Rust development agent"
allowedTools = ["read", "knowledge", "web_search", "@rustdocs"]
```

Then, later, a project can add a local manifest override:

```toml
# .kiro/generators/manifests/rust.toml
[agents.rust]
inherits = ["project-resources"]
description = "Rust development agent for this project"
```

That is often enough.

## Practical Advice

- Start small
- Put inheritance and templates in manifests
- Put larger agent bodies in `agents/<name>.toml`
- Use inline manifest config for small local overrides
- Use `kg tree details <agent-name>` when you are unsure where a value comes from

If the docs ever make this sound more complicated than it is, the short version is: define agents wherever it is clearest, then let `kg` merge the pieces.
