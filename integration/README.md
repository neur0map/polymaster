# wwatcher Integration

Scoring MCP server for wwatcher whale alerts. Two tools:
- **score_alert** — score an alert and get tier + factors
- **check_preferences** — filter alerts against user preferences

## Quick Start

```bash
cd integration
npm install
npm run build
```

No API keys required.

---

## MCP Server Setup

Add to your MCP client config:

```json
{
  "mcpServers": {
    "wwatcher": {
      "command": "node",
      "args": ["/absolute/path/to/integration/dist/index.js"]
    }
  }
}
```

### MCP Tools

| Tool | Input | Output |
|------|-------|--------|
| `score_alert` | `{ alert: "<json>" }` | `{ score, tier, factors }` |
| `check_preferences` | `{ alert: "<json>", preferences: "<json>" }` | `{ passes: boolean }` |

---

## Agent Instructions

See [`instructions.md`](../instructions.md) for scoring system, tier thresholds, and preference fields.
