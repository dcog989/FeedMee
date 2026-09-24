import { describe, expect, it } from "vitest";
import { bodyEmbedsImage, stripDuplicateTitle } from "./articleContent";

describe("stripDuplicateTitle", () => {
  it("removes an h1 that duplicates the article title", () => {
    const html = "<h1>Hello World</h1><p>Body</p>";
    expect(stripDuplicateTitle(html, "Hello World")).toBe("<p>Body</p>");
  });

  it("removes an h2 that duplicates the article title", () => {
    const html = "<h2>Hello World</h2><p>Body</p>";
    expect(stripDuplicateTitle(html, "Hello World")).toBe("<p>Body</p>");
  });

  it("only removes the first matching heading", () => {
    const html = "<h1>Hello World</h1><h1>Hello World</h1>";
    expect(stripDuplicateTitle(html, "Hello World")).toBe("<h1>Hello World</h1>");
  });

  it("keeps non-matching headings", () => {
    const html = "<h1>Something Else</h1><p>Body</p>";
    expect(stripDuplicateTitle(html, "Hello World")).toBe("<h1>Something Else</h1><p>Body</p>");
  });

  it("matches case-insensitively and ignores extra whitespace", () => {
    const html = "<h1>  HELLO   world </h1><p>Body</p>";
    expect(stripDuplicateTitle(html, "hello world")).toBe("<p>Body</p>");
  });

  it("does not remove anything when the title is empty", () => {
    const html = "<h1>Hello World</h1>";
    expect(stripDuplicateTitle(html, "")).toBe(html);
  });
});

describe("bodyEmbedsImage", () => {
  it("returns true when the body references the image (case-insensitive)", () => {
    expect(bodyEmbedsImage('<img src="HTTPS://Example.com/a.png">', "https://example.com/a.png")).toBe(true);
  });

  it("returns false when the image is absent", () => {
    expect(bodyEmbedsImage("<p>Body</p>", "https://example.com/a.png")).toBe(false);
  });

  it("returns false for an empty image url", () => {
    expect(bodyEmbedsImage("<p>Body</p>", "")).toBe(false);
  });

  it("matches a body image whose query string is HTML-entity encoded", () => {
    const html = '<img src="https://i0.wp.com/craphound.com/images/21Sep2026.jpg?w=840&#038;ssl=1">';
    const imageUrl = "https://i0.wp.com/craphound.com/images/21Sep2026.jpg?w=840&ssl=1";
    expect(bodyEmbedsImage(html, imageUrl)).toBe(true);
  });

  it("matches a body image referenced via srcset", () => {
    const html =
      '<img srcset="https://example.com/a-2x.png 2x, https://example.com/a.png 1x" src="https://example.com/a.png">';
    expect(bodyEmbedsImage(html, "https://example.com/a-2x.png")).toBe(true);
  });
});
