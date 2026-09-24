import { beforeEach, describe, expect, it } from "vitest";
import { readInt, readJson, readString, writeInt, writeJson, writeString } from "./persistence";

describe("persistence", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  describe("readInt", () => {
    it("returns the fallback when the key is absent", () => {
      expect(readInt("missing", 42)).toBe(42);
    });

    it("parses a stored integer", () => {
      localStorage.setItem("count", "7");
      expect(readInt("count", 0)).toBe(7);
    });

    it("returns the fallback for a non-numeric value", () => {
      localStorage.setItem("count", "abc");
      expect(readInt("count", 5)).toBe(5);
    });
  });

  describe("writeInt", () => {
    it("stores the value as a string", () => {
      writeInt("count", 3);
      expect(localStorage.getItem("count")).toBe("3");
    });
  });

  describe("readJson", () => {
    it("returns the fallback when the key is absent", () => {
      expect(readJson("missing", { a: 1 })).toEqual({ a: 1 });
    });

    it("parses stored JSON", () => {
      localStorage.setItem("data", JSON.stringify([1, 2]));
      expect(readJson<number[]>("data", [])).toEqual([1, 2]);
    });

    it("returns the fallback for malformed JSON", () => {
      localStorage.setItem("data", "{not json");
      expect(readJson("data", { safe: true })).toEqual({ safe: true });
    });
  });

  describe("writeJson", () => {
    it("round-trips through storage", () => {
      writeJson("data", { a: [1, 2] });
      expect(readJson("data", null)).toEqual({ a: [1, 2] });
    });
  });

  describe("readString/writeString", () => {
    it("returns null when absent and the stored value otherwise", () => {
      expect(readString("key")).toBeNull();
      writeString("key", "value");
      expect(readString("key")).toBe("value");
    });
  });
});
