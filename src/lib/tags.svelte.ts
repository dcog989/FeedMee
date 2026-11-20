import { invoke } from "@tauri-apps/api/core";
import type { Tag } from "./types";

export function createTagOps() {
  async function getArticleTags(articleId: number): Promise<Tag[]> {
    return await invoke<Tag[]>("get_tags_for_article", { articleId });
  }

  async function getAllTags(): Promise<Tag[]> {
    return await invoke<Tag[]>("get_all_tags");
  }

  async function addTag(articleId: number, name: string): Promise<Tag> {
    return await invoke<Tag>("add_tag", { articleId, name });
  }

  async function removeTag(articleId: number, tagId: number): Promise<void> {
    await invoke("remove_tag", { articleId, tagId });
  }

  async function deleteTag(tagId: number): Promise<void> {
    await invoke("delete_tag", { tagId });
  }

  return {
    getArticleTags,
    getAllTags,
    addTag,
    removeTag,
    deleteTag,
  };
}
