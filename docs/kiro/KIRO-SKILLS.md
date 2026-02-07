# Agent Skills

## What are skills?

Skills are portable instruction packages that follow the open [Agent Skills](https://agentskills.io) standard. They bundle instructions, scripts, and templates into reusable packages that Kiro can activate when relevant to your task.

Kiro supports the Agent Skills standard, so you can import skills from the community or other compatible AI tools, and share your own skills across the ecosystem.

## How skills work

AI agents are increasingly capable, but they often lack the specific context needed for real work. Without knowledge of your team's deployment process, your company's code review standards, or your project's data analysis pipeline, agents guess and iterate — just like you would when learning something new.

Loading all this context upfront isn't practical either. Too much information overwhelms the agent, slowing responses and reducing quality.

Skills solve this with progressive disclosure:

1. **Discovery** - At startup, Kiro loads only the name and description of each skill
2. **Activation** - When your request matches a skill's description, Kiro loads the full instructions
3. **Execution** - Kiro follows the instructions, loading scripts or reference files only as needed

This keeps context focused while giving Kiro access to extensive specialized knowledge on demand.

## Using skills

Kiro automatically activates skills when your request matches a skill's description. View and manage skills in the **Agent Steering & Skills** section in the Kiro panel.

## Skill scope

Skills can be created with a workspace scope or a global scope.

### Workspace skills

Workspace skills reside in your project under `.kiro/skills/`, and apply only to that specific workspace. Use workspace skills for project-specific workflows like deployment procedures or team conventions.

### Global skills

Global skills reside in your home directory under `~/.kiro/skills/`, and are available across all workspaces. Use global skills for personal workflows you use regardless of project—like your code review process or documentation standards.

In case of conflicting names between global and workspace skills, Kiro will prioritize the workspace skill. This allows you to define global skills that generally apply to all your workspaces, while preserving the ability to override them for specific projects.

## Importing skills

1. Open **Agent Steering & Skills** section in the Kiro panel
2. Click **+** and select **Import a skill**
3. Choose your source:
   - **GitHub** - Import from a public repository URL
   - **Local folder** - Import from your filesystem

Imported skills are copied to your skills directory and work immediately.

## Creating a skill

A skill is a folder containing a `SKILL.md` file:

```
my-skill/
├── SKILL.md           # Required
├── scripts/           # Optional executable code
├── references/        # Optional documentation
└── assets/            # Optional templates
```

### SKILL.md format

```markdown
---
name: pr-review
description: Review pull requests for code quality, security issues, and test coverage. Use when reviewing PRs or preparing code for review.
---

## Review process

1. Check for security vulnerabilities
2. Verify error handling
3. Confirm test coverage
4. Review naming and structure
```

### Frontmatter fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Must match folder name. Lowercase, numbers, hyphens only (max 64 chars). |
| `description` | Yes | When to use this skill. Kiro matches this against your requests (max 1024 chars). |
| `license` | No | License name or reference to a bundled license file. |
| `compatibility` | No | Environment requirements (e.g., required tools, network access). |
| `metadata` | No | Additional key-value data like author or version. |

See the [full specification](https://agentskills.io/specification) for detailed field constraints.

## How skills differ from steering and powers

**Skills** are portable packages following an open standard. They load on-demand and can include scripts. Use for reusable workflows you want to share or import from others.

**Steering** is Kiro-specific context that shapes agent behavior. It supports `always`, `auto`, `fileMatch`, and `manual` modes. Use for project standards and conventions.

**Powers** bundle MCP tools with knowledge and workflows. They activate dynamically based on context. Use for integrations where you need both tools and guidance.

**Tip**: For MCP integrations, [powers](https://kiro.dev/docs/powers) are usually a better fit—they bundle tools with built-in guidance and activate automatically based on what you're working on.

## Best practices

**Write precise descriptions** - Kiro uses the description to decide when to activate. Include specific keywords: "Review pull requests for security and test coverage" beats "helps with code review."

**Keep SKILL.md focused** - Put detailed documentation in `references/` files. Kiro loads the full SKILL.md on activation.

**Use scripts for deterministic tasks** - Validation, file generation, and API calls work better as scripts than LLM-generated code.

**Choose the right scope** - Global for personal workflows (your review checklist), workspace for team procedures (project deployment).

## Related documentation

- [Steering](https://kiro.dev/docs/steering) - Project-specific context and standards
- [Powers](https://kiro.dev/docs/powers) - MCP integrations with bundled knowledge
- [Agent Skills specification](https://agentskills.io/specification) - Full format details