import { invoke } from '@tauri-apps/api/core';
import type { Tag } from './types';

export function createTagOps(_state: unknown) {
  async function getArticleTags(articleId: number): Promise<Tag[]> {
    try {
      return await invoke<Tag[]>('get_tags_for_article', { articleId });
    } catch (e) {
      console.error('Failed to get tags:', e);
      return [];
    }
  }

  async function getAllTags(): Promise<Tag[]> {
    try {
      return await invoke<Tag[]>('get_all_tags');
    } catch (e) {
      console.error('Failed to get all tags:', e);
      return [];
    }
  }

  async function addTag(articleId: number, name: string, color = '#4899ec'): Promise<Tag> {
    return await invoke<Tag>('add_tag', { articleId, name, color });
  }

  async function removeTag(articleId: number, tagId: number): Promise<void> {
    await invoke('remove_tag', { articleId, tagId });
  }

  async function deleteTag(tagId: number): Promise<void> {
    await invoke('delete_tag', { tagId });
  }

  return {
    getArticleTags,
    getAllTags,
    addTag,
    removeTag,
    deleteTag,
  };
}
