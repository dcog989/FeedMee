export interface Folder {
  id: number;
  name: string;
  feeds: Feed[];
}

export interface Feed {
  id: number;
  name: string;
  url: string;
  folder_id: number | null;
  unread_count: number;
  has_error: boolean;
}

export interface Article {
  id: number;
  feed_id: number;
  title: string;
  summary: string;
  author: string;
  url: string;
  image_url: string;
  timestamp: number;
  is_read: boolean;
  is_saved: boolean;
  has_tags: boolean;
}

export interface Tag {
  id: number;
  name: string;
  color: string;
}

export interface AppSettings {
  feed_refresh_debounce_minutes: number;
  auto_update_interval_minutes: number;
  log_level: string;
  default_view_type: string;
  default_view_id: number;
  auto_collapse_folders: boolean;
  mark_feed_read_on_exit: boolean;
  article_title_font: string;
  article_body_font: string;
  article_title_color: string;
  article_body_color: string;
  article_bg_color: string;
  thumbnail_size: number;
  article_retention_days: number;
}

export const DEFAULT_SETTINGS: AppSettings = {
  feed_refresh_debounce_minutes: 4,
  auto_update_interval_minutes: 30,
  log_level: 'info',
  default_view_type: 'latest',
  default_view_id: -1,
  auto_collapse_folders: true,
  mark_feed_read_on_exit: false,
  article_title_font: '',
  article_body_font: '',
  article_title_color: '',
  article_body_color: '',
  article_bg_color: '',
  thumbnail_size: 0,
  article_retention_days: 90,
};
