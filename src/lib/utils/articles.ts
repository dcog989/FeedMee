import type { Article } from "../types";

/**
 * Appends new articles, dropping ids already in the list. Offset paging can
 * hand back rows we already show when a refresh inserts articles mid-scroll,
 * and a keyed list cannot hold the same id twice.
 */
export function mergeArticles(existing: Article[], incoming: Article[]): Article[] {
  const seen = new Set(existing.map((article) => article.id));
  const fresh: Article[] = [];
  for (const article of incoming) {
    if (seen.has(article.id)) continue;
    seen.add(article.id);
    fresh.push(article);
  }
  return fresh.length === 0 ? existing : [...existing, ...fresh];
}
