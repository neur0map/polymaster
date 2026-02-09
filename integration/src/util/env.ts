import * as dotenv from "dotenv";
import * as path from "path";
import * as fs from "fs";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export interface EnvConfig {
  historyPath: string;
  rapidApiKey: string | undefined;
  perplexityApiKey: string | undefined;
  providersConfigPath: string;
}

/** Load and validate environment variables. Tries .env in integration/ dir first. */
export function loadEnv(): EnvConfig {
  const integrationDir = path.resolve(__dirname, "../..");
  const envPath = path.join(integrationDir, ".env");

  if (fs.existsSync(envPath)) {
    dotenv.config({ path: envPath });
  } else {
    dotenv.config();
  }

  const homeDir = process.env.HOME || process.env.USERPROFILE || "";

  const historyPath = resolveHomePath(
    process.env.WWATCHER_HISTORY_PATH ||
      path.join(homeDir, ".config", "wwatcher", "alert_history.jsonl")
  );

  const providersConfigPath = resolveHomePath(
    process.env.PROVIDERS_CONFIG ||
      path.join(integrationDir, "providers")
  );

  return {
    historyPath,
    rapidApiKey: process.env.RAPIDAPI_KEY || undefined,
    perplexityApiKey: process.env.PERPLEXITY_API_KEY || undefined,
    providersConfigPath,
  };
}

function resolveHomePath(p: string): string {
  const homeDir = process.env.HOME || process.env.USERPROFILE || "";
  if (p.startsWith("~")) {
    return path.join(homeDir, p.slice(1));
  }
  return path.resolve(p);
}
