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

export function bodyEmbedsImage(html: string, imageUrl: string): boolean {
  if (!imageUrl) return false;
  return html.toLowerCase().includes(imageUrl.toLowerCase());
}
