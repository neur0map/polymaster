import * as fs from "fs";
import type { Provider, ProvidersConfig } from "../util/types.js";

/**
 * Loads providers.json and routes requests by matching keywords against
 * market titles / categories.
 */
export class ProviderRegistry {
  private providers: ProvidersConfig = {};

  constructor(configPath: string) {
    this.load(configPath);
  }

  private load(configPath: string): void {
    if (!fs.existsSync(configPath)) {
      console.error(`[ProviderRegistry] providers.json not found at ${configPath}`);
      return;
    }
    const raw = fs.readFileSync(configPath, "utf-8");
    this.providers = JSON.parse(raw) as ProvidersConfig;
  }

  /** Get all loaded providers */
  getAll(): ProvidersConfig {
    return this.providers;
  }

  /** Get a specific provider by key */
  get(key: string): Provider | undefined {
    return this.providers[key];
  }

  /** List provider names and categories */
  list(): Array<{ key: string; name: string; category: string }> {
    return Object.entries(this.providers).map(([key, p]) => ({
      key,
      name: p.name,
      category: p.category,
    }));
  }

  /**
   * Match a market title (or explicit category) to relevant providers.
   * Returns all matching providers sorted by keyword relevance.
   * When explicit category is provided, always includes that provider.
   */
  match(marketTitle: string, category?: string): Array<{ key: string; provider: Provider; matchedKeywords: string[] }> {
    const titleLower = marketTitle.toLowerCase();
    const results: Array<{ key: string; provider: Provider; matchedKeywords: string[] }> = [];

    for (const [key, provider] of Object.entries(this.providers)) {
      // If explicit category provided, only match that category
      if (category && provider.category !== category) continue;

      // match_all providers (like news) always match
      if (provider.match_all) {
        results.push({ key, provider, matchedKeywords: ["*"] });
        continue;
      }

      const matched = provider.keywords.filter((kw) =>
        titleLower.includes(kw.toLowerCase())
      );

      // If explicit category provided, include even without keyword matches
      if (category && provider.category === category) {
        results.push({ key, provider, matchedKeywords: matched.length > 0 ? matched : ["(category override)"] });
      } else if (matched.length > 0) {
        results.push({ key, provider, matchedKeywords: matched });
      }
    }

    // Sort by number of matched keywords (most relevant first)
    results.sort((a, b) => b.matchedKeywords.length - a.matchedKeywords.length);

    return results;
  }
}
