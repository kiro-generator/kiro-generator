# Systemd Watch (WIP)

> **Work in progress** -- this feature is under active development. Behavior and CLI flags may change.

Automatically regenerate Kiro agent files when your configuration changes, using systemd path units.

**Linux only.** On other platforms, `kg watch` exits with an error.

## Quick Start

```bash
# Watch the current project directory
kg watch

# Watch a specific path
kg watch /home/user/projects/my-app

# List active watchers
kg watch --list

# Disable a watcher
kg watch --disable
```

## How It Works

`kg watch` manages systemd user path units (`kiro-generator-local@<escaped-path>.path`) via D-Bus. When systemd detects changes in your `.kiro/generators/` directory, it triggers `kg generate` automatically.

The path is escaped using the same algorithm as `systemd-escape --path` (see `systemd.unit(5)`).

## Environment Files

`kg init` creates environment files that control how systemd-triggered runs behave:

```
$XDG_CONFIG_HOME/kg/systemd/
├── global.env    # Applied to all systemd-triggered runs
└── home.env      # Applied to home-directory agent generation
```

### global.env

```bash
KG_FORMAT=plain
KG_DIFF=true
```

### home.env

```bash
KG_NOTIFY=true
# KG_DEBUG=true
# KG_FORMAT=plain
# KG_COLOR=auto
# KG_DIFF=true
# KG_FORCE=true
```

These map to CLI flags via clap's `env` attribute:

| Env Var      | CLI Flag    |
|--------------|-------------|
| `KG_DEBUG`   | `--debug`   |
| `KG_COLOR`   | `--color`   |
| `KG_FORMAT`  | `-f`        |
| `KG_NOTIFY`  | `--notify`  |
| `KG_DIFF`    | `--diff`    |
| `KG_FORCE`   | `--force`   |

## Prerequisites

The systemd unit files must be installed in `~/.config/systemd/user/` before `kg watch` can enable them. See the project's `resources/systemd/` for reference.

## Listing and Disabling

```bash
# Show all active kg watchers and their state
kg watch --list

# Stop and disable a watcher for the current directory
kg watch --disable

# Stop and disable for a specific path
kg watch --disable /home/user/projects/my-app
```
