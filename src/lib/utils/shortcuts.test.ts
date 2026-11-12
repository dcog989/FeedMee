import { describe, expect, it } from "vitest";
import { KeyboardShortcutManager } from "./shortcuts";

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
