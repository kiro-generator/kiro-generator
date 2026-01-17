# Kiro Agent Configuration Reference

**Official Documentation:** https://kiro.dev/docs/cli/custom-agents/configuration-reference/

This document summarizes the Kiro agent configuration format that `kiro-generator` generates.

## File Locations

- **Global agents:** `~/.kiro/agents/<agent-name>.json`
- **Local agents:** `.kiro/agents/<agent-name>.json`
- **Precedence:** Local overrides global

## Configuration Fields

### Core Fields

- **`name`** - Agent identifier (optional, derived from filename)
- **`description`** - Human-readable purpose
- **`prompt`** - High-level context (inline text or `file://` URI)
- **`model`** - Model ID (e.g., `"claude-sonnet-4"`)
- **`keyboardShortcut`** - Quick switch shortcut (e.g., `"ctrl+a"`)
- **`welcomeMessage`** - Message shown when switching to agent

### MCP Servers

```json
{
  "mcpServers": {
    "server-name": {
      "command": "command-to-run",
      "args": [],
      "env": {},
      "timeout": 120000
    }
  }
}
```

### Tools

```json
{
  "tools": [
    "read",              // Built-in tool
    "@server",           // All tools from MCP server
    "@server/tool_name", // Specific MCP tool
    "*"                  // All available tools
  ]
}
```

### AllowedTools (Security)

**Purpose:** Tools that can execute **without user permission prompts**.

**Pattern Support:**
```json
{
  "allowedTools": [
    // Exact matches
    "read",
    "@server/specific_tool",
    "@server",                    // All tools from server
    
    // Wildcards
    "r*",                         // Prefix match
    "*_get",                      // Suffix match
    "@server/read_*",             // MCP tool prefix
    "@server/*_info",             // MCP tool pattern
    "@git-*/status"               // Server pattern
  ]
}
```

**Rules:**
- `*` matches any sequence of characters
- `?` matches exactly one character
- Case-sensitive matching
- Exact matches take precedence
- **Does NOT support `"*"` wildcard for all tools** (unlike `tools` field)

### ToolAliases

Remap tool names to resolve collisions:

```json
{
  "toolAliases": {
    "@github-mcp/get_issues": "github_issues",
    "@gitlab-mcp/get_issues": "gitlab_issues"
  }
}
```

### ToolsSettings

Tool-specific configuration:

```json
{
  "toolsSettings": {
    "write": {
      "allowedPaths": ["src/**", "tests/**"]
    },
    "shell": {
      "allowedCommands": ["git status", "git fetch"],
      "deniedCommands": ["git commit .*", "git push .*"],
      "autoAllowReadonly": true
    }
  }
}
```

**Note:** Specifications in `toolsSettings` are overridden if the tool is in `allowedTools`.

### Resources

```json
{
  "resources": [
    // File resources (loaded at startup)
    "file://README.md",
    "file://docs/**/*.md",
    
    // Skill resources (metadata at startup, content on-demand)
    "skill://.kiro/skills/**/SKILL.md",
    
    // Knowledge base resources
    {
      "type": "knowledgeBase",
      "source": "file://./docs",
      "name": "ProjectDocs",
      "description": "Project documentation",
      "indexType": "best",      // or "fast"
      "autoUpdate": true
    }
  ]
}
```

**Skill Format:**
```markdown
---
name: skill-name
description: When to use this skill
---

# Skill Content
...
```

### Hooks

Commands run at specific trigger points:

```json
{
  "hooks": {
    "agentSpawn": [
      { "command": "git status" }
    ],
    "userPromptSubmit": [
      { "command": "ls -la" }
    ],
    "preToolUse": [
      {
        "matcher": "execute_bash",
        "command": "echo 'Audit log' >> /tmp/audit.log"
      }
    ],
    "postToolUse": [
      {
        "matcher": "fs_write",
        "command": "cargo fmt --all"
      }
    ],
    "stop": [
      { "command": "npm test" }
    ]
  }
}
```

**Hook Types:**
- `agentSpawn` - When agent is activated
- `userPromptSubmit` - When user submits a prompt
- `preToolUse` - Before tool execution (can block)
- `postToolUse` - After tool execution
- `stop` - When assistant finishes responding

**Note:** Hook matchers use internal tool names (`fs_read`, `fs_write`, `execute_bash`, `use_aws`).

### IncludeMcpJson

```json
{
  "includeMcpJson": true
}
```

When `true`, includes MCP servers from `~/.kiro/settings/mcp.json` and `.kiro/settings/mcp.json`.

## Complete Example

```json
{
  "name": "aws-rust-agent",
  "description": "Specialized agent for AWS and Rust development",
  "prompt": "file://./prompts/aws-rust-expert.md",
  "mcpServers": {
    "git": {
      "command": "git-mcp",
      "args": []
    }
  },
  "tools": [
    "read",
    "write",
    "shell",
    "@git"
  ],
  "allowedTools": [
    "read",
    "@git/git_status",
    "@git/git_*_info"
  ],
  "toolsSettings": {
    "write": {
      "allowedPaths": ["src/**", "Cargo.toml"]
    }
  },
  "resources": [
    "file://README.md",
    "file://docs/**/*.md"
  ],
  "hooks": {
    "postToolUse": [
      {
        "matcher": "fs_write",
        "command": "cargo fmt --all"
      }
    ]
  },
  "model": "claude-sonnet-4",
  "keyboardShortcut": "ctrl+r",
  "welcomeMessage": "Ready to help with AWS and Rust!"
}
```

## Security Best Practices

1. **Start restrictive** - Minimal tool access, expand as needed
2. **Use specific patterns** - Prefer `"@server/read_*"` over `"@server"`
3. **Configure toolsSettings** - Add path/command restrictions
4. **Test in safe environments** - Verify permissions before production use
5. **Document dangerous operations** - Use `resources` to add warnings

## Key Differences from kg Config

- **Format:** JSON (Kiro) vs TOML (kg)
- **Purpose:** Runtime agent config (Kiro) vs generator config (kg)
- **Inheritance:** Not supported in Kiro JSON (handled by kg during generation)
- **Templates:** Not a concept in Kiro JSON (kg-specific)
