import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { scoreAlert, passesPreferences } from "./scoring/scorer.js";
import type { WhalertAlert, AlertPreferences } from "./util/types.js";

export function createServer(): McpServer {
  const server = new McpServer({
    name: "wwatcher",
    version: "2.0.0",
  });

  server.tool(
    "score_alert",
    "Score a whale alert. Returns { score, tier, factors }.",
    { alert: z.string().describe("Full alert JSON string") },
    async ({ alert }) => {
      const parsed: WhalertAlert = JSON.parse(alert);
      const result = scoreAlert(parsed);
      return { content: [{ type: "text", text: JSON.stringify(result) }] };
    },
  );

  server.tool(
    "check_preferences",
    "Check if an alert passes user preference filters. Returns { passes: boolean }.",
    {
      alert: z.string().describe("Full alert JSON string"),
      preferences: z.string().describe("Preferences JSON string"),
    },
    async ({ alert, preferences }) => {
      const parsedAlert: WhalertAlert = JSON.parse(alert);
      const parsedPrefs: AlertPreferences = JSON.parse(preferences);
      const passes = passesPreferences(parsedAlert, parsedPrefs);
      return { content: [{ type: "text", text: JSON.stringify({ passes }) }] };
    },
  );

  return server;
}
