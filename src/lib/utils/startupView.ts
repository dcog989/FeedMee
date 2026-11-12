import type { Folder } from "../types";
import { findFeedFolderId } from "./feeds";

export type StartupTarget =
  | { kind: "latest" }
  | { kind: "saved" }
  | { kind: "feed"; id: number }
  | { kind: "folder"; id: number };

interface StartupSettings {
  default_view_type: string;
  default_view_id: number;
}

/**
 * Picks the view to open on launch. Saved ids are checked against the current
 * folders so a deleted feed or folder falls back to Latest instead of an empty
 * list with nothing selected.
 */
export function pickStartupTarget(
  settings: StartupSettings,
  folders: Folder[],
  lastView: { type: string | null; id: number },
): StartupTarget {
  const hasFolder = (id: number) => folders.some((f) => f.id === id);
  const hasFeed = (id: number) => findFeedFolderId(folders, id) !== null;

  switch (settings.default_view_type) {
    case "saved":
      return { kind: "saved" };
    case "last":
      if (lastView.type === "folder" && hasFolder(lastView.id)) return { kind: "folder", id: lastView.id };
      if (lastView.type === "feed" && hasFeed(lastView.id)) return { kind: "feed", id: lastView.id };
      return { kind: "latest" };
    case "folder":
      return hasFolder(settings.default_view_id)
        ? { kind: "folder", id: settings.default_view_id }
        : { kind: "latest" };
    case "feed":
      return hasFeed(settings.default_view_id) ? { kind: "feed", id: settings.default_view_id } : { kind: "latest" };
    default:
      return { kind: "latest" };
  }
}

/** The folder to auto-expand after opening the startup target, if any. */
export function startupExpandFolderId(target: StartupTarget, folders: Folder[]): number | null {
  if (target.kind === "folder") return target.id;
  if (target.kind === "feed") return findFeedFolderId(folders, target.id);
  return null;
}
