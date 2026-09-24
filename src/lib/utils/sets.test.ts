import { describe, expect, it } from "vitest";
import { withAdded, withRemoved } from "./sets";

describe("withAdded", () => {
  it("returns a new set with the item added", () => {
    const current = new Set([1, 2]);
    const next = withAdded(current, 3);
    expect(next).not.toBe(current);
    expect([...next]).toEqual([1, 2, 3]);
  });

  it("adds multiple items without mutating the original", () => {
    const current = new Set([1]);
    const next = withAdded(current, 2, 3);
    expect([...next]).toEqual([1, 2, 3]);
    expect([...current]).toEqual([1]);
  });
});

describe("withRemoved", () => {
  it("returns a new set with the item removed", () => {
    const current = new Set([1, 2]);
    const next = withRemoved(current, 1);
    expect(next).not.toBe(current);
    expect([...next]).toEqual([2]);
    expect([...current]).toEqual([1, 2]);
  });

  it("removes multiple items", () => {
    const current = new Set([1, 2, 3]);
    expect([...withRemoved(current, 1, 3)]).toEqual([2]);
  });
});
