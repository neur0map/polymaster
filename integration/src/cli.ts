#!/usr/bin/env node
/**
 * wwatcher-ai CLI — OpenClaw-compatible interface for whale alert research
 * 
 * Commands:
 *   status                     Health check: history file, alert count, providers, API keys
 *   alerts [options]           Query recent alerts with filters
 *   summary                    Aggregate stats: volume, top markets, whale counts
 *   search <query>             Search alerts by market title/outcome text
 *   fetch <market_title>       Fetch RapidAPI data for a market (crypto/sports/weather/news)
 *   perplexity <query>         Run a single Perplexity search
 *   research <market_title>    Full research: RapidAPI + 5 Perplexity searches + analysis
 */

import * as fs from "fs";
import { AlertStore } from "./watcher/alert-store.js";
import { ProviderRegistry } from "./providers/registry.js";
import { fetchAutoFromProvider } from "./providers/fetcher.js";
import { queryPerplexity, runResearchQueries, generateResearchQueries } from "./providers/perplexity.js";
import { loadEnv } from "./util/env.js";

interface CliOptions {
  limit?: number;
  platform?: string;
  alertType?: string;
  minValue?: number;
  since?: string;
  category?: string;
  queries?: number;
}

function parseArgs(args: string[]): { command: string; positional: string[]; options: CliOptions } {
  const command = args[0] || "help";
  const positional: string[] = [];
  const options: CliOptions = {};

  for (let i = 1; i < args.length; i++) {
    const arg = args[i];
    if (arg.startsWith("--")) {
      const [key, value] = arg.slice(2).split("=");
      switch (key) {
        case "limit":
          options.limit = parseInt(value, 10);
          break;
        case "platform":
          options.platform = value;
          break;
        case "alert-type":
        case "type":
          options.alertType = value;
          break;
        case "min-value":
        case "min":
          options.minValue = parseFloat(value);
          break;
        case "since":
          options.since = value;
          break;
        case "category":
        case "cat":
          options.category = value;
          break;
        case "queries":
        case "q":
          options.queries = parseInt(value, 10);
          break;
      }
    } else if (arg.startsWith("-")) {
      // Short flags
      const key = arg.slice(1);
      const value = args[++i];
      switch (key) {
        case "l":
          options.limit = parseInt(value, 10);
          break;
        case "p":
          options.platform = value;
          break;
        case "t":
          options.alertType = value;
          break;
        case "m":
          options.minValue = parseFloat(value);
          break;
        case "s":
          options.since = value;
          break;
        case "c":
          options.category = value;
          break;
        case "q":
          options.queries = parseInt(value, 10);
          break;
      }
    } else {
      positional.push(arg);
    }
  }

  return { command, positional, options };
}

function printHelp(): void {
  console.log(`
wwatcher-ai — Whale Alert Research CLI for OpenClaw

USAGE:
  wwatcher-ai <command> [options]

COMMANDS:
  status                     Health check: history file, alert count, providers, API keys
  alerts                     Query recent alerts with filters
  summary                    Aggregate stats: volume, top markets, whale counts
  search <query>             Search alerts by market title/outcome text
  fetch <market_title>       Fetch RapidAPI data for a market
  perplexity <query>         Run a single Perplexity search
  research <market_title>    Full research: RapidAPI + Perplexity searches + prediction

ALERT OPTIONS:
  --limit=N, -l N            Max alerts to return (default: 20)
  --platform=X, -p X         Filter by platform (polymarket, kalshi)
  --type=X, -t X             Filter by alert type (WHALE_ENTRY, WHALE_EXIT)
  --min=N, -m N              Minimum transaction value in USD
  --since=ISO, -s ISO        Only alerts after this timestamp

FETCH/RESEARCH OPTIONS:
  --category=X, -c X         Override category (weather, crypto, sports, news, politics)
  --queries=N, -q N          Number of Perplexity queries for research (default: 5)

EXAMPLES:
  wwatcher-ai status
  wwatcher-ai alerts --limit=10 --min=50000
  wwatcher-ai fetch "Bitcoin price above 100k"
  wwatcher-ai perplexity "What are the latest Bitcoin ETF inflows?"
  wwatcher-ai research "Bitcoin above 100k by March" --category=crypto
`);
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const { command, positional, options } = parseArgs(args);

  if (command === "help" || command === "--help" || command === "-h") {
    printHelp();
    return;
  }

  // Load environment
  const env = loadEnv();

  // Initialize store and registry
  const store = new AlertStore();
  store.loadFromFile(env.historyPath);

  const registry = new ProviderRegistry(env.providersConfigPath);

  switch (command) {
    case "status": {
      const historyExists = fs.existsSync(env.historyPath);
      let historySize = 0;
      if (historyExists) {
        historySize = fs.statSync(env.historyPath).size;
      }

      const providers = registry.list();
      const result = {
        status: "running",
        history_file: {
          path: env.historyPath,
          exists: historyExists,
          size_bytes: historySize,
        },
        alerts: {
          total_loaded: store.count,
          latest_alert_time: store.latestAlertTime,
        },
        providers: {
          count: providers.length,
          categories: registry.categories(),
          list: providers,
        },
        api_keys: {
          rapidapi: !!env.rapidApiKey,
          perplexity: !!env.perplexityApiKey,
        },
      };
      console.log(JSON.stringify(result, null, 2));
      break;
    }

    case "alerts": {
      const alerts = store.query({
        limit: options.limit || 20,
        platform: options.platform,
        alert_type: options.alertType,
        min_value: options.minValue,
        since: options.since,
      });

      const result = {
        count: alerts.length,
        filters: {
          limit: options.limit || 20,
          platform: options.platform,
          alert_type: options.alertType,
          min_value: options.minValue,
          since: options.since,
        },
        alerts: alerts.map((a) => ({
          platform: a.platform,
          alert_type: a.alert_type,
          action: a.action,
          value: a.value,
          price_percent: a.price_percent,
          market_title: a.market_title,
          outcome: a.outcome,
          timestamp: a.timestamp,
          wallet_id: a.wallet_id,
          wallet_activity: a.wallet_activity,
        })),
      };
      console.log(JSON.stringify(result, null, 2));
      break;
    }

    case "summary": {
      const summary = store.summarize();
      console.log(JSON.stringify(summary, null, 2));
      break;
    }

    case "search": {
      const query = positional.join(" ");
      if (!query) {
        console.error(JSON.stringify({ error: "Search query required. Usage: wwatcher-ai search <query>" }));
        process.exit(1);
      }

      const results = store.search(query, options.limit || 20);
      console.log(JSON.stringify({
        query,
        count: results.length,
        alerts: results,
      }, null, 2));
      break;
    }

    case "fetch": {
      const marketTitle = positional.join(" ");
      if (!marketTitle) {
        console.error(JSON.stringify({ error: "Market title required. Usage: wwatcher-ai fetch <market_title>" }));
        process.exit(1);
      }

      if (!env.rapidApiKey) {
        const matches = registry.match(marketTitle, options.category);
        console.log(JSON.stringify({
          error: "RAPIDAPI_KEY not configured",
          help: "Set RAPIDAPI_KEY in integration/.env",
          matched_providers: matches.map((m) => ({
            provider: m.provider.name,
            category: m.provider.category,
            matched_keywords: m.matchedKeywords,
          })),
        }, null, 2));
        process.exit(1);
      }

      const matches = registry.match(marketTitle, options.category);

      if (matches.length === 0) {
        console.log(JSON.stringify({
          market_title: marketTitle,
          message: "No matching providers found for this market title",
          available_providers: registry.list(),
        }, null, 2));
        break;
      }

      const providersToFetch = options.category
        ? matches
        : matches.filter((m) => !m.provider.match_all || matches.length === 1);

      const results = await Promise.all(
        providersToFetch.map((m) =>
          fetchAutoFromProvider(m.provider, marketTitle, env.rapidApiKey!)
        )
      );

      console.log(JSON.stringify({
        market_title: marketTitle,
        providers_matched: matches.map((m) => ({
          name: m.provider.name,
          category: m.provider.category,
          keywords_matched: m.matchedKeywords,
        })),
        results: results.map((r) => ({
          provider: r.provider,
          endpoint: r.endpoint,
          status: r.status,
          error: r.error,
          data: r.data,
        })),
      }, null, 2));
      break;
    }

    case "perplexity": {
      const query = positional.join(" ");
      if (!query) {
        console.error(JSON.stringify({ error: "Query required. Usage: wwatcher-ai perplexity <query>" }));
        process.exit(1);
      }

      if (!env.perplexityApiKey) {
        console.log(JSON.stringify({
          error: "PERPLEXITY_API_KEY not configured",
          help: "Set PERPLEXITY_API_KEY in integration/.env. Get your key at https://perplexity.ai/settings/api",
        }, null, 2));
        process.exit(1);
      }

      const result = await queryPerplexity(query, env.perplexityApiKey);
      console.log(JSON.stringify({
        query: result.query,
        answer: result.answer,
        citations: result.citations,
        error: result.error,
      }, null, 2));
      break;
    }

    case "research": {
      const marketTitle = positional.join(" ");
      if (!marketTitle) {
        console.error(JSON.stringify({ error: "Market title required. Usage: wwatcher-ai research <market_title>" }));
        process.exit(1);
      }

      const missingKeys: string[] = [];
      if (!env.rapidApiKey) missingKeys.push("RAPIDAPI_KEY");
      if (!env.perplexityApiKey) missingKeys.push("PERPLEXITY_API_KEY");

      if (missingKeys.length > 0) {
        console.log(JSON.stringify({
          error: `Missing API keys: ${missingKeys.join(", ")}`,
          help: "Full research requires both RAPIDAPI_KEY and PERPLEXITY_API_KEY in integration/.env",
        }, null, 2));
        process.exit(1);
      }

      // Step 1: Fetch RapidAPI data
      const matches = registry.match(marketTitle, options.category);
      let rapidApiResults: any[] = [];

      if (matches.length > 0) {
        const providersToFetch = options.category
          ? matches
          : matches.filter((m) => !m.provider.match_all || matches.length === 1);

        rapidApiResults = await Promise.all(
          providersToFetch.map((m) =>
            fetchAutoFromProvider(m.provider, marketTitle, env.rapidApiKey!)
          )
        );
      }

      // Step 2: Run Perplexity searches
      const numQueries = options.queries || 5;
      const queries = generateResearchQueries(marketTitle, options.category).slice(0, numQueries);
      const perplexityResults = await runResearchQueries(
        marketTitle,
        env.perplexityApiKey!,
        options.category,
        queries
      );

      // Step 3: Compile research report
      const report = {
        market_title: marketTitle,
        category: options.category || (matches[0]?.provider.category ?? "general"),
        timestamp: new Date().toISOString(),
        rapidapi_data: {
          providers_matched: matches.map((m) => ({
            name: m.provider.name,
            category: m.provider.category,
          })),
          results: rapidApiResults.map((r) => ({
            provider: r.provider,
            status: r.status,
            data: r.data,
            error: r.error,
          })),
        },
        perplexity_research: {
          queries_run: perplexityResults.queries.length,
          results: perplexityResults.results.map((r) => ({
            query: r.query,
            answer: r.answer,
            citations: r.citations,
            error: r.error,
          })),
        },
        research_summary: {
          data_sources: matches.length + perplexityResults.queries.length,
          rapidapi_providers: matches.length,
          perplexity_queries: perplexityResults.queries.length,
          successful_queries: perplexityResults.results.filter(r => !r.error).length,
        },
      };

      console.log(JSON.stringify(report, null, 2));
      break;
    }

    default:
      console.error(JSON.stringify({ error: `Unknown command: ${command}`, help: "Run 'wwatcher-ai help' for usage" }));
      process.exit(1);
  }
}

main().catch((err) => {
  console.error(JSON.stringify({ error: err instanceof Error ? err.message : String(err) }));
  process.exit(1);
});
