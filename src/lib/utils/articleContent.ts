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

// Trailing image-format suffix, optionally followed by a re-encoded format
// (e.g. `.jpg`, `.jpeg.webp`).
const FORMAT_SUFFIX = /\.(jpe?g|png|gif|webp|avif|bmp|tiff?|svg)(\.(webp|avif))?$/i;

interface ImageIdentity {
  host: string;
  name: string;
}

// Image CDNs serve the same asset under different size/format variants: BBC
// uses `/branded_news/1200/…​.jpg` for og:image and `/standard/1800/…​.jpg` in
// the body, while WordPress appends `?w=1600&h=900` style resize parameters.
// Compare the host and the decoded file name, ignoring query strings and
// format suffixes, so the same asset is recognised across those variants.
function imageIdentity(url: string): ImageIdentity | null {
  const trimmed = url.trim();
  if (!trimmed) return null;
  try {
    const parsed = new URL(trimmed, "https://placeholder.invalid");
    const name = decodeURIComponent(parsed.pathname.split("/").pop() ?? "")
      .toLowerCase()
      .replace(FORMAT_SUFFIX, "");
    if (!name) return null;
    return { host: parsed.host.toLowerCase(), name };
  } catch {
    return null;
  }
}

// Parsing decodes HTML entities, and query strings are ignored, so an
// `&amp;`/`&#038;`-encoded URL compares equal to the resolved `image_url` the
// backend extracted from the same markup.
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
  const target = imageIdentity(imageUrl);
  if (!target) return false;
  return imageSourceUrls(html).some((url) => {
    const identity = imageIdentity(url);
    return identity !== null && identity.host === target.host && identity.name === target.name;
  });
}
