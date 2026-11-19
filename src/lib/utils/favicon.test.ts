import { describe, expect, it } from "vitest";
import { getFavicon, getFaviconFallback, handleFaviconError } from "./favicon";

describe("getFavicon", () => {
  it("prefers the site's own icon", () => {
    expect(getFavicon("https://www.independent.co.uk/news/uk/rss")).toBe("https://www.independent.co.uk/favicon.ico");
    expect(getFavicon("https://feeds.bbci.co.uk/news/uk/rss.xml")).toBe("https://feeds.bbci.co.uk/favicon.ico");
  });

  it("returns an empty string for a source without a host", () => {
    expect(getFavicon("not a url")).toBe("");
    expect(getFavicon("bsky:did:plc:abc")).toBe("");
  });
});

describe("getFaviconFallback", () => {
  it("looks up the parent domain, where feed subdomains keep their icon", () => {
    expect(getFaviconFallback("https://feeds.bbci.co.uk/news/uk/rss.xml")).toBe(
      "https://icons.duckduckgo.com/ip3/bbci.co.uk.ico",
    );
  });

  it("does not guess a two-letter parent for an apex domain", () => {
    expect(getFaviconFallback("https://example.com/feed")).toBe("");
  });
});

describe("handleFaviconError", () => {
  function makeIcon(source: string) {
    const wrap = document.createElement("div");
    const img = document.createElement("img");
    img.dataset.fallback = "https://icons.duckduckgo.com/ip3/bbci.co.uk.ico";
    img.src = source;
    wrap.appendChild(img);
    img.addEventListener("error", handleFaviconError);
    return { wrap, img };
  }

  it("loads the fallback icon once", () => {
    const { wrap, img } = makeIcon("https://feeds.bbci.co.uk/favicon.ico");

    img.dispatchEvent(new Event("error"));

    expect(img.src).toBe("https://icons.duckduckgo.com/ip3/bbci.co.uk.ico");
    expect(wrap.classList.contains("favicon-failed")).toBe(false);
  });

  it("marks the wrapper so the fallback letter replaces a second failure", () => {
    const { wrap, img } = makeIcon("https://feeds.bbci.co.uk/favicon.ico");

    img.dispatchEvent(new Event("error"));
    img.dispatchEvent(new Event("error"));

    expect(wrap.classList.contains("favicon-failed")).toBe(true);
  });

  it("gives up immediately when there is no fallback", () => {
    const wrap = document.createElement("div");
    const img = document.createElement("img");
    wrap.appendChild(img);
    img.addEventListener("error", handleFaviconError);

    img.dispatchEvent(new Event("error"));

    expect(wrap.classList.contains("favicon-failed")).toBe(true);
  });
});
