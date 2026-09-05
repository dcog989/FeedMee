const FAVICON_TTL = 48 * 60 * 60 * 1000;
const faviconCache = new Map<string, { url: string; time: number }>();

export function getFavicon(url: string): string {
  try {
    const domain = new URL(url).hostname;
    const cached = faviconCache.get(domain);
    if (cached && Date.now() - cached.time < FAVICON_TTL) {
      return cached.url;
    }
    const result = `https://icons.duckduckgo.com/ip3/${domain}.ico`;
    faviconCache.set(domain, { url: result, time: Date.now() });
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
