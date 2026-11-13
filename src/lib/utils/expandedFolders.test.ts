import { describe, expect, it } from "vitest";
import { collapseAllFolders, expandAllFolders, expandFolder, toggleFolder } from "./expandedFolders";

const FOLDER_A = 1;
const FOLDER_B = 2;

describe("expandFolder", () => {
  it("keeps other folders open when auto-collapse is off", () => {
    expect([...expandFolder(new Set([FOLDER_A]), FOLDER_B, false)]).toEqual([FOLDER_A, FOLDER_B]);
  });

  it("closes the other folders when auto-collapse is on", () => {
    expect([...expandFolder(new Set([FOLDER_A]), FOLDER_B, true)]).toEqual([FOLDER_B]);
  });
});

describe("toggleFolder", () => {
  it("opens a closed folder", () => {
    expect([...toggleFolder(new Set(), FOLDER_A, false)]).toEqual([FOLDER_A]);
  });

  it("closes an open folder", () => {
    expect([...toggleFolder(new Set([FOLDER_A]), FOLDER_A, false)]).toEqual([]);
  });

  it("closes the other folders when auto-collapse is on", () => {
    expect([...toggleFolder(new Set([FOLDER_A]), FOLDER_B, true)]).toEqual([FOLDER_B]);
  });
});

describe("expandAllFolders", () => {
  it("opens every given folder", () => {
    expect([...expandAllFolders([{ id: FOLDER_A }, { id: FOLDER_B }])]).toEqual([FOLDER_A, FOLDER_B]);
  });
});

describe("collapseAllFolders", () => {
  it("returns an empty set", () => {
    expect(collapseAllFolders().size).toBe(0);
  });
});
