export const LS_NAV_WIDTH = "navWidth";
export const LS_LIST_WIDTH = "listWidth";
export const LS_SORT_ORDER = "sortOrder";
export const LS_THEME = "theme";
export const LS_BLOCKED_PHRASES = "blockedPhrases";
export const LS_LAST_REFRESHED = "lastRefreshed";
export const LS_LAST_VIEW_TYPE = "lastViewType";
export const LS_LAST_VIEW_ID = "lastViewId";
export const LS_EXPANDED_FOLDERS = "appState.expandedFolders";

export function readString(key: string): string | null {
  return localStorage.getItem(key);
}

export function writeString(key: string, value: string): void {
  localStorage.setItem(key, value);
}

export function readInt(key: string, fallback: number): number {
  const raw = readString(key);
  if (raw === null) return fallback;
  const value = parseInt(raw, 10);
  return Number.isNaN(value) ? fallback : value;
}

export function writeInt(key: string, value: number): void {
  localStorage.setItem(key, String(value));
}

export function readJson<T>(key: string, fallback: T): T {
  const raw = readString(key);
  if (raw === null) return fallback;
  try {
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

export function writeJson(key: string, value: unknown): void {
  localStorage.setItem(key, JSON.stringify(value));
}
