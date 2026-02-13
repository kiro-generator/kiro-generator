# Product Overview

## What is kiro-generator?

`kiro-generator` (binary: `kg`) is a CLI tool that generates Kiro custom agent JSON configuration files from TOML source definitions. It replaces hand-editing JSON with a composable, type-safe TOML workflow.

## Problem

Kiro agent configuration files are verbose JSON. Managing them by hand is error-prone, repetitive, and doesn't scale across projects. There's no inheritance, no reuse, no validation before runtime.

## Target Users

Developers who use Kiro CLI and manage multiple agents across projects. Power users who want DRY, version-controlled agent configurations.

## Key Features

- **TOML-to-JSON generation**: Define agents in TOML, output valid Kiro agent JSON
- **Inheritance**: Agents can inherit from parent agents, overriding only what differs
- **Templates**: Define reusable base agents that aren't generated themselves
- **Hierarchical config**: Global agents (`~/.kiro/generators/`) and local agents (`.kiro/generators/`) with local taking precedence
- **Validation**: `kg validate` checks configuration without writing files
- **Diffing**: `kg diff` compares generated output against existing agent files
- **Schema generation**: `kg schema manifest|agent` outputs JSON schemas for editor support
- **Systemd file watching**: `kg watch` sets up automatic regeneration on config changes (Linux)
- **Bootstrap**: `kg bootstrap` scans existing agent JSON files and installs the kg-helper skill

## Non-Goals

- This is not a library crate -- no one imports kg as a dependency
- Runtime performance is not a priority; the CLI runs infrequently
- No GUI, no TUI -- pure CLI following Unix philosophy
