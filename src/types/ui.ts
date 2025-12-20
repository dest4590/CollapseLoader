export interface Client {
    id: number;
    name: string;
    version: string;
    filename: string;
    md5_hash: string;
    main_class: string;
    show: boolean;
    working: boolean;
    insecure: boolean;
    launches: number;
    downloads: number;
    client_type?: string; // 'default' | 'fabric'
    meta: {
        [key: string]: any;
        installed?: boolean;
        is_custom?: boolean;
        size?: string | number;
    };
}

export interface CustomClient {
    id: number;
    name: string;
    version: string;
    filename: string;
    file_path: string;
    main_class: string;
    description?: string;
    created_at: string;
    is_installed: boolean;
}

export interface ClientDetails {
    source_link: string;
    screenshot_urls: string[];
    changelog_entries: {
        version: string;
        content: string;
        created_at: string;
    }[];
    client_type?: string;
    created_at: string;
    comments_count?: number;
}

export interface ClientComment {
    id: number;
    client: number;
    user: number;
    author_username: string;
    author_avatar: string | null;
    content: string;
    created_at: string;
}

export interface InstallProgress {
    percentage: number;
    action: string;
    isComplete: boolean;
}
