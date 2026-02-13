---
name: kg-helper
description: Help set up and manage kg (kiro-generator) TOML agent configurations. Use when users ask about kg, migrating JSON agents to kg, or bootstrapping a new kg setup.
license: MIT
compatibility: Requires access to user's kg configuration files in ~/.kiro/generators/ and .kiro/generators/
metadata:
  version: 0.1.0
  author: agents
---

## What is kg?

`kg` (kiro-generator) generates Kiro agent JSON files from composable TOML configurations. It solves the problem of managing agent configs by hand:

- **Composable**: Build agents from reusable templates
- **Hierarchical**: Global configs merge with local project configs
- **Deterministic**: Inherit and extend -- no gaps, no drift

## Step 1: Discovery

Before reading or modifying any config files, run discovery to understand the current setup:

```bash
# Show all agents
kg tree

# Single agent
kg tree rust

# Multiple agents
kg tree rust node
```

Example JSON output:
```json
{
  "rust": {
    "template": false,
    "description": "Rust development agent",
    "sources": [
      {"type": "local-manifest", "path": ".kiro/generators/manifests/kg.toml"},
      {"type": "global-manifest", "path": "/home/user/.kiro/generators/manifests/base.toml"},
      {"type": "global-file", "path": "/home/user/.kiro/generators/agents/lang/rust.toml"}
    ],
    "inherits": ["default", "kg-resources"]
  }
}
```

**Source types:**
- `global-manifest` / `local-manifest` — Agent declared inline in a manifests/*.toml file
- `global-file` / `local-file` — Dedicated agent config file in agents/ directory

**Use the `sources` array to know exactly which files to read or edit.**

## Step 2: Validate and Generate

**Scope detection:** kg automatically determines whether to operate on global or local agents based on your current directory:

- **In a project with `.kiro/generators/`**: Commands default to local scope
- **Outside a project**: Commands default to global scope
- **Force global scope**: Use `--global` flag when you're in a project but need to work with global agents

This matters when you edit global agent files while in a project directory. Without `--global`, your changes won't be validated or generated.

```bash
# When in a project directory and editing GLOBAL agents:
kg validate --global
kg generate --global

# When in a project directory and editing LOCAL agents:
kg validate
kg generate

# When outside any project (global scope is automatic):
kg validate
kg generate
```

**Rule of thumb:** If `kg tree <agent>` shows `"type": "global-file"` or `"type": "global-manifest"` in sources, and you're in a project directory, use `--global`.

## Step 2.5: Preview Changes with Diff

Before running `kg generate`, use `kg diff` to preview what will change:

```bash
# Preview changes (same scope rules as validate/generate)
kg diff --global        # When editing global agents in a project
kg diff                 # When editing local agents

# Compact output (RECOMMENDED - shows only changed paths)
kg diff --global --compact

# Plain output (no colors, for CI/scripts)
kg diff --global --plain

# Both flags together
kg diff --global --compact --plain
```

**Output formats:**
- **Default**: Full structural diff with colors (verbose, shows entire structure)
- **Compact** (recommended): Path-based output showing only what changed
  ```
  shell.denied_commands.0: + "pkill .*"
  description: "old text" → "new text"
  ```
- **Plain**: Disables colors (useful for scripts/CI)

**When to use diff:**
- After editing TOML files and before running `kg generate`
- To verify inheritance is working as expected
- To catch unintended changes from template modifications
- When debugging why an agent isn't getting expected config

**Pro tip:** Always use `--compact` when reviewing changes as an agent. The full structural output is verbose and harder to parse. Compact mode shows exactly what paths changed, making it trivial to verify the edit was correct.

**CAUTION:** `kg diff` is not 100% reliable and may produce empty output even when changes exist. This is a known limitation. If diff shows no changes but you expect some, do not dwell on debugging it. Instead:

1. Proceed with `kg validate` and `kg generate` commands as normal
2. Verify the generated agent files directly:
   - Global agents: `~/.kiro/agents/<agent-name>.json`
   - Local agents: `.kiro/agents/<agent-name>.json`
3. Read the generated JSON to confirm your TOML changes were applied

The diff command is a convenience tool for quick previews, not a source of truth. The actual generated JSON files are authoritative.

## Step 3: Editing Agent Configs

To modify an agent's configuration:

1. Run `kg tree <agent-name>` to find which TOML files define it
2. Read the `sources` array -- it tells you exactly which files to edit
3. Edit the TOML file(s) directly
4. Run `kg validate` to verify the change (add `--global` if sources are global and you're in a project directory)
5. Run `kg diff --compact` to preview what will change (add `--global` if needed)
6. Run `kg generate` to apply (add `--global` if sources are global and you're in a project directory)

## How kg Organizes Configuration

kg uses a two-layer system:

```
manifests/          # WHO exists and HOW they relate
  └── kg.toml       # Agent declarations + inheritance

agents/             # WHAT each agent does
  └── rust.toml     # Agent configuration
```

**Manifests** declare agents and relationships:
```toml
[agents]
default = { inherits = [] }
rust = { inherits = ["default"] }
```

**Agent files** define configuration:
```toml
# agents/rust.toml
description = "Rust development agent"
allowedTools = ["@rustdocs", "@cargo"]
```

Both layers exist at two locations:
- `~/.kiro/generators/` — Global (all projects)
- `.kiro/generators/` — Local (this project)

Local configs **merge with** global configs (not replace). This is what makes kg useful for real projects -- global tooling + local project context in one generated agent.

**Merge rules** when configs combine via inheritance or global+local:
- **Arrays** (allowedTools, resources): Combined
- **Objects** (toolsSettings, mcpServers): Deep merged
- **Scalars** (description, model): Child replaces parent

## Templates

Templates are the core of what makes kg worth using. They're not about saving keystrokes -- they're about making your agent fleet **deterministic**.

When every agent inherits from shared templates, you get a guarantee: change the git deny list in one place, every agent changes. Add a hook to the Rust template, every Rust agent gets it. No gaps, no drift, no "I forgot to update that one agent."

### The Problem Without Templates

Here's a real project (opensearch-feature-explorer) with 15 agents. Here are four of them:

```json
// create-issues.json
{
  "name": "create-issues",
  "description": "Create individual investigation Issues from a tracking Issue",
  "tools": ["@builtin", "@github"],
  "allowedTools": ["@builtin", "@github"],
  "resources": [
    "file://README.md",
    "file://.kiro/steering/**/*.md",
    "file://.kiro/agents/prompts/create-issues.md"
  ],
  "mcpServers": {
    "github": {
      "command": "bash",
      "args": ["-c", "GITHUB_PERSONAL_ACCESS_TOKEN=$(gh auth token) npx -y @modelcontextprotocol/server-github"]
    }
  },
  "model": "claude-opus-4.6"
}
```

```json
// investigate.json
{
  "name": "investigate",
  "description": "Investigate a single feature and create/update feature reports",
  "tools": ["@builtin", "@github"],
  "allowedTools": ["@builtin", "@github"],
  "resources": [
    "file://DEVELOPMENT.md",
    "file://README.md",
    "file://.kiro/steering/**/*.md",
    "file://.kiro/agents/prompts/investigate.md",
    "skill://.kiro/skills/opensearch-docs-search/SKILL.md"
  ],
  "mcpServers": {
    "github": {
      "command": "bash",
      "args": ["-c", "GITHUB_PERSONAL_ACCESS_TOKEN=$(gh auth token) npx -y @modelcontextprotocol/server-github"]
    }
  },
  "model": "claude-opus-4.6"
}
```

```json
// planner.json
{
  "name": "planner",
  "description": "Analyze release notes and create GitHub Issues for investigation tasks",
  "tools": ["@builtin", "@github"],
  "allowedTools": ["@builtin", "@github"],
  "resources": [
    "file://README.md",
    "file://.kiro/steering/**/*.md",
    "file://.kiro/agents/prompts/planner.md",
    "skill://.kiro/skills/opensearch-docs-search/SKILL.md"
  ],
  "mcpServers": {
    "github": {
      "command": "bash",
      "args": ["-c", "GITHUB_PERSONAL_ACCESS_TOKEN=$(gh auth token) npx -y @modelcontextprotocol/server-github"]
    }
  },
  "model": "claude-opus-4.6"
}
```

```json
// translate.json
{
  "name": "translate",
  "description": "Translate existing reports to other languages",
  "tools": ["@builtin"],
  "allowedTools": ["@builtin"],
  "resources": [
    "file://DEVELOPMENT.md",
    "file://README.md",
    "file://.kiro/steering/**/*.md",
    "file://.kiro/agents/prompts/translate.md"
  ],
  "model": "claude-opus-4.6"
}
```

The pattern is obvious: every agent has the same `model`, the same `mcpServers.github` block (a 4-line JSON blob), the same base `resources`, and the same `tools`/`allowedTools`. The only real differences are `name`, `description`, `prompt`, and sometimes one extra resource or skill.

This is copy-pasted across all 15 agents. The problems:
- Change the MCP server config? Edit 15 files and hope you got them all.
- Pin a new model? 15 edits.
- One typo in one file? That agent silently behaves differently. No one notices until something breaks.
- No way to audit "do all my agents have the same base config?" without diffing 15 files by hand.

### How Templates Work

A template is an agent with `template = true`. It doesn't generate a JSON file -- it exists only to be inherited:

```toml
[agents.git]
template = true

[agents.default]
inherits = ["git"]
```

`git` provides configuration. `default` inherits it and generates a real agent JSON file. That's the entire mechanism.

Templates can inherit from other templates, forming chains:

```toml
[agents.git]
template = true

[agents.git-write]
template = true
inherits = ["git"]

[agents.dependabot]
inherits = ["git-write"]
```

`dependabot` gets everything from `git` (via `git-write`) plus whatever `git-write` adds. Configuration flows from root to leaf, merging at each step.

### Template Categories

Templates naturally fall into four categories. Most real setups use at least two.

#### 1. Permission Templates — "every agent gets the same shell boundaries"

Shell commands are the most dangerous capability an agent has. Unlike tool calls that go through Kiro's permission system, shell commands execute directly on the host. A careless `allow = [".*"]` is basically unrestricted shell access for an LLM. Permission templates define these boundaries once, and every agent that inherits them gets the same guarantees.

Here's a real git permission template:

```toml
# agents/templates/git.toml
[nativeTools.shell]
allow = [
  "git status .*",
  "git fetch .*",
  "git grep .*",
  "git diff .*",
  "git pull .*",
  "git branch .*",
  "git log .*",
  "git remote .*",
  "git ls-remote .*",
  "mygh pr diff .*",
  "mygh pr create .*",
  "mygh pr list .*",
  "mygh pr view .*",
  "mygh pr edit .*",
  "mygh pr comment .*",
  "mygh pr review .*",
  "mygh workflow .*",
  "mygh run watch .*",
  "mygh run list .*",
  "mygh run view .*",
  "mygh run cancel .*",
  "mygh run download .*",
  "gh pr diff .*",
  "gh pr create .*",
  "gh pr list .*",
  "gh pr view .*",
  "gh pr edit .*",
  "gh pr comment .*",
  "gh pr review .*",
  "gh run watch .*",
  "gh run list .*",
  "gh run view .*",
  "gh run cancel .*",
  "gh workflow .*",
  "gh run download .*"
]
deny = ["git commit .*", "git push .*", "git tag .*"]

[hooks.agentSpawn.gitstatus]
command = "hook-git-status"
```

And a CLI safety template:

```toml
# agents/templates/cli.toml
[nativeTools.shell]
allow = [
  "^kg .*",
  "^jq .*",
  "^yq .*",
  "^cd.*",
  "tail .*",
  "head .*",
  "grep .*",
  "^man .*",
  "^wc .*",
  "pdf2 .*",
  "^ps .*",
  "pgrep .*",
  "dig .+",
  "find .*",
  "^ls .*",
  "rg .*",
  "readlink.*",
  "notify-send.*",
  "logger.*"
]
deny = [
  ".*yarn install.*",
  ".*npx.*",
  ".*uvx.*",
  ".*bun install.*",
  ".*npm install.*",
  ".*delete.*",
  ".pulumi up.*",
  "\\srm .*",
  ".*srm.*",
  ".*destroy.*",
  ".*rollout.*",
  "\\A.*\\bkill\\b.*\\z",
  ".*patch.*"
]
```

Every agent that inherits `git` gets the same git boundaries -- read-only git operations allowed, commits/pushes/tags denied, plus a hook that runs `git status` on spawn. Every agent that inherits `cli` gets the same safe CLI tools with destructive operations denied. You audit two files, not twenty.

**Force properties** let templates guarantee permissions that children can't accidentally restrict:

```toml
[agents.git-write]
template = true
inherits = ["git"]

[agents.git-write.nativeTools.shell]
forceAllow = ["git add .*", "git commit .*", "git push .*"]
```

A `dependabot` agent inheriting `git-write` gets commit and push even if something else in its inheritance chain tries to deny them. The `force` prefix is an operational guarantee -- it means "this permission cannot be overridden by any child or sibling in the inheritance tree."

Force properties work for paths too:

```toml
[agents.cargo-editor]
template = true

[agents.cargo-editor.nativeTools.write]
forceAllow = [".*Cargo.toml.*"]

[agents.cargo-editor.nativeTools.read]
forceAllow = [".*Cargo.toml.*", ".*Cargo.lock.*"]
```

A child agent that tries to deny write access to `Cargo.toml` will be overridden. The forced paths are added to the allow list and removed from any deny list in the inheritance chain.

#### 2. Capability Templates — "every agent gets the same MCP servers and tool access"

MCP server blocks are verbose and identical across agents. Templates eliminate the duplication.

Here's the opensearch project rewritten with one template:

```toml
# manifests/kg.toml

# The base template -- everything shared across all 15 agents
[agents.opensearch-base]
template = true
model = "claude-opus-4.6"
tools = ["@builtin", "@github"]
allowedTools = ["@builtin", "@github"]
resources = ["file://README.md", "file://.kiro/steering/**/*.md"]

[agents.opensearch-base.mcpServers.github]
command = "bash"
args = ["-c", "GITHUB_PERSONAL_ACCESS_TOKEN=$(gh auth token) npx -y @modelcontextprotocol/server-github"]

# Each agent: 2-4 lines instead of 20
# Only what's unique to this agent -- everything else comes from opensearch-base

[agents.create-issues]
inherits = ["opensearch-base"]
description = "Create investigation Issues from a tracking Issue"
resources = ["file://.kiro/agents/prompts/create-issues.md"]

[agents.investigate]
inherits = ["opensearch-base"]
description = "Investigate a single feature and create/update feature reports"
resources = [
  "file://DEVELOPMENT.md",
  "file://.kiro/agents/prompts/investigate.md",
  "skill://.kiro/skills/opensearch-docs-search/SKILL.md"
]

[agents.planner]
inherits = ["opensearch-base"]
description = "Analyze release notes and create GitHub Issues for investigation tasks"
resources = [
  "file://.kiro/agents/prompts/planner.md",
  "skill://.kiro/skills/opensearch-docs-search/SKILL.md"
]

[agents.summarize]
inherits = ["opensearch-base"]
description = "Create release summary from feature reports"
resources = [
  "file://DEVELOPMENT.md",
  "file://.kiro/agents/prompts/summarize.md",
  "skill://.kiro/skills/opensearch-docs-search/SKILL.md"
]

[agents.translate]
inherits = ["opensearch-base"]
description = "Translate existing reports to other languages"
tools = ["@builtin"]
allowedTools = ["@builtin"]
resources = [
  "file://DEVELOPMENT.md",
  "file://.kiro/agents/prompts/translate.md"
]

[agents.create-release-groups]
inherits = ["opensearch-base"]
description = "Group raw release items into feature groups"
tools = ["@builtin"]
allowedTools = ["@builtin"]
resources = ["file://.kiro/agents/prompts/create-release-groups.md"]

[agents.refactor]
inherits = ["opensearch-base"]
description = "Batch structural changes to existing reports"
resources = [
  "file://DEVELOPMENT.md",
  "file://.kiro/agents/prompts/refactor.md"
]

# ... and so on for all 15 agents
```

Change the MCP server config once, all 15 agents update. Pin a new model once, all 15 agents update. Add a new shared resource once, all 15 agents get it. The `translate` and `create-release-groups` agents override `tools`/`allowedTools` to drop `@github` since they don't need it -- scalars replace, so the child's value wins.

Other real capability templates:

```toml
# agents/cloud/k8s.toml — Kubernetes access
[nativeTools.shell]
allow = ["kubectl .*", "kubectx .*", "helm .*"]
deny = [
  "kubectl annotate .*",
  "kubectl label .*",
  "kubectl scale .*",
  "kubectl patch .*",
  "kubectl cordon .*",
  "kubectl drain .*",
  "kubectl taint .*",
  "kubectl autoscale .*",
  "kubectl delete .*",
  "kubectl create .*",
  "kubectl rollout .*"
]
```

```toml
# agents/node.toml — Node/Yarn tooling
[mcpServers.bun]
disabled = true
url = "https://bun.com/docs/mcp"

[nativeTools.shell]
allow = [
  "yarn test .*",
  "yarn run test .*",
  "yarn run test",
  "yarn run build",
  "yarn run lint"
]
```

```toml
# agents/cloud/pulumi.toml — Infrastructure as Code
[nativeTools.shell]
allow = ["pulumi preview .*", "pulumi stack .*", "pulumi config .*"]
deny = ["pulumi up .*", "pulumi destroy .*"]

[mcpServers.pulumi]
command = "pulumi-mcp"
timeout = 120000
```

#### 3. Context Templates — "every agent sees the same project docs and knowledge bases"

Context templates ensure every agent has access to the same documentation, steering files, skills, and knowledge bases. Without them, you'd copy-paste resource lists across every agent and inevitably miss one.

Here's a real context template:

```toml
# manifests/base.toml
[agents.resources]
template = true
resources = [
  "file://AGENTS.md",
  "file://README.md",
  "file://~/.config/agents/resources/me.md",
  "file://.kiro/steering/**/*.md",
  "skill://~/.config/agents/skills/**/SKILL.md"
]
```

Every agent that inherits `resources` gets the project README, the AGENTS.md conventions file, personal preferences, all steering docs, and all skills. Add a new steering doc? Every agent sees it automatically.

For team projects, context templates let everyone share project knowledge while keeping personal agent configs separate:

```toml
# .kiro/generators/manifests/kg.toml (checked into git)
[agents.project-docs]
template = true
resources = [
  "file://docs/**/*.md",
  "file://AGENTS.md",
  "file://README.md",
  "file://.kiro/steering/**/*.md"
]

[agents.project-docs.knowledge.api-docs]
source = "file://./api-docs"
description = "API reference documentation"
autoUpdate = true
indexType = "best"

[agents.project-docs.knowledge.architecture]
source = "file://./docs/architecture"
description = "System architecture and design decisions"
autoUpdate = true
indexType = "best"
```

Each developer creates their own manifest inheriting from `project-docs`:

```toml
# .kiro/generators/manifests/my-agents.toml (gitignored, personal)
[agents.rust]
inherits = ["project-docs"]

[agents.frontend]
inherits = ["project-docs"]
```

Same project context, different personal tooling. The gitignore setup:

```gitignore
# Ignore personal agent manifests
.kiro/generators/manifests/*.toml
# Keep shared project resources
!.kiro/generators/manifests/kg.toml

# Ignore generated agent files
.kiro/agents/
```

This pattern also works for augmenting global agents with local project knowledge. Global provides tooling, local adds project-specific context:

```toml
# Global: ~/.kiro/generators/agents/rust.toml
description = "General Rust agent"
resources = ["file://~/.config/agents/resources/rust.md"]

[nativeTools.shell]
allow = ["cargo .*"]
deny = ["cargo publish .*"]
```

```toml
# Local: .kiro/generators/agents/rust.toml
[knowledge.facet]
source = "file://./facet-docs"
description = "Information about the Rust crates facet-json, facet-toml, facet-diff and other facet libraries"
autoUpdate = true
indexType = "best"
```

The generated `rust.json` gets both -- global cargo tooling plus local facet knowledge. No duplication, no override.

#### 4. Lifecycle Templates — "every agent runs the same checks at the same time"

Hooks define actions that run at specific points in an agent's lifecycle. Language templates are the natural home for these -- you want every Rust agent to run `cargo check` on spawn, every TypeScript agent to run the type checker, every Go agent to run `go vet`. Define it once in the language template, never think about it again.

```toml
# agents/templates/rust-lang.toml
[hooks.agentSpawn.cargocheck]
command = "cargo check --quiet"

[hooks.agentSpawn.clippycheck]
command = "cargo clippy --quiet -- -D warnings"

[nativeTools.shell]
allow = [
  "cargo .*",
  "rustup .*"
]
deny = ["cargo publish .*"]
```

```toml
# agents/templates/typescript-lang.toml
[hooks.agentSpawn.typecheck]
command = "yarn run tsc --noEmit"

[hooks.agentSpawn.lintcheck]
command = "yarn run lint --quiet"

[nativeTools.shell]
allow = [
  "yarn test .*",
  "yarn run build",
  "yarn run lint",
  "yarn run tsc .*"
]
```

```toml
# agents/templates/go-lang.toml
[hooks.agentSpawn.govet]
command = "go vet ./..."

[nativeTools.shell]
allow = [
  "go build .*",
  "go test .*",
  "go vet .*",
  "go mod .*"
]
deny = ["go install .*"]
```

Every Rust agent runs `cargo check` and `clippy` on spawn. Every TypeScript agent runs the type checker and linter. Every Go agent runs `go vet`. The agent starts with awareness of the project's current state -- compile errors, lint warnings, type issues -- without the user having to ask.

Notice that lifecycle templates naturally combine with capability templates. The `rust-lang` template provides both hooks (lifecycle) and shell permissions (capability). A single template can span multiple categories.

### Composing Templates

The real power is composition. Here's a real manifest that combines all four categories:

```toml
# manifests/base.toml
[agents]
git = { template = true }          # permission: git boundaries
cli = { template = true }          # permission: safe CLI tools
resources = { template = true }    # context: shared docs + skills
k8s = { template = true }          # capability: kubernetes access
node = { template = true }         # capability + lifecycle: node tooling

default = { inherits = ["git", "resources", "cli"] }
rust = { inherits = ["default"] }
corsa = { inherits = ["default", "k8s", "node"] }
```

`default` inherits from three templates: `git` (shell boundaries for git/gh commands), `resources` (shared docs and skills), and `cli` (safe CLI tools with destructive ops denied). Every agent that inherits `default` gets all three.

`corsa` inherits from `default` (which brings git + resources + cli), plus `k8s` (kubernetes access with destructive kubectl ops denied) and `node` (yarn tooling + bun MCP server). The final generated agent has:
- Git commands allowed (read-only), commits/pushes denied — from `git`
- Safe CLI tools (jq, grep, find, etc.), npm/yarn install denied — from `cli`
- Project docs, personal preferences, all skills — from `resources`
- kubectl/helm allowed, kubectl delete/scale/drain denied — from `k8s`
- yarn test/build/lint allowed, bun MCP server configured — from `node`
- Plus whatever `corsa` itself adds (its own resources, MCP servers, etc.)

Change the git deny list in `git.toml` and every agent in the fleet updates — `default`, `rust`, `corsa`, and any future agents that inherit from them.

### Subagent Templates

Subagents deserve their own base template -- typically with a cheaper model and tighter restrictions than primary agents:

```toml
# manifests/subagents.toml
[agents.subagent-default]
template = true
model = "claude-haiku-4.5"
inherits = ["git", "cli"]
tools = ["*"]

[agents.gh-workflow]
description = "GitHub Workflow monitor"
inherits = ["subagent-default"]
resources = ["file://~/.config/agents/resources/gh-workflow.md"]

[agents.jina]
description = "HTML to markdown converter"
inherits = ["subagent-default"]
```

```toml
# agents/subagent-default.toml
resources = [
  "file://~/.config/agents/resources/me.md",
  "file://~/.config/agents/resources/subagents.md",
  "skill://~/.config/agents/skills/**/SKILL.md"
]
allowedTools = [
  "read", "shell", "write", "knowledge",
  "introspect", "todo_list", "web_fetch", "web_search"
]

[nativeTools.write]
allow = ["~/.config/agents/scratchpad/**"]

[nativeTools.shell]
deny = [
  ".*delete.*", "pulumi up.*",
  ".*destroy.*", ".*rollout.*",
  ".*kill.*", ".*patch.*"
]
```

All subagents get the same security baseline: cheaper model, restricted write paths, shared deny list. The `gh-workflow` subagent adds its own resources, `jina` adds its own MCP server. Neither needs to duplicate the base config.

## Configuration Resolution Order

When resolving agent `rust`, kg searches (lowest to highest precedence):

1. `~/.kiro/generators/manifests/*.toml` - Global declarations
2. `~/.kiro/generators/agents/rust.toml` - Global config
3. `.kiro/generators/manifests/*.toml` - Local declarations
4. `.kiro/generators/agents/rust.toml` - Local config

All found configs merge together. Use `kg tree rust` to see which sources apply.

## Error States

- **No agents found**: `kg tree` returns empty JSON object `{}` (exit 0). Consumers should check for an empty object.
- **Named agent not found**: `kg tree nonexistent` returns empty JSON object `{}` (exit 0). The requested agent key will be absent from the response.
- **Invalid TOML**: `kg validate` reports parse errors with file path and line number

## Bootstrap: Migrating from JSON Agents

When a user runs `kg bootstrap`, kg scans their existing `~/.kiro/agents/*.json` files (global agents only) and produces `~/.kiro/skills/kg-helper/references/analysis.json`. This file is your primary input for recommending a kg configuration.

**Important:** Before generating any TOML, tell the user to back up their existing agent configs:
```bash
mv ~/.kiro/agents ~/.kiro/agents.bak
```
`kg generate` will overwrite `~/.kiro/agents/*.json`. A clean break avoids mixing hand-written JSON with generated output.

### analysis.json Schema

```json
{
  "agents": {
    "<agent-name>": {
      "source": "global|local",
      "path": "/absolute/path/to/agent.json",
      "name": "agent-name",
      "description": "Human-readable description of the agent",
      "tools": ["*"],
      "allowedTools": ["read", "grep", "@context7/*"],
      "mcpServers": {
        "context7": { "command": "context7-mcp", "args": [], "timeout": 120000 }
      },
      "toolsSettings": {
        "shell": {
          "allowedCommands": ["git status", "cargo .*"],
          "deniedCommands": ["rm -rf .*"],
          "autoAllowReadonly": true
        }
      },
      "resources": ["file://README.md", "file://AGENTS.md"],
      "knowledge": {},
      "nativeTools": {},
      "hooks": {}
    }
  },
  "overlap": {
    "allowedTools": {
      "read": ["agent-a", "agent-b", "agent-c"],
      "grep": ["agent-a", "agent-b"]
    },
    "mcpServers": {
      "context7": ["agent-a", "agent-c"]
    },
    "shellCommands": {
      "git status": ["agent-a", "agent-b", "agent-c"]
    },
    "resources": {
      "file://README.md": ["agent-a", "agent-b"]
    }
  },
  "summary": {
    "total_agents": 5,
    "total_mcp_servers": 3,
    "agents_with_shell_settings": 4,
    "agents_with_knowledge": 1
  }
}
```

### Interview the User

Before generating any TOML, ask these three questions in order:

**1. Security posture** -- "How much do you trust your tools?"

- **Permissive**: `allowedTools = ["*"]` -- no prompts, full speed. Good for personal projects.
- **Moderate**: `tools = ["*"]` but curated `allowedTools` list -- tools available but gated on dangerous ones. Most common in the wild.
- **Strict**: Explicit `tools` list, locked-down shell, deny-by-default. For teams or sensitive environments.

If analysis.json shows most agents already use curated allowedTools (not `"*"`), default to **Moderate**.
If most agents use `allowedTools = ["*"]`, default to **Permissive**.

After the user picks a posture, recommend a default deny list for shell commands.
Every posture gets a base deny list. Stricter postures add more:

**Base (all postures):**
```toml
deniedCommands = ["rm -rf .*", "chmod -R 777 .*"]
```

**Moderate adds:**
```toml
deniedCommands = ["rm -rf .*", "chmod -R 777 .*", "git push .*", "git commit .*", "cargo publish .*", "npm publish .*", "docker push .*"]
```

**Strict adds:**
```toml
deniedCommands = ["rm -rf .*", "chmod -R 777 .*", "git push .*", "git commit .*", "cargo publish .*", "npm publish .*", "docker push .*", "kubectl delete .*", "terraform destroy.*", "aws .* delete.*"]
```

Present the deny list to the user and ask if they want to adjust it before proceeding. Some users have project-specific dangerous commands (like `terraform apply`, `kubectl delete`, or deployment scripts) that should be added to the deny list.

**2. Structure preference** -- "How do you want to organize your agents?"

- **Flat**: Independent agents, no inheritance. Best for 1-3 agents with little overlap.
- **Layered**: One `default` base agent, others inherit from it. Best for 3-6 agents with shared config.
- **Composable**: Templates for git, MCP servers, permissions -- agents composed from building blocks. Best for 6+ agents or teams.

Use the `overlap` section to guide the recommendation:
- If overlap shows 3+ agents sharing the same tools/commands, suggest **Layered** or **Composable**.
- If overlap is minimal, suggest **Flat**.
- If there are clear permission tiers (readonly vs write vs admin), suggest **Composable**.

**3. Scope** -- "Global, local, or both?"

- **Global only**: Personal tooling across all projects (`~/.kiro/generators/`)
- **Local only**: This project only (`.kiro/generators/`)
- **Both**: Global base + local project augmentation -- the kg sweet spot

If agents come from `~/.kiro/agents/`, suggest **Global** or **Both**.
If agents come from `.kiro/agents/`, suggest **Local** or **Both**.

### Decision Framework

After the interview, map answers + analysis to concrete TOML:

1. **Identify the base set**: Look at `overlap.allowedTools` and `overlap.shellCommands`. Fields appearing in >50% of agents belong in a `default` agent or template.

2. **Identify template candidates**: Look for clusters in `overlap.mcpServers`. If 3+ agents share the same MCP server, that's a template (e.g., `aws-base`, `frontend-tools`).

3. **Identify permission tiers**: Look at `overlap.shellCommands`. If some agents have read-only git and others have full git, that's a `git-readonly` / `git-write` template hierarchy.

4. **Map remaining config**: Whatever isn't shared goes into individual agent files.

5. **Don't over-engineer**: If there are only 1-2 agents, skip templates entirely. A flat structure with no inheritance is fine. Don't build a 3-layer hierarchy for two agents.

### Example Layouts

#### Flat (1-3 agents, minimal overlap)

```
~/.kiro/generators/
  manifests/kg.toml
  agents/
    rust.toml
    frontend.toml
```

```toml
# manifests/kg.toml
[agents]
rust = { inherits = [] }
frontend = { inherits = [] }
```

#### Layered (3-6 agents, shared base)

```
~/.kiro/generators/
  manifests/kg.toml
  agents/
    default.toml
    rust.toml
    frontend.toml
    devops.toml
```

```toml
# manifests/kg.toml
[agents]
default = { inherits = [] }
rust = { inherits = ["default"] }
frontend = { inherits = ["default"] }
devops = { inherits = ["default"] }
```

```toml
# agents/default.toml
description = "Base agent"
allowedTools = ["read", "grep", "glob", "web_search"]
resources = ["file://README.md"]

[toolsSettings.shell]
allowedCommands = ["git status", "git fetch", "git diff .*"]
autoAllowReadonly = true
```

#### Composable (6+ agents, clear groupings)

```
~/.kiro/generators/
  manifests/
    kg.toml
    subagents.toml
  agents/
    default.toml
    rust.toml
    frontend.toml
    templates/
      git.toml
      cli.toml
      resources.toml
```

Place template agent files in `agents/templates/` to keep them visually separate from real agents.

```toml
# manifests/kg.toml
[agents]
git = { template = true }
cli = { template = true }
resources = { template = true }
k8s = { template = true }

default = { inherits = ["git", "resources", "cli"] }
rust = { inherits = ["default"] }
frontend = { inherits = ["default"] }
devops = { inherits = ["default", "k8s"] }
```

### Generating the TOML Files

After the user confirms the layout:

1. Create `~/.kiro/generators/manifests/kg.toml` with the agent declarations and inheritance
2. Create individual `~/.kiro/generators/agents/<name>.toml` files with agent-specific config
3. For each agent, only include config that ISN'T inherited from parents
4. Run `kg validate` to verify the merged output matches the original JSON agents
5. Run `kg generate` to produce the new JSON files
6. Suggest the user diff the generated JSON against their original `.kiro/agents/*.json` to verify correctness

### Edge Cases

- **No existing agents**: User is starting fresh. Skip analysis, ask what kind of project they work on, and scaffold a minimal setup based on their answers.
- **Single agent**: Don't suggest inheritance. Create a flat config with one manifest entry and one agent file.
- **Massive duplication** (20+ agents with identical allowedTools): Strongly recommend Composable layout. Point out the duplication explicitly.
- **Mixed global and local agents**: Suggest the Both scope. Global for shared tooling, local for project-specific knowledge/resources.

## Keeping This Skill Updated

This skill document may be updated as kg evolves. To get the latest version:

```bash
curl -o ~/.kiro/skills/kg-helper/SKILL.md https://kiro-generator.io/SKILL.md
```

Check for updates periodically or when kg releases a new version. The skill document version is tracked in the metadata header.
