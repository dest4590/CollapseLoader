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
    meta: {
        [key: string]: any;
        installed?: boolean;
        is_custom?: boolean;
        size?: string;
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
    created_at: string;
}

export interface InstallProgress {
    percentage: number;
    action: string;
    isComplete: boolean;
}
