export type ShortcutHandler = (e: KeyboardEvent) => void | Promise<void>;

export interface ShortcutDefinition {
    id: string;
    command: string;
    defaultKey: string;
    description: string;
    category: string;
    handler?: ShortcutHandler;
}

export class KeyboardShortcutManager {
    private definitions: Map<string, ShortcutDefinition> = new Map();
    private customMappings: Record<string, string> = {};
    private enabled: boolean = true;

    register(definition: ShortcutDefinition): void {
        this.definitions.set(definition.command, definition);
    }

    setCustomMappings(mappings: Record<string, string>): void {
        this.customMappings = mappings;
    }

    getCustomMappings(): Record<string, string> {
        return { ...this.customMappings };
    }

    unregister(commandId: string): void {
        this.definitions.delete(commandId);
    }

    private isInputElement(target: EventTarget | null): boolean {
        if (!target || !(target instanceof HTMLElement)) return false;
        const tagName = target.tagName.toLowerCase();
        const isInput = tagName === 'input' || tagName === 'textarea' || tagName === 'select';
        return isInput || target.isContentEditable;
    }

    private getEventKey(e: KeyboardEvent): string {
        const parts: string[] = [];
        if (e.ctrlKey) parts.push('ctrl');
        if (e.altKey) parts.push('alt');
        if (e.shiftKey) parts.push('shift');
        if (e.metaKey) parts.push('meta');
        let key = e.key.toLowerCase();
        if (key === ' ') key = 'space';
        if (!['control', 'shift', 'alt', 'meta'].includes(key)) {
            parts.push(key);
        }
        return parts.join('+');
    }

    async handleKeyEvent(e: KeyboardEvent): Promise<boolean> {
        if (!this.enabled || e.repeat) return false;

        const pressedKey = this.getEventKey(e);
        const isInput = this.isInputElement(e.target);

        if (isInput && e.key !== 'Escape' && e.key !== 'Enter') {
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

    getShortcutsByCategory(): Map<string, ShortcutDefinition[]> {
        const grouped = new Map<string, ShortcutDefinition[]>();
        for (const def of this.definitions.values()) {
            if (!grouped.has(def.category)) {
                grouped.set(def.category, []);
            }
            grouped.get(def.category)!.push(def);
        }
        return grouped;
    }

    getShortcutDisplay(commandId: string): string {
        const def = this.definitions.get(commandId);
        if (!def) return '';
        const key = this.customMappings[commandId] || def.defaultKey;
        return key
            .split('+')
            .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
            .join('+');
    }

    isRegistered(commandId: string): boolean {
        return this.definitions.has(commandId);
    }

    setEnabled(enabled: boolean): void {
        this.enabled = enabled;
    }

    clear(): void {
        this.definitions.clear();
    }

    getDefinitions(): ShortcutDefinition[] {
        return Array.from(this.definitions.values());
    }
}

export const shortcutManager = new KeyboardShortcutManager();
