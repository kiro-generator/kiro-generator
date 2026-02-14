# Templates

Templates are the core of what makes kg worth using. They're not about saving keystrokes -- they're about making your agent fleet **deterministic**.

When every agent inherits from shared templates, you get a guarantee: change the git deny list in one place, every agent changes. Add a hook to the Rust template, every Rust agent gets it. No gaps, no drift, no "I forgot to update that one agent."

## The Problem Without Templates

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

## How Templates Work

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

## Template Categories

Templates naturally fall into four categories. Most real setups use at least two.

### 1. Permission Templates -- "every agent gets the same shell boundaries"

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

### 2. Capability Templates -- "every agent gets the same MCP servers and tool access"

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
# agents/cloud/k8s.toml -- Kubernetes access
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
# agents/node.toml -- Node/Yarn tooling
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
# agents/cloud/pulumi.toml -- Infrastructure as Code
[nativeTools.shell]
allow = ["pulumi preview .*", "pulumi stack .*", "pulumi config .*"]
deny = ["pulumi up .*", "pulumi destroy .*"]

[mcpServers.pulumi]
command = "pulumi-mcp"
timeout = 120000
```

### 3. Context Templates -- "every agent sees the same project docs and knowledge bases"

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

### 4. Lifecycle Templates -- "every agent runs the same checks at the same time"

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

## Composing Templates

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
- Git commands allowed (read-only), commits/pushes denied -- from `git`
- Safe CLI tools (jq, grep, find, etc.), npm/yarn install denied -- from `cli`
- Project docs, personal preferences, all skills -- from `resources`
- kubectl/helm allowed, kubectl delete/scale/drain denied -- from `k8s`
- yarn test/build/lint allowed, bun MCP server configured -- from `node`
- Plus whatever `corsa` itself adds (its own resources, MCP servers, etc.)

Change the git deny list in `git.toml` and every agent in the fleet updates -- `default`, `rust`, `corsa`, and any future agents that inherit from them.

## Subagent Templates

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
