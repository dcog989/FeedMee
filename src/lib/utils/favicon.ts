const DDG_ICON_URL = "https://icons.duckduckgo.com/ip3";

function hostOf(url: string): string {
  try {
    return new URL(url).hostname;
  } catch {
    return "";
  }
}

/** The site's own icon. Preferred over icon services: DuckDuckGo answers an
 * unknown domain with a 404 *image*, which the browser renders as a generic
 * grey placeholder instead of firing `error`. */
export function getFavicon(url: string): string {
  const host = hostOf(url);
  return host ? `https://${host}/favicon.ico` : "";
}

/** Icon-service lookup for the parent domain, used when the site serves no
 * `/favicon.ico` of its own. Feed hosts are often subdomains that host nothing
 * (`feeds.bbci.co.uk`) while the icon sits with `bbci.co.uk`. */
export function getFaviconFallback(url: string): string {
  const host = hostOf(url);
  const parent = host.split(".").slice(1).join(".");
  return parent.includes(".") ? `${DDG_ICON_URL}/${parent}.ico` : "";
}

/**
 * Load the fallback icon once, then mark the container so its CSS can swap the
 * last broken icon for the fallback letter. Uses the parent, not the next
 * sibling, so markup order is free.
 */
export function handleFaviconError(e: Event) {
  const img = e.currentTarget as HTMLImageElement;
  const fallback = img.dataset.fallback;
  if (fallback) {
    delete img.dataset.fallback;
    img.src = fallback;
    return;
  }
  img.parentElement?.classList.add("favicon-failed");
}
