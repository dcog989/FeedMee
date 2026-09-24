const faviconCache = new Map<string, string>();

export function getFavicon(url: string): string {
  try {
    const domain = new URL(url).hostname;
    const cached = faviconCache.get(domain);
    if (cached !== undefined) return cached;
    const result = `https://icons.duckduckgo.com/ip3/${domain}.ico`;
    faviconCache.set(domain, result);
    return result;
  } catch {
    return "";
  }
}

export function handleFaviconError(e: Event) {
  const img = e.currentTarget as HTMLImageElement;
  img.style.display = "none";
  const fallback = img.nextElementSibling;
  if (fallback) fallback.classList.remove("favicon-fallback-hidden");
}
