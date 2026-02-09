/**
 * Perplexity API client for deep web research
 * Used for gathering comprehensive market intelligence
 */

export interface PerplexityResponse {
  query: string;
  answer: string;
  citations: string[];
  error?: string;
}

export interface PerplexitySearchResult {
  queries: string[];
  results: PerplexityResponse[];
  summary?: string;
}

/**
 * Query Perplexity API for market research
 */
export async function queryPerplexity(
  query: string,
  apiKey: string,
  model: string = "llama-3.1-sonar-small-128k-online"
): Promise<PerplexityResponse> {
  try {
    const response = await fetch("https://api.perplexity.ai/chat/completions", {
      method: "POST",
      headers: {
        "Authorization": `Bearer ${apiKey}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        model,
        messages: [
          {
            role: "system",
            content: "You are a research assistant focused on prediction markets, financial analysis, and current events. Provide concise, factual answers with relevant data points. Focus on information that would help predict market outcomes."
          },
          {
            role: "user",
            content: query
          }
        ],
        max_tokens: 1024,
        temperature: 0.2,
        return_citations: true,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      return {
        query,
        answer: "",
        citations: [],
        error: `Perplexity API error (${response.status}): ${errorText}`,
      };
    }

    const data = await response.json();
    const answer = data.choices?.[0]?.message?.content || "";
    const citations = data.citations || [];

    return {
      query,
      answer,
      citations,
    };
  } catch (err) {
    return {
      query,
      answer: "",
      citations: [],
      error: err instanceof Error ? err.message : String(err),
    };
  }
}

/**
 * Generate research queries for a market/alert
 */
export function generateResearchQueries(
  marketTitle: string,
  category?: string
): string[] {
  const baseQueries = [
    `Latest news and developments: ${marketTitle}`,
    `Expert analysis and predictions: ${marketTitle}`,
    `Historical data and trends: ${marketTitle}`,
    `Risk factors and uncertainties: ${marketTitle}`,
    `Recent events affecting: ${marketTitle}`,
  ];

  // Add category-specific queries
  if (category === "crypto") {
    baseQueries.push(
      `${marketTitle} - technical analysis and price targets`,
      `${marketTitle} - whale activity and institutional interest`
    );
  } else if (category === "sports") {
    baseQueries.push(
      `${marketTitle} - injury reports and team news`,
      `${marketTitle} - betting odds movement and sharp money`
    );
  } else if (category === "weather") {
    baseQueries.push(
      `${marketTitle} - forecast models and confidence levels`,
      `${marketTitle} - historical weather patterns`
    );
  } else if (category === "politics") {
    baseQueries.push(
      `${marketTitle} - polling data and trends`,
      `${marketTitle} - key demographics and swing factors`
    );
  }

  return baseQueries.slice(0, 5); // Return top 5 queries
}

/**
 * Run multiple Perplexity searches for comprehensive research
 */
export async function runResearchQueries(
  marketTitle: string,
  apiKey: string,
  category?: string,
  customQueries?: string[]
): Promise<PerplexitySearchResult> {
  const queries = customQueries || generateResearchQueries(marketTitle, category);
  
  const results: PerplexityResponse[] = [];
  
  for (const query of queries) {
    const result = await queryPerplexity(query, apiKey);
    results.push(result);
    
    // Small delay to avoid rate limiting
    await new Promise(resolve => setTimeout(resolve, 500));
  }

  return {
    queries,
    results,
  };
}
