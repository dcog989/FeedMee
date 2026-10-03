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

  it("matches a body image served from a different CDN size variant", () => {
    const ogImage =
      "https://ichef.bbci.co.uk/ace/branded_news/1200/cpsprodpb/1255/live/cd194e30-be43-11f1-beff-83dd12d67cca.jpg";
    const html =
      '<img src="https://ichef.bbci.co.uk/ace/standard/1800/cpsprodpb/1255/live/cd194e30-be43-11f1-beff-83dd12d67cca.jpg">';
    expect(bodyEmbedsImage(html, ogImage)).toBe(true);
  });

  it("matches a body image with different resize query parameters", () => {
    const ogImage =
      "https://static0.howtogeekimages.com/wordpress/wp-content/uploads/wm/2026/09/linux-hero-2.jpeg?w=1600&h=900&fit=crop";
    const html =
      '<img src="https://static0.howtogeekimages.com/wordpress/wp-content/uploads/wm/2026/09/linux-hero-2.jpeg?q=49&amp;fit=crop&amp;w=1600&amp;h=1200&amp;dpr=2">';
    expect(bodyEmbedsImage(html, ogImage)).toBe(true);
  });

  it("matches a body image served in an alternate format", () => {
    const ogImage = "https://example.com/photo.jpg";
    const html = '<img src="https://example.com/photo.jpg.webp">';
    expect(bodyEmbedsImage(html, ogImage)).toBe(true);
  });

  it("does not match a different image with the same host", () => {
    const html = '<img src="https://example.com/b.png">';
    expect(bodyEmbedsImage(html, "https://example.com/a.png")).toBe(false);
  });

  it("does not match the same file name on a different host", () => {
    const html = '<img src="https://cdn-a.example.com/a.png">';
    expect(bodyEmbedsImage(html, "https://cdn-b.example.com/a.png")).toBe(false);
  });
});
