import { beforeEach, describe, expect, it, vi } from "vitest";
import { setupKeyHandler } from "./keyboardNav.svelte";
import { shortcutManager } from "./utils/shortcuts";

function key(init: KeyboardEventInit): KeyboardEvent {
  return new KeyboardEvent("keydown", { bubbles: true, ...init });
}

function makeState() {
  return {
    showSettings: false,
    focusedPane: "nav" as const,
    selectedArticle: null,
    navUp: vi.fn(),
    navDown: vi.fn(),
    articleUp: vi.fn(),
    articleDown: vi.fn(),
  };
}

const action = vi.fn();
let state: ReturnType<typeof makeState>;

beforeEach(() => {
  document.body.innerHTML = "";
  action.mockReset();
  shortcutManager.register({
    command: "probe",
    defaultKey: "q",
    description: "probe",
    category: "Test",
    handler: action,
  });
  state = makeState();
  setupKeyHandler(state);
});

describe("setupKeyHandler", () => {
  it("runs a shortcut for a key pressed outside a field", () => {
    window.dispatchEvent(key({ key: "q" }));

    expect(action).toHaveBeenCalledTimes(1);
  });

  it("does nothing while a dialog is open", () => {
    const overlay = document.createElement("div");
    overlay.className = "modal-overlay";
    document.body.appendChild(overlay);
    window.dispatchEvent(key({ key: "q" }));

    expect(action).not.toHaveBeenCalled();

    overlay.remove();
    window.dispatchEvent(key({ key: "q" }));

    expect(action).toHaveBeenCalledTimes(1);
  });

  it("ignores keys typed into a field", () => {
    const input = document.createElement("input");
    document.body.appendChild(input);
    input.dispatchEvent(key({ key: "q" }));

    expect(action).not.toHaveBeenCalled();
  });

  it("moves the selection with the arrow keys in the focused pane", () => {
    window.dispatchEvent(key({ key: "ArrowDown" }));

    expect(state.navDown).toHaveBeenCalledTimes(1);
    expect(action).not.toHaveBeenCalled();
  });
});
