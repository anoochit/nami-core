import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export const isThai = (char: string) => /[\u0E00-\u0E7F]/.test(char);

export const shouldAddSpace = (prev: string, next: string) => {
    if (!prev || !next) return false;
    const prevChar = prev.slice(-1);
    const nextChar = next[0];
    
    // Don't add space if either is a newline or whitespace
    if (/\s/.test(prevChar) || /\s/.test(nextChar)) return false;
    
    // Add space if switching between Thai and non-Thai
    return (isThai(prevChar) !== isThai(nextChar));
};

export function formatError(e: Error | string | unknown): string {
  const errStr = typeof e === 'string' 
    ? e 
    : (e instanceof Error ? e.message : JSON.stringify(e));

  if (errStr.includes("insufficient_quota")) {
    return "🚫 API Quota Exceeded\n\nYou have exceeded your OpenAI quota. Please check your plan and billing details.";
  }

  if (errStr.includes("rate_limited") || errStr.includes("429 Too Many Requests")) {
    return "⏳ Rate Limit Reached\n\nThe AI provider is currently rate limiting requests. Please wait a moment before trying again.";
  }

  if (errStr.includes("invalid_api_key") || errStr.includes("401 Unauthorized")) {
    return "🔑 Invalid API Key\n\nThe API key provided is invalid or has expired. Please check your configuration.";
  }

  // Try to extract a clean message from common error patterns
  let cleanMsg = errStr;

  // If it contains a JSON error from a provider, try to parse it
  const jsonMatch = errStr.match(/\{.*\}/);
  if (jsonMatch) {
    try {
      const v = JSON.parse(jsonMatch[0]);
      if (v.error?.message) {
        cleanMsg = v.error.message;
      } else if (v.message) {
        cleanMsg = v.message;
      }
    } catch (err) {
      console.warn("Failed to parse error JSON", err);
    }
  }

  // If it's the specific format ADK uses, try to strip the prefix
  if (cleanMsg.includes("error=")) {
    const parts = cleanMsg.split("error=");
    cleanMsg = parts[parts.length - 1];
  }

  // Strip any remaining JSON-like trailing parts
  if (cleanMsg.includes("): {")) {
    cleanMsg = cleanMsg.split("): {")[0];
  }

  return `❌ Error\n\n${cleanMsg.trim()}`;
}
