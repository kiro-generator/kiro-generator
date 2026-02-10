# Field Research: .kiro/agents in the Wild

Date: 2026-02-07

## Data Collection

### GitHub Search Commands

```bash
# Find repos with .kiro/agents JSON files containing allowedTools
gh api 'search/code?q=allowedTools+path:.kiro/agents+language:json&per_page=100' \
  --jq '.total_count, (.items[] | "\(.repository.full_name) :: \(.path)")'

# Clone all repos (shallow)
# Extract repo names from search results, then:
# git clone --depth 1 https://github.com/<owner>/<repo>
```

### Extraction Commands

```bash
# Extract key fields from all agent JSON files
find /path/to/repos -path '*/.kiro/agents/*.json' -not -name '*.example' | while read -r f; do
  echo "=== $f ==="
  jq -c '{name: .name, description: .description, tools_count: (.tools // [] | length), allowedTools: (.allowedTools // []), mcpServers: (.mcpServers // {} | keys), resources_count: (.resources // [] | length)}' "$f" 2>/dev/null || echo "PARSE_ERROR"
done

# Aggregate allowedTools
grep -v "^===" agent-summary.txt | grep -v "PARSE_ERROR" | jq -r '.allowedTools[]' | sort | uniq -c | sort -rn

# Aggregate mcpServers
grep -v "^===" agent-summary.txt | grep -v "PARSE_ERROR" | jq -r '.mcpServers[]' | sort | uniq -c | sort -rn

# Count agents using allowedTools = ["*"]
grep -v "^===" agent-summary.txt | grep -v "PARSE_ERROR" | jq -r 'select(.allowedTools == ["*"]) | .name' | wc -l
```

## Results

- **151** repos found via GitHub search
- **48** repos cloned
- **182** agent JSON files parsed
- **1** parse error

## allowedTools Frequency

| Count | Tool |
|------:|------|
| 93 | read |
| 54 | grep |
| 52 | glob |
| 48 | shell |
| 40 | write |
| 25 | fs_read |
| 24 | @chrome-devtools/* |
| 23 | web_search |
| 22 | web_fetch |
| 21 | @context7/* |
| 20 | list |
| 20 | @shadcn/* |
| 20 | @sequentialthinking/* |
| 20 | @next-devtools/* |
| 20 | @time/* |

Only **5** agents use `allowedTools = ["*"]` (full permissive).

## MCP Servers Frequency

| Count | Server |
|------:|--------|
| 20 | github |
| 15 | aws-knowledge-mcp-server |
| 10 | file-watcher |
| 9 | chrome-devtools |
| 8 | playwright |
| 8 | awslabs.aws-documentation-mcp-server |
| 6 | awslabs.cdk-mcp-server |
| 6 | Context7 |
| 5 | context7 |
| 5 | awslabs.aws-pricing-mcp-server |
| 5 | aws-mcp |
| 4 | awslabs.frontend-mcp-server |
| 4 | awslabs.core-mcp-server |
| 3 | strands |
| 3 | sequential-thinking |

## Key Observations

1. **Massive duplication** -- repos like `agent-bridge-kit` (20+ agents) have identical allowedTools arrays across all agents. Prime candidate for kg templates.

2. **Most users are NOT permissive** -- only 5/182 agents use `allowedTools = ["*"]`. People curate their tool lists.

3. **Common base set** -- `read`, `grep`, `glob`, `shell`, `write` appear in 25-50% of agents. Natural candidate for a `default` template.

4. **MCP server clusters** -- AWS users cluster around `awslabs.*` servers. Frontend devs cluster around `chrome-devtools`, `playwright`, `context7`. These are natural template groupings.

5. **name + description fields are rich** -- agent names and descriptions immediately reveal the user's domain (backend, frontend, devops, security, etc.). Low-cost, high-signal for bootstrap analysis.

## Notable Repos

- **agent-bridge-kit** -- 20+ agents, all with identical allowedTools. Poster child for kg templates.
- **livestockai** -- 10 agents covering full-stack + domain specialist. Good example of role-based agents.
- **textspace-mud** -- 20+ agents from "bmad" framework. Heavy agent orchestration pattern.
- **opensearch-feature-explorer** -- 15 agents for different workflow stages (create, review, summarize, translate).

