# Templates

Templates are agent configurations that don't generate JSON files. They exist only to be inherited by other agents.

## Purpose

Use templates to:
- Create reusable configuration components
- Define common tool settings
- Build agent templates
- Organize complex inheritance hierarchies

## Declaration

Mark an agent as a template with `template = true`:

```toml
[agents]
git-base = { template = true }
```

kg will skip generating `~/.kiro/agents/git-base.json` but the configuration is available for inheritance.

## Example: Git Permissions

Create a template with git commands:

```toml
[agents.git-readonly]
template = true

[agents.git-readonly.toolsSettings.execute_bash]
allowedCommands = [
    "git status .*",
    "git fetch .*",
    "git diff .*",
    "git log .*"
]
autoAllowReadonly = true
```

Use it in real agents:

```toml
[agents.rust]
inherits = ["git-readonly"]
allowedTools = ["@rustdocs", "@cargo"]
```

## Example: Permission Levels

Create a hierarchy of permission templates:

```toml
[agents.git-readonly]
template = true
[agents.git-readonly.toolsSettings.execute_bash]
allowedCommands = ["git status .*", "git fetch .*", "git diff .*"]

[agents.git-write]
template = true
inherits = ["git-readonly"]
[agents.git-write.toolsSettings.execute_bash]
allowedCommands = ["git add .*", "git commit .*"]

[agents.git-full]
template = true
inherits = ["git-write"]
[agents.git-full.toolsSettings.execute_bash]
allowedCommands = ["git push .*", "git pull .*"]
```

Then use them:

```toml
[agents.reviewer]
inherits = ["git-readonly"]

[agents.developer]
inherits = ["git-write"]

[agents.maintainer]
inherits = ["git-full"]
```

## Validation

Templates appear in `kg validate` output but are marked as templates:

```bash
kg validate
```

They won't appear in the generated agents directory.

## Best Practices

- Use descriptive names that indicate purpose (`git-readonly`, not `git1`)
- Keep templates focused on a single concern
- Document what each template provides
- Avoid deep template-only inheritance chains (max 2-3 levels)
