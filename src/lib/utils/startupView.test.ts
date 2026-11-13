import { describe, expect, it } from "vitest";
import type { Feed, Folder } from "../types";
import { pickStartupTarget, startupExpandFolderId } from "./startupView";

const FOLDER_ID = 1;
const DELETED_ID = 99;

function makeFeed(id: number, folderId: number): Feed {
  return {
    id,
    name: `feed ${id}`,
    folder_id: folderId,
    unread_count: 0,
    has_error: false,
    error_count: 0,
    source_type: "rss",
    display_url: "",
    source_id: "",
  };
}

function makeFolders(): Folder[] {
  return [{ id: FOLDER_ID, name: "News", feeds: [makeFeed(10, FOLDER_ID), makeFeed(11, FOLDER_ID)] }];
}

function settings(default_view_type: string, default_view_id = -1) {
  return { default_view_type, default_view_id };
}

describe("pickStartupTarget", () => {
  it("opens Latest by default", () => {
    expect(pickStartupTarget(settings("latest"), makeFolders(), { type: null, id: 0 })).toEqual({ kind: "latest" });
  });

  it("opens Read Later", () => {
    expect(pickStartupTarget(settings("saved"), makeFolders(), { type: null, id: 0 })).toEqual({ kind: "saved" });
  });

  it("restores an existing last-viewed feed and folder", () => {
    expect(pickStartupTarget(settings("last"), makeFolders(), { type: "feed", id: 10 })).toEqual({
      kind: "feed",
      id: 10,
    });
    expect(pickStartupTarget(settings("last"), makeFolders(), { type: "folder", id: FOLDER_ID })).toEqual({
      kind: "folder",
      id: FOLDER_ID,
    });
  });

  it("falls back to Latest when the last feed or folder was deleted", () => {
    expect(pickStartupTarget(settings("last"), makeFolders(), { type: "feed", id: DELETED_ID })).toEqual({
      kind: "latest",
    });
    expect(pickStartupTarget(settings("last"), makeFolders(), { type: "folder", id: DELETED_ID })).toEqual({
      kind: "latest",
    });
  });

  it("falls back to Latest when the configured folder or feed is gone", () => {
    expect(pickStartupTarget(settings("folder", -1), makeFolders(), { type: null, id: 0 })).toEqual({ kind: "latest" });
    expect(pickStartupTarget(settings("folder", DELETED_ID), makeFolders(), { type: null, id: 0 })).toEqual({
      kind: "latest",
    });
    expect(pickStartupTarget(settings("feed", DELETED_ID), makeFolders(), { type: null, id: 0 })).toEqual({
      kind: "latest",
    });
  });

  it("opens a configured feed or folder that still exists", () => {
    expect(pickStartupTarget(settings("feed", 11), makeFolders(), { type: null, id: 0 })).toEqual({
      kind: "feed",
      id: 11,
    });
    expect(pickStartupTarget(settings("folder", FOLDER_ID), makeFolders(), { type: null, id: 0 })).toEqual({
      kind: "folder",
      id: FOLDER_ID,
    });
  });

  it("falls back to Latest for an unknown view type", () => {
    expect(pickStartupTarget(settings("bogus"), makeFolders(), { type: null, id: 0 })).toEqual({ kind: "latest" });
  });

  it("never restores the uncategorized folder (id 0)", () => {
    const folders = [...makeFolders(), { id: 0, name: "", feeds: [makeFeed(20, 0)] }];
    expect(pickStartupTarget(settings("last"), folders, { type: "folder", id: 0 })).toEqual({ kind: "latest" });
    expect(pickStartupTarget(settings("folder", 0), folders, { type: null, id: 0 })).toEqual({ kind: "latest" });
  });
});

describe("startupExpandFolderId", () => {
  it("returns the folder itself or the feed's parent folder", () => {
    expect(startupExpandFolderId({ kind: "folder", id: FOLDER_ID }, makeFolders())).toBe(FOLDER_ID);
    expect(startupExpandFolderId({ kind: "feed", id: 11 }, makeFolders())).toBe(FOLDER_ID);
    expect(startupExpandFolderId({ kind: "latest" }, makeFolders())).toBeNull();
  });
});
