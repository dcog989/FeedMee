import { describe, expect, it } from "vitest";
import { mergeArticles } from "./articles";
import { makeArticle } from "./testFixtures";

describe("mergeArticles", () => {
  it("appends articles that are not in the list", () => {
    expect(mergeArticles([makeArticle(1)], [makeArticle(2), makeArticle(3)]).map((a) => a.id)).toEqual([1, 2, 3]);
  });

  it("drops ids already shown, as offset paging can repeat a row", () => {
    expect(mergeArticles([makeArticle(1), makeArticle(2)], [makeArticle(2), makeArticle(3)]).map((a) => a.id)).toEqual([
      1, 2, 3,
    ]);
  });

  it("drops ids repeated inside the incoming page", () => {
    expect(mergeArticles([], [makeArticle(1), makeArticle(1), makeArticle(2)]).map((a) => a.id)).toEqual([1, 2]);
  });

  it("returns the existing list when nothing is new", () => {
    const existing = [makeArticle(1), makeArticle(2)];
    expect(mergeArticles(existing, [makeArticle(2)])).toBe(existing);
  });
});
