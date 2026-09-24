const TITLE_MATCH_LENGTH = 30;

function normalize(text: string): string {
  return text.toLowerCase().replace(/\s+/g, " ").trim();
}

export function stripDuplicateTitle(html: string, articleTitle: string): string {
  const doc = new DOMParser().parseFromString(html, "text/html");
  const normalizedTitle = normalize(articleTitle).slice(0, TITLE_MATCH_LENGTH);
  if (!normalizedTitle) return doc.body.innerHTML;
  for (const el of doc.querySelectorAll("h1, h2")) {
    if (normalize(el.textContent ?? "").includes(normalizedTitle)) {
      el.remove();
      break;
    }
  }
  return doc.body.innerHTML;
}

const IMAGE_SOURCE_ATTRIBUTES = ["src", "data-src", "data-original"] as const;

function normalizeUrl(url: string): string {
  return url.trim().toLowerCase();
}

// Parsing decodes HTML entities, so an `&amp;`/`&#038;`-encoded query string
// compares equal to the resolved `image_url` the backend extracted from the
// same markup.
function imageSourceUrls(html: string): string[] {
  const doc = new DOMParser().parseFromString(html, "text/html");
  const urls: string[] = [];
  for (const el of doc.querySelectorAll("[src], [srcset], [data-src], [data-original]")) {
    for (const attr of IMAGE_SOURCE_ATTRIBUTES) {
      const value = el.getAttribute(attr);
      if (value) urls.push(value);
    }
    const srcset = el.getAttribute("srcset");
    if (srcset) {
      for (const candidate of srcset.split(",")) {
        const [url] = candidate.trim().split(/\s+/);
        if (url) urls.push(url);
      }
    }
  }
  return urls;
}

export function bodyEmbedsImage(html: string, imageUrl: string): boolean {
  if (!imageUrl) return false;
  const target = normalizeUrl(imageUrl);
  if (!target) return false;
  return imageSourceUrls(html).some((url) => normalizeUrl(url) === target);
}
