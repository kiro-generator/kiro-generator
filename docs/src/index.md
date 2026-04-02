# kiro-generator

`kiro-generator` (`kg`) helps you manage [Kiro](https://kiro.dev/docs/) custom agents without hand-editing piles of JSON.

Define agents in TOML, compose them with inheritance and templates, validate the result, then generate the JSON Kiro expects.

If editing agent JSON by hand has ever felt like doing plumbing with a spoon, this tool exists for you.

## Why `kg`?

Kiro agent files are JSON. JSON is fine right up until you want reuse, overrides, shared defaults, or confidence that a larger config change did not quietly break something.

`kg` gives you:

- Reusable agent definitions built from TOML
- Inheritance and templates for shared configuration
- Global config in `~/.kiro/generators/` and project-local config in `.kiro/generators/`
- Validation before generation
- Helpful inspection commands like `diff`, `tree`, and `schema`

## Basic Workflow

Most people will use `kg` like this:

1. Install `kg`
2. Run `kg init`
3. Define agents in manifest and agent config files
4. Run `kg validate`
5. Run `kg generate`
6. Use `kg diff`, `kg tree`, or `kg watch` when you need more visibility or automation

## What This Book Covers

- [Installation](install.md) gets `kg` on your machine
- [Quick Start](quickstart.md) walks through the happy path
- [Directory Structure](config/structure.md) explains where files live
- [Defining Agents](config/defining-agents.md), [Inheritance](config/inheritance.md), and [Templates](config/templates.md) cover configuration
- [CLI Commands](reference/cli.md) is the command reference
- [Troubleshooting](reference/troubleshooting.md) helps when things get weird

## Prerequisites

- [kiro-cli](https://kiro.dev/cli/)
- A working tolerance for TOML
- A mild distrust of hand-maintained JSON
