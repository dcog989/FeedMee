export interface Folder {
    id: number;
    name: string;
    feeds: Feed[];
}

export interface Feed {
    id: number;
    name: string;
    url: string;
    folder_id: number;
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
    timestamp: number;
    is_read: boolean;
    is_saved: boolean;
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
}
