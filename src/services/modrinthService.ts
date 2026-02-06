
const MODRINTH_API_URL = 'https://api.modrinth.com/v2';

export interface ModrinthSearchResult {
    project_id: string;
    project_type: string;
    slug: string;
    author: string;
    title: string;
    description: string;
    categories: string[];
    display_categories: string[];
    versions: string[];
    downloads: number;
    follows: number;
    icon_url: string;
    date_created: string;
    date_modified: string;
    latest_version: string;
    license: string;
    client_side: string;
    server_side: string;
    gallery: string[];
}

export interface ModrinthVersion {
    id: string;
    project_id: string;
    author_id: string;
    featured: boolean;
    name: string;
    version_number: string;
    changelog: string;
    changelog_url: string | null;
    date_published: string;
    downloads: number;
    version_type: 'release' | 'beta' | 'alpha';
    files: {
        hashes: {
            sha1: string;
            sha512: string;
        };
        url: string;
        filename: string;
        primary: boolean;
        size: number;
    }[];
    dependencies: {
        version_id: string | null;
        project_id: string | null;
        file_name: string | null;
        dependency_type: 'required' | 'optional' | 'incompatible' | 'embedded';
    }[];
    game_versions: string[];
    loaders: string[];
}

export class ModrinthService {
    private static async request<T>(endpoint: string, params: Record<string, any> = {}): Promise<T> {
        const url = new URL(`${MODRINTH_API_URL}${endpoint}`);
        Object.keys(params).forEach(key => {
            if (params[key] !== undefined && params[key] !== null) {
                url.searchParams.append(key, String(params[key]));
            }
        });

        const response = await fetch(url.toString(), {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                'User-Agent': 'CollapseLoader/ModrinthIntegration (dest4590@gmail.com)'
            }
        });

        if (!response.ok) {
            throw new Error(`Modrinth API Error: ${response.status} ${response.statusText}`);
        }

        return response.json();
    }

    static async searchMods(query: string, filters: {
        limit?: number,
        offset?: number,
        facets?: string,
        index?: 'relevance' | 'downloads' | 'followers' | 'newest' | 'updated'
    } = {}): Promise<{ hits: ModrinthSearchResult[], offset: number, limit: number, total_hits: number }> {
        return this.request('/search', {
            query,
            limit: filters.limit || 20,
            offset: filters.offset || 0,
            facets: filters.facets,
            index: filters.index || 'relevance'
        });
    }

    static async getModVersions(slug: string, loaders: string[] = ['fabric'], game_versions?: string[]): Promise<ModrinthVersion[]> {
        const params: Record<string, any> = {};
        
        if (loaders && loaders.length > 0) {
            params.loaders = JSON.stringify(loaders);
        }
        
        if (game_versions && game_versions.length > 0) {
            params.game_versions = JSON.stringify(game_versions);
        }

        return this.request(`/project/${slug}/version`, params);
    }
    
    static async getProject(slug: string): Promise<any> {
        return this.request(`/project/${slug}`);
    }
}
