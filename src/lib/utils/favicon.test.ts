import { describe, expect, it } from "vitest";
import { getFavicon, handleFaviconError } from "./favicon";

describe("getFavicon", () => {
  it("returns a DuckDuckGo icon URL for a valid URL", () => {
    expect(getFavicon("https://example.com/feed.xml")).toBe("https://icons.duckduckgo.com/ip3/example.com.ico");
  });

  it("returns an empty string for an invalid URL", () => {
    expect(getFavicon("not a url")).toBe("");
  });
});

describe("handleFaviconError", () => {
  it("marks the wrapper so the fallback can replace the broken icon", () => {
    const wrap = document.createElement("div");
    const img = document.createElement("img");
    wrap.appendChild(img);
    img.addEventListener("error", handleFaviconError);

    img.dispatchEvent(new Event("error"));

    expect(wrap.classList.contains("favicon-failed")).toBe(true);
  });
});
