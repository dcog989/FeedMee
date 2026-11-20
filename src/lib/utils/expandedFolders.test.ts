import { describe, expect, it } from "vitest";
import {
  collapseAllFolders,
  expandAllFolders,
  expandFolder,
  expandFolderKeepingOthers,
  toggleFolder,
} from "./expandedFolders";

const FOLDER_A = 1;
const FOLDER_B = 2;

describe("expandFolder", () => {
  it("keeps only the expanded folder open", () => {
    expect([...expandFolder(FOLDER_B)]).toEqual([FOLDER_B]);
  });
});

describe("expandFolderKeepingOthers", () => {
  it("keeps the other folders open", () => {
    expect([...expandFolderKeepingOthers(new Set([FOLDER_A]), FOLDER_B)]).toEqual([FOLDER_A, FOLDER_B]);
  });
});

describe("toggleFolder", () => {
  it("opens a closed folder", () => {
    expect([...toggleFolder(new Set(), FOLDER_A)]).toEqual([FOLDER_A]);
  });

  it("closes an open folder", () => {
    expect([...toggleFolder(new Set([FOLDER_A]), FOLDER_A)]).toEqual([]);
  });

  it("closes the other folders when opening one", () => {
    expect([...toggleFolder(new Set([FOLDER_A]), FOLDER_B)]).toEqual([FOLDER_B]);
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
