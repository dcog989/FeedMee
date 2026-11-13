import { describe, expect, it } from "vitest";
import { KeyboardShortcutManager, shortcutKeyFromEvent } from "./shortcuts";

function key(init: KeyboardEventInit): KeyboardEvent {
  return new KeyboardEvent("keydown", init);
}

describe("shortcutKeyFromEvent", () => {
  it("spells the space bar as space", () => {
    expect(shortcutKeyFromEvent(key({ key: " " }))).toBe("space");
  });

  it("puts modifiers in a fixed order and lowercases the key", () => {
    expect(shortcutKeyFromEvent(key({ key: "K", ctrlKey: true, shiftKey: true }))).toBe("ctrl+shift+k");
    expect(shortcutKeyFromEvent(key({ key: "K", altKey: true, metaKey: true }))).toBe("alt+meta+k");
  });

  it("returns bare modifiers when no key is pressed", () => {
    expect(shortcutKeyFromEvent(key({ key: "Control", ctrlKey: true }))).toBe("ctrl");
  });
});

function managerWith(...commands: [string, string][]): KeyboardShortcutManager {
  const manager = new KeyboardShortcutManager();
  for (const [command, defaultKey] of commands) {
    manager.register({ command, defaultKey, description: command, category: "Test" });
  }
  return manager;
}

describe("KeyboardShortcutManager.findConflicts", () => {
  it("returns nothing when every command has its own key", () => {
    const manager = managerWith(["a", "x"], ["b", "y"]);

    expect(manager.findConflicts({})).toEqual([]);
  });

  it("reports a custom mapping that steals another command's key", () => {
    const manager = managerWith(["a", "x"], ["b", "y"]);

    expect(manager.findConflicts({ b: "x" })).toEqual([{ key: "x", commands: ["a", "b"] }]);
  });

  it("ignores case when comparing keys", () => {
    const manager = managerWith(["a", "x"], ["b", "y"]);

    expect(manager.findConflicts({ b: "X" })).toEqual([{ key: "x", commands: ["a", "b"] }]);
  });
});
