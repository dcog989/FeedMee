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
