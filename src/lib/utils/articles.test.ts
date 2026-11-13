import { describe, expect, it } from "vitest";
import type { Article } from "../types";
import { mergeArticles } from "./articles";

function article(id: number): Article {
  return {
    id,
    feed_id: 1,
    title: `Article ${id}`,
    summary: "",
    author: "",
    url: `https://example.com/${id}`,
    image_url: "",
    timestamp: id,
    is_read: false,
    is_saved: false,
    has_tags: false,
  };
}

describe("mergeArticles", () => {
  it("appends articles that are not in the list", () => {
    expect(mergeArticles([article(1)], [article(2), article(3)]).map((a) => a.id)).toEqual([1, 2, 3]);
  });

  it("drops ids already shown, as offset paging can repeat a row", () => {
    expect(mergeArticles([article(1), article(2)], [article(2), article(3)]).map((a) => a.id)).toEqual([1, 2, 3]);
  });

  it("drops ids repeated inside the incoming page", () => {
    expect(mergeArticles([], [article(1), article(1), article(2)]).map((a) => a.id)).toEqual([1, 2]);
  });

  it("returns the existing list when nothing is new", () => {
    const existing = [article(1), article(2)];
    expect(mergeArticles(existing, [article(2)])).toBe(existing);
  });
});
