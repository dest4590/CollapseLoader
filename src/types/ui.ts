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
        size?: string;
    };
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
