## About

`kiro-generator` (kg) is a CLI tool for managing and generating [Kiro](https://kiro.dev/docs/) agent files.

## Why?

Because managing config files via `JSON` is painful. `kg` lets you define agents in TOML with inheritance, templates, and composition.

## Prerequisites

* [kiro-cli](https://kiro.dev/cli/)
* A distaste for `JSON` config files

## Features

### Composable Configuration

Build agents from reusable templates. Define common settings once, inherit everywhere.

### Hierarchical Structure

- **Global agents** (`~/.kiro/generators/`) - Available in all projects
- **Local agents** (`.kiro/generators/`) - Project-specific overrides

### Inheritance & Templates

Create base agents with common tools and permissions. Extend them for specific use cases.

### Force Permissions

Override permission restrictions in child agents. Ensure critical capabilities are always available.

### Vendor Packages

Share complete agent configurations (MCP servers + resources + permissions). See [VENDOR.md](https://github.com/CarteraMesh/kiro-generator/blob/main/VENDOR.md) for future plans.
