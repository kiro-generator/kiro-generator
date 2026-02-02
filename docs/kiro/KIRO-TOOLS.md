Title: Built-in tools - CLI - Docs - Kiro

URL Source: https://kiro.dev/docs/cli/reference/built-in-tools/
Scraped: 2026-02-01T11:24:35.399Z

---

## Select your cookie preferences

We use essential cookies and similar tools that are necessary to provide our site and services. We use performance cookies to collect anonymous statistics, so we can understand how customers use our site and make improvements. Essential cookies cannot be deactivated, but you can choose “Customize” or “Decline” to decline performance cookies.

If you agree, AWS and approved third parties will also use cookies to provide useful site features, remember your preferences, and display relevant content, including relevant advertising. To accept or decline all non-essential cookies, choose “Accept” or “Decline.” To make more detailed choices, choose “Customize.”

AcceptDeclineCustomize

## Customize cookie preferences

We use cookies and similar tools (collectively, "cookies") for the following purposes.

### Essential

Essential cookies are necessary to provide our site and services and cannot be deactivated. They are usually set in response to your actions on the site, such as setting your privacy preferences, signing in, or filling in forms.

### Performance

Performance cookies provide anonymous statistics about how customers navigate our site so we can improve site experience and performance. Approved third parties may perform analytics on our behalf, but they cannot use the data for their own purposes.

Allowed

### Functional

Functional cookies help us provide useful site features, remember your preferences, and display relevant content. Approved third parties may set these cookies to provide certain site features. If you do not allow these cookies, then some or all of these services may not function properly.

Allowed

### Advertising

Advertising cookies may be set through our site by us or our advertising partners and help us deliver relevant marketing content. If you do not allow these cookies, you will experience less relevant advertising.

Allowed

Blocking some types of cookies may impact your experience of our sites. You may review and change your choices at any time by selecting Cookie preferences in the footer of this site. We and selected third-parties use cookies or similar technologies as specified in the [AWS Cookie Notice](https://aws.amazon.com/legal/cookies/).

CancelSave preferences

## Unable to save cookie preferences

We will only store essential cookies at this time, because we were unable to save your cookie preferences.

If you want to change your cookie preferences, try again later using the link in the AWS console footer, or contact support if the problem persists.

Dismiss

On this page

# Built-in tools

Kiro CLI includes a collection of built-in tools that enhance your terminal experience with AI-powered assistance for common development tasks.

## File read [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#file-read)

**Tool name**: `read`

**Description**: Reads files, folders and images

bash

```bash
> what dependencies does my application have

Reading file: src/snake/package.json, all lines (using tool: read)
 ✓ Successfully read 1417 bytes from src/snake/package.json
 - Completed in 0.86s

> Your application has:

Runtime Dependencies:
- typescript (^3.5.1) - TypeScript compiler
- gh-pages (^2.0.1) - GitHub Pages deployment

Dev Dependencies:
- sass (^1.20.3) - CSS preprocessing
- eslint (^5.16.0) + TypeScript plugins - Code linting
- jest (^29.5.0) + ts-jest - Testing framework

No external game libraries or frameworks.
```

You can also configure the `read` tool in the agent configuration to have specific access to the files and folders in your project, giving you granular control over what the Kiro can read.

json

```json
{
  "toolsSettings": {
    "read": {
      "allowedPaths": ["~/projects", "./src/**"],
      "deniedPaths": ["d1/denied/path/", "d2/denied/path/**/file.txt"]
    }
  }
}
```

### Configuration options [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#configuration-options)

| Option | Type | Required | Description |
| --- | --- | --- | --- |
| `allowedPaths` | array of paths | No | Paths that can read without prompting. |
| `deniedPaths` | array of paths | No | Paths that are denied. |

Path values can be glob patterns similar to the behavior as gitignore. For example, `~/temp` would match `~/temp/child` and `~/temp/child/grandchild`

## Glob [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#glob)

**Tool name**: `glob`

**Description**: Fast file discovery using glob patterns. Respects `.gitignore`. Prefer this over `find` command in bash.

bash

```bash
> find all TypeScript test files in my project

Finding files matching pattern: **/*.test.ts (using tool: glob)

 ✓ Found 8 files
 - Completed in 0.12s

> Here are the TypeScript test files:

src/utils/helpers.test.ts
src/utils/validators.test.ts
src/components/button.test.ts
src/components/header.test.ts
src/services/auth.test.ts
src/services/api.test.ts
src/hooks/useAuth.test.ts
src/hooks/useForm.test.ts
```

You can configure the `glob` tool in the agent configuration to control which paths can be searched.

json

```json
{
  "toolsSettings": {
    "glob": {
      "allowedPaths": ["~/projects", "./src/**"],
      "deniedPaths": ["/etc", "/var"],
      "allowReadOnly": true
    }
  }
}
```

### Configuration options [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#configuration-options-1)

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| `allowedPaths` | array of strings | `[]` | List of paths that can be searched without prompting. Supports glob patterns |
| `deniedPaths` | array of strings | `[]` | List of paths that are denied. Deny rules are evaluated before allow rules |
| `allowReadOnly` | boolean | `false` | Whether to allow searches anywhere without prompting |

## Grep [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#grep)

**Tool name**: `grep`

**Description**: Fast content search using regex. Respects `.gitignore`. Use this instead of `grep`, `rg`, or `ag` commands in bash.

bash

```bash
> find all TODO comments in my project

Searching for pattern: TODO (using tool: grep)

 ✓ Found 12 matches across 5 files
 - Completed in 0.34s

> Here are the TODO comments I found:

src/utils/helpers.ts:23
  // TODO: Add error handling for edge cases

src/components/header.tsx:45
  {/* TODO: Implement mobile navigation */}

src/api/routes.ts:12
  // TODO: Add rate limiting
  // TODO: Implement caching

src/services/auth.ts:78
  // TODO: Add refresh token logic

...
```

You can configure the `grep` tool in the agent configuration to control which paths can be searched.

json

```json
{
  "toolsSettings": {
    "grep": {
      "allowedPaths": ["~/projects", "./src/**"],
      "deniedPaths": ["/etc", "/var"],
      "allowReadOnly": true
    }
  }
}
```

### Configuration options [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#configuration-options-2)

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| `allowedPaths` | array of strings | `[]` | List of paths that can be searched without prompting. Supports glob patterns |
| `deniedPaths` | array of strings | `[]` | List of paths that are denied. Deny rules are evaluated before allow rules |
| `allowReadOnly` | boolean | `false` | Whether to allow searches anywhere without prompting |

## File write [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#file-write)

**Tool name**: `write`

**Description**: Tool for creating and editing files

bash

```bash
> collect best practices for writing typescript code and save it to the kiro steering folder as codingpractices.md

I'll create the following file: src/snake/.kiro/steering/codingpractices.md (using tool: write)
Purpose: Create TypeScript coding best practices document

+     1: # TypeScript Coding Best Practices
+     2:
+     3: ## Type Safety
+     4: - Enable strict mode in `tsconfig.json` (`strict: true`)
+     5: - Avoid `any` type - use `unknown` for truly dynamic types
+     6: - Use explicit return types for functions
+     7: - Leverage type inference where it improves readability
+     8: - Use union types instead of enums when possible
+     9: - Prefer `interface` for object shapes, `type` for unions/intersections
+    10:
+    11: ## Code Organization
+    12: - One class/interface per file (exceptions for small related types)
+    13: - Group related functionality into modules
+    14: - Use barrel exports (`index.ts`) for clean imports
+    15: - Keep functions small and focused (single responsibility)
+    16: - Separate business logic from presentation logic
```

You can also configure the `write` tool in the agent configuration to have specific access to the files and folders in your project, giving you granular control over what the Kiro can write to.

json

```json
{
  "toolsSettings": {
    "write": {
      "allowedPaths": ["~/projects/output.txt", "./src/**"],
      "deniedPaths": ["/d1/denied/path/", "/d2/denied/path/**/file.txt"]
    }
  }
}
```

### Custom diff tools [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#custom-diff-tools)

By default, the write tool displays code changes using a built-in inline diff. You can configure an external diff tool to view changes instead. See [Custom diff tools](https://kiro.dev/docs/cli/chat/diff-tools) for setup instructions and supported tools.

### Configuration options [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#configuration-options-3)

| Option | Type | Required | Description |
| --- | --- | --- | --- |
| `allowedPaths` | array of paths | No | Paths that can be written to without prompting. |
| `deniedPaths` | array of paths | No | Paths that are denied. |

Path values can be glob patterns similar to the behavior as gitignore. For example, `~/temp` would match `~/temp/child` and `~/temp/child/grandchild`

## Execute shell commands [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#execute-shell-commands)

**Tool name**: `shell`

**Description**: Tool for executing a specified bash command.

You can also configure the `shell` tool in the agent configuration to control what commands Kiro can execute.

json

```json
{
  "toolsSettings": {
    "shell": {
      "allowedCommands": ["git status", "git fetch"],
      "deniedCommands": ["git commit .*", "git push .*"],
      "autoAllowReadonly": true
    }
  }
}
```

### Configuration Options [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#configuration-options-4)

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| allowedCommands | array of strings | \[\] | List of commands that are allowed without prompting |
| deniedCommands | array of strings | \[\] | List of commands that are denied. Deny rules are evaluated before allow rules |
| autoAllowReadonly | boolean | false | When enabled, read-only commands are allowed without prompting. This parameter does not restrict write actions. |
| denyByDefault | boolean | false | When true, deny any command outside allowedCommands and not auto-approved by `autoAllowReadonly`, instead of prompting for approval |

**Note**: `allowedCommands` and `deniedCommands` support regex formatting. regex entered are anchored with \\A and \\z, and does NOT support look-around, including look-ahead and look-behind.

## Execute AWS commands [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#execute-aws-commands)

**Tool name**: `aws`

**Description**: Tool for making AWS CLI calls with the specified service, operation, and parameters

bash

```bash
 list my s3 buckets

Running aws cli command (using tool: aws):

Service name: s3api
Operation name: list-buckets
Parameters:
Region: us-east-1
Label: List all S3 buckets
```

You can also configure the `aws` tool in the agent configuration to only perform specific AWS operations.

json

```json
{
  "toolsSettings": {
    "aws": {
      "allowedServices": ["s3", "lambda", "ec2"],
      "deniedServices": ["eks", "rds"],
      "autoAllowReadonly": true
    }
  }
}
```

## Web search and fetch [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#web-search-and-fetch)

Web access capabilities enable Kiro agent to access current information from the internet in real-time. This feature enables you to get up-to-date answers about topics that may have changed since the model's training data was created. These tools have been designed to not reproduce meaningful chunks of text and it should not be able to access webpages behind paywalls, authentication, and similar access restrictions. Search results may vary over time as internet content changes. Some content may not be accessible through web search due to various restrictions or the nature of the content.

| Tool | Description |
| --- | --- |
| `web_search` | Tool for searching the web |
| `web_fetch` | Tool for fetching content from a URL |

bash

```bash

# Example 1
> What is the latest on EC2 instances?

> I'll search for the latest information on EC2 instances.
Searching the web for: AWS EC2 instances latest 2025 (using tool: web_search)

 ✓ Found 10 search results
 - Completed in 2.12s

> Here's what I found:

Recent EC2 Instance Launches (2025):

New Instance Families:
- **M8a instances** - General-purpose instances with 5th Gen AMD EPYC (Turin) processors running at 4.5GHz. They deliver up to 30% higher performance and 19% better price-performance versus M7a instances.

- **C8gd, M8gd, R8gd instances** - Graviton-based instances with up to 11.4 TB of local NVMe SSD storage, now generally available.

- **X8g instances** - Powered by AWS Graviton4 processors, delivering up to 60% better performance than X2gd instances. Available in US East (Ohio).

- **R8i and R8i-flex instances** - Intel-based memory-optimized instances offering up to 15% better price-performance and 2.5x more memory bandwidth compared to previous Intel generations. They deliver 20% better performance than R7i
instances.

- **I7i instances** - Storage-optimized instances with 5th Gen Intel Xeon processors, delivering up to 23% better compute performance and 10% better price-performance over I4i instances.

Expanded Availability:
- C7i-flex and M7i-flex now available in 12xlarge and 16xlarge sizes
- Various instances expanding to additional regions including Europe (Milan) and US West (N. California)

Key Trend: AWS continues focusing on performance improvements (20-30% gains), better price-performance (10-19% improvements), and expanding processor options across Intel, AMD, and Graviton architectures.

References:
[1] Document history for the Amazon EC2 Instance Types Guide - https://docs.aws.amazon.com/ec2/latest/instancetypes/doc-history.html
...
[10] Amazon EC2 C7i-flex and C7i instances are now available in 2 additional regions - https://aws.amazon.com/about-aws/whats-new/2025/06/amazon-ec2-c7i-flex-c7i-instances-additional-...
 ▸ Time: 19s

# Example 2
> What is the latest on t3 medium instance pricing.

Searching the web for: AWS EC2 t3.medium instance pricing 2025 (using tool: web_search)

 ✓ Found 10 search results
 - Completed in 1.596s

Fetching content from: https://aws.amazon.com/ec2/pricing/on-demand/ (searching for: t3.medium price cost) [mode: selective] (using tool: web_fetch)

 ✓ Fetched 11395 bytes (selective) from URL
 - Completed in 0.718s

> The page doesn't show specific pricing in the excerpt. Let me get the actual pricing information:
Fetching content from: https://instances.vantage.sh/aws/ec2/t3.medium (searching for: t3.medium $0.04) [mode: selective] (using tool: web_fetch)

 ✓ Fetched 3971 bytes (selective) from URL
 - Completed in 0.780s

> Here's what I found:

AWS EC2 t3.medium Instance Pricing (2025):

...
```

To optimize retrieving content from a URL and better manage your context window, `web_fetch` tool has "Selective" and "Truncated" modes.

bash

```bash
# Example - using truncated
>  https://kiro.dev/blog/introducing-kiro-cli/ --> Can you get some of this page for me?

Fetching content from: https://kiro.dev/blog/introducing-kiro-cli/ [mode: truncated] (using tool: web_fetch)

 ✓ Fetched 8051 bytes (truncated content) from URL
 - Completed in 0.521s

> I've fetched the beginning portion of the page. It's a blog post announcing Kiro CLI. Here's what this section covers:

Title: "Bring Kiro agents to your terminal with Kiro CLI" (November 17, 2025)

Main Points:
- Kiro CLI brings AI agents into your terminal so you don't lose context when debugging production servers or working in SSH sessions
- Built on Q Developer CLI technology with additions like social login, Haiku 4.5, and the Auto agent
- Supports custom agents tailored to specific workflows (backend, frontend, DevOps)
- Your .kiro folder configs work in both IDE and CLI

Installation:
bash
curl -fsSL https://cli.kiro.dev/install | bash

Key Features covered in this section:
1. Custom agents with pre-approved tools, persistent context, and controlled access
2. Smart context management (Agent Resources, Session Context, Knowledge Bases)
3. Multiple authentication options (GitHub, Google, AWS Builder ID, IAM Identity Center)
4. Full IDE integration - MCP servers and steering rules transfer seamlessly
5. Interactive chat, multimodal input, and MCP support

The page continues beyond what was fetched. Would you like me to get a specific section or search for particular information?
```

In this case, since no specific pointers on the type of content to retrieve was provided, the tool automatically only retrieves part of the page (up to 8KB) to manage the context window.

bash

```bash
# Example - using selective

> https://kiro.dev/blog/introducing-kiro-cli/ --> Can you get installation information from this page for me.

Fetching content from: https://kiro.dev/blog/introducing-kiro-cli/ (searching for: installation install getting started) [mode: selective] (using tool: web_fetch)

 ✓ Fetched 7909 bytes (selective) from URL
 - Completed in 0.434s

> Here's the installation information from the page:

## Installation

Kiro CLI is available for macOS and Linux. Installation is straightforward:

bash
curl -fsSL https://cli.kiro.dev/install | bash

## First Steps

1. Authenticate and start chatting: Sign in with your credentials

2. Explore commands: Get help anytime

The page also mentions at the end:

bash
curl -fsSL https://cli.kiro.dev/install | bash

That's the single command you need to install Kiro CLI on macOS or Linux systems.
```

In this case, since the prompt was specific, the tool only retrieves selected content from the page.

### Fetch modes [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#fetch-modes)

| Mode | Behavior | Use case |
| --- | --- | --- |
| `selective` (default) | Returns 10 sentences before/after search term matches; 20 sentences if no matches | Targeted extraction |
| `truncated` | First 8000 characters | Quick preview |
| `full` | Complete content (up to 10MB) | Comprehensive analysis |

### Configuration [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#configuration)

Configure URL-based permissions using `toolsSettings`:

json

```json
{
  "toolsSettings": {
    "web_fetch": {
      "trusted": [".*docs\\.aws\\.amazon\\.com.*", ".*github\\.com.*"],
      "blocked": [".*pastebin\\.com.*"]
    }
  }
}
```

| Option | Type | Description |
| --- | --- | --- |
| `trusted` | array of regex | URL patterns to auto-allow without prompting |
| `blocked` | array of regex | URL patterns to deny (takes precedence over trusted) |

**Pattern behavior:**

- Patterns are regex, automatically anchored with `^` and `$`
- `blocked` takes precedence over `trusted`
- Invalid regex in `blocked` denies all URLs (fail-safe)
- Invalid regex in `trusted` are skipped

### Limitations [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#limitations)

- **Size**: 10MB maximum per page fetch
- **Timeout**: 30 seconds per request
- **Redirects**: Maximum 10 redirects followed
- **Content type**: Only text/html pages supported
- **Retries**: 3 automatic retry attempts on failure

### Troubleshooting [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#troubleshooting)

| Issue | Cause | Solution |
| --- | --- | --- |
| Fetch failed | Page >10MB, timeout, too many redirects, or binary content | Try different URL or check page accessibility |
| Empty content | Search terms don't match page content | Use different search terms or `truncated`/`full` mode |
| URL blocked | URL matches `blocked` pattern | Remove pattern from `toolsSettings` |
| Tool requires approval | URL not in `trusted` patterns and `web_fetch` not in `allowedTools` | Add URL pattern to `trusted` or add `web_fetch` to `allowedTools` |

## Introspect Kiro CLI capabilities [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#introspect-kiro-cli-capabilities)

**Tool name**: `introspect`

**Description**: Provide information about Kiro CLI capabilities, features, commands, and documentation. This tool accesses Kiro CLI's built-in documentation and help content to answer questions about the CLI's functionality.

bash

```bash
> what can you do

Introspecting to get you the right information (using tool: introspect) - Completed in 0.68s

> I can help you with a wide range of tasks:

File Operations
- Read, write, and edit files
- List directories and search for patterns
- Process images

Command Execution
- Run bash commands on your system
- Execute scripts and build tools
- Manage processes
```

### Usage [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#usage)

The introspect tool is automatically used when you ask questions about Kiro CLI itself, such as:

- "What can you do?"
- "How do I save conversations?"
- "What commands are available?"
- "Do you have feature X?"

### Configuration [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#configuration-1)

To automatically enter [tangent mode](https://kiro.dev/docs/cli/experimental/tangent-mode) for introspect questions:

bash

```bash
kiro-cli settings introspect.tangentMode true
```

## Submit an issue or feature request [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#submit-an-issue-or-feature-request)

**Tool name**: `report`

**Description**: Opens the browser to a pre-filled GitHub issue template to report chat issues, bugs, or feature requests.

This tool has no configuration options.

## Knowledge tool (experimental) [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#knowledge-tool-experimental)

**Tool name**: `knowledge`

**Description**: Store and retrieve information in a knowledge base across chat sessions. Provides semantic search capabilities for files, directories, and text content.

This tool has no configuration options.

## Thinking tool (experimental) [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#thinking-tool-experimental)

**Tool name**: `thinking`

**Description**: An internal reasoning mechanism that improves the quality of complex tasks by breaking them down into atomic actions.

This tool has no configuration options.

## ToDo list tool (experimental) [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#todo-list-tool-experimental)

**Tool name**: `todo`

**Description**:

Create and manage ToDo lists for tracking multi-step tasks.

This tool has no configuration options.

## Subagent tool [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#subagent-tool)

**Tool name**: `use_subagent`

**Description**: Delegate complex tasks to specialized subagents that run in parallel with isolated context. Useful for breaking down multi-step tasks into parallel subtasks, preventing context window bloat, running independent research simultaneously, or delegating to different agent configurations.

**Features:**

- Spawn up to 4 subagents simultaneously for parallel task execution
- Each subagent operates with its own isolated context to prevent main conversation bloat
- Real-time visual indicator showing status of all running subagents
- Support for different agent configurations per subagent
- Automatic execution summary with tool usage and duration metrics

### Configuration [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#configuration-2)

The `use_subagent` tool itself has no configurable `toolsSettings`. However, subagents can use different agent configurations:

- **Default subagent**: Uses the built-in default agent configuration
- **Custom subagents**: Can reference custom agent configurations by name when delegating tasks

The subagent inherits its tool access, permissions, and behavior from whichever agent configuration it's assigned to use.

### Example workflow [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#example-workflow)

bash

```bash
> Research the top 3 JavaScript frameworks and compare their performance

# Main agent spawns 3 subagents:
# - Subagent 1: Research React performance metrics
# - Subagent 2: Research Vue.js performance metrics
# - Subagent 3: Research Angular performance metrics

# Each subagent:
# - Conducts independent research
# - Gathers relevant data
# - Calls summary tool with findings

# Main agent receives all summaries and synthesizes comparison
```

For details on how subagents work and best practices, see [Subagents](https://kiro.dev/docs/cli/chat/subagents).

## Using tool settings in agent configuration [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#using-tool-settings-in-agent-configuration)

Tool settings are specified in the `toolsSettings` section of the agent configuration file. Each tool's settings are specified using the tool's name as the key.

For MCP server tools, use the format `@server_name/tool_name` as the key:

json

```json
{
  "toolsSettings": {
    "write": {
      "allowedPaths": ["~/projects"]
    },
    "@git/git_status": {
      "git_user": "$GIT_USER"
    }
  }
}
```

## Tool permissions [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#tool-permissions)

Tools can be explicitly allowed in the `allowedTools` section of the agent configuration:

json

```json
{
  "allowedTools": [\
    "read",\
    "knowledge",\
    "@git/git_status"\
  ]
}
```

If a tool is not in the `allowedTools` list, the user will be prompted for permission when the tool is used unless an allowed `toolSettings` configuration is set.

Some tools have default permission behaviors:

- `report` is trusted by default
- `read`, `grep`, and `glob` are trusted in the current working directory
- `shell`, `write`, and `aws` prompt for permission by default, but can be configured to allow specific commands/paths/services

## Next steps [Copy link to this section](https://kiro.dev/docs/cli/reference/built-in-tools/\#next-steps)

- **[Agent Integration](https://kiro.dev/docs/command-line/agents)** \- Use tools with custom agents
- **[MCP Integration](https://kiro.dev/docs/mcp/cli-integration)** \- Connect external tools via MCP
- **[Settings](https://kiro.dev/docs/command-line/settings)** \- Configure tool preferences
- **[Troubleshooting](https://kiro.dev/docs/command-line/agents-troubleshooting)** \- Common tool issues

Page updated: January 16, 2026

[Slash commands](https://kiro.dev/docs/cli/reference/slash-commands/)

[Settings](https://kiro.dev/docs/cli/reference/settings/)

* * *