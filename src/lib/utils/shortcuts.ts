export type ShortcutHandler = (e: KeyboardEvent) => void | Promise<void>;

export interface ShortcutDefinition {
  command: string;
  defaultKey: string;
  description: string;
  category: string;
  handler?: ShortcutHandler;
}

const MODIFIER_NAMES = ["control", "shift", "alt", "meta"];

/**
 * Canonical string for a key event: modifiers in a fixed order plus the key,
 * lowercased, with the space bar spelled "space". Recording and matching must
 * use this same function or a recorded binding never fires.
 */
export function shortcutKeyFromEvent(e: KeyboardEvent): string {
  const parts: string[] = [];
  if (e.ctrlKey) parts.push("ctrl");
  if (e.altKey) parts.push("alt");
  if (e.shiftKey) parts.push("shift");
  if (e.metaKey) parts.push("meta");
  let key = e.key.toLowerCase();
  if (key === " ") key = "space";
  if (!MODIFIER_NAMES.includes(key)) {
    parts.push(key);
  }
  return parts.join("+");
}

export class KeyboardShortcutManager {
  private definitions: Map<string, ShortcutDefinition> = new Map();
  private customMappings: Record<string, string> = {};

  register(definition: ShortcutDefinition): void {
    this.definitions.set(definition.command, definition);
  }

  setCustomMappings(mappings: Record<string, string>): void {
    this.customMappings = mappings;
  }

  private isInputElement(target: EventTarget | null): boolean {
    if (!target || !(target instanceof HTMLElement)) return false;
    const tagName = target.tagName.toLowerCase();
    const isInput = tagName === "input" || tagName === "textarea" || tagName === "select";
    return isInput || target.isContentEditable;
  }

  async handleKeyEvent(e: KeyboardEvent): Promise<boolean> {
    if (e.repeat) return false;

    const pressedKey = shortcutKeyFromEvent(e);
    const isInput = this.isInputElement(e.target);

    if (isInput && e.key !== "Escape" && e.key !== "Enter") {
      return false;
    }

    for (const def of this.definitions.values()) {
      const mappedKey = this.customMappings[def.command] || def.defaultKey;
      if (pressedKey === mappedKey.toLowerCase() && def.handler) {
        e.preventDefault();
        e.stopPropagation();
        try {
          await def.handler(e);
        } catch (err) {
          console.error(`[Shortcuts] Handler failed for "${def.command}":`, err);
        }
        return true;
      }
    }
    return false;
  }

  getShortcutDisplay(commandId: string): string {
    const def = this.definitions.get(commandId);
    if (!def) return "";
    const key = this.customMappings[commandId] || def.defaultKey;
    return key
      .split("+")
      .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
      .join("+");
  }

  clear(): void {
    this.definitions.clear();
  }

  getDefinitions(): ShortcutDefinition[] {
    return Array.from(this.definitions.values());
  }

  /** Keys bound to more than one command. The first match wins at runtime, so the rest are dead. */
  findConflicts(mappings: Record<string, string>): { key: string; commands: string[] }[] {
    const byKey = new Map<string, string[]>();
    for (const def of this.definitions.values()) {
      const key = (mappings[def.command] || def.defaultKey).toLowerCase();
      const commands = byKey.get(key);
      if (commands) commands.push(def.command);
      else byKey.set(key, [def.command]);
    }
    return Array.from(byKey, ([key, commands]) => ({ key, commands })).filter((entry) => entry.commands.length > 1);
  }
}

export const shortcutManager = new KeyboardShortcutManager();
