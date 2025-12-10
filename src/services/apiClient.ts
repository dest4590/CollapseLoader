import axios, { AxiosRequestConfig } from 'axios';
import { getAuthUrl, ensureAuthUrl } from '../config';
import { getCurrentLanguage } from '../i18n';

interface CacheEntry<T = any> {
    data: T;
    timestamp: number;
    ttl: number;
    etag?: string;
}

interface BatchConfig {
    maxBatchSize: number;
    batchTimeout: number;
    enabled: boolean;
}

class ApiClient {
    private client = axios.create();
    private cache = new Map<string, CacheEntry>();
    private pendingRequests = new Map<string, Promise<any>>();
    private requestTimings = new Map<string, number>();

    private batchConfig: BatchConfig = {
        maxBatchSize: 10,
        batchTimeout: 50,
        enabled: true
    };

    constructor() {
        this.setupInterceptors();
        this.setupCacheCleanup();
    }

    private setupInterceptors() {
        this.client.interceptors.request.use(async (config) => {
            await ensureAuthUrl();
            const baseUrl = getAuthUrl();

            if (config.url?.startsWith('/')) {
                config.url = `${baseUrl}${config.url}`;
            }

            const token = localStorage.getItem('authToken');
            if (token) {
                config.headers = config.headers || {};
                config.headers['Authorization'] = `Token ${token}`;
            }

            config.headers = config.headers || {};
            config.headers['Accept-Language'] = getCurrentLanguage() || 'en';


            const requestKey = `${(config.method || 'GET').toUpperCase()}:${config.url}:${this.normalizeParams(config.params)}`;
            this.requestTimings.set(requestKey, Date.now());

            (config as any).__timingKey = requestKey;

            return config;
        });

        this.client.interceptors.response.use(
            (response) => {
                return response;
            },
            (error) => {
                return Promise.reject(error);
            }
        );
    }

    private setupCacheCleanup() {
        setInterval(() => {
            const now = Date.now();
            for (const [key, entry] of this.cache.entries()) {
                if (now - entry.timestamp > entry.ttl) {
                    this.cache.delete(key);
                }
            }
        }, 5 * 60 * 1000);
    }

    private getCacheKey(url: string, config?: AxiosRequestConfig): string {
        const method = (config?.method || 'GET').toUpperCase();
        const params = config?.params ? this.normalizeParams(config.params) : '';
        const data = config?.data ? JSON.stringify(config.data) : '';
        return `${method}:${url}:${params}:${data}`;
    }

    private normalizeParams(params: any): string {
        try {
            if (!params) return '';
            if (typeof params !== 'object') return String(params);
            const keys = Object.keys(params).sort();
            const normalized: any = {};
            for (const k of keys) {
                normalized[k] = params[k];
            }
            return JSON.stringify(normalized);
        } catch (e) {
            console.error('Failed to normalize params:', e);
            return JSON.stringify(params);
        }
    }

    private shouldCache(url: string, method: string = 'GET'): boolean {
        if (method !== 'GET') return false;

        const cacheableEndpoints = [
            '/auth/friends/',
            '/auth/friends/batch/',
            '/auth/init/',
            '/auth/users/search/',
            '/auth/profile/',
            '/auth/admin/dashboard/'
        ];

        return cacheableEndpoints.some(endpoint => url.includes(endpoint));
    }

    private getCacheTTL(url: string): number {
        if (url.includes('/friends/batch/')) return 30000;
        if (url.includes('/friends/status/')) return 15000;
        if (url.includes('/profile/')) return 60000;
        if (url.includes('/init/')) return 120000;
        if (url.includes('/admin/')) return 10000;

        return 30000;
    }

    async get<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
        const cacheKey = this.getCacheKey(url, config);
        const method = 'GET';

        const cached = this.cache.get(cacheKey);

        if (this.shouldCache(url, method)) {
            if (cached && Date.now() - cached.timestamp < cached.ttl) {
                return cached.data;
            }
        }

        if (this.pendingRequests.has(cacheKey)) {
            console.log(`Deduplicating request for ${url}`);
            return this.pendingRequests.get(cacheKey) as Promise<T>;
        }

        const requestConfig: AxiosRequestConfig = { ...(config || {}), method };
        if (cached && cached.etag) {
            requestConfig.headers = { ...(requestConfig.headers || {}) };
            requestConfig.headers['If-None-Match'] = cached.etag;
        }

        const requestPromise = (async () => {
            try {
                const response = await this.executeRequest<T>(url, requestConfig);

                const axiosResp: any = response as any;

                if (axiosResp.status === 304 && cached) {
                    cached.timestamp = Date.now();
                    return cached.data as T;
                }

                const result = axiosResp.data as T;

                if (this.shouldCache(url, method)) {
                    const etag = axiosResp.headers?.etag || axiosResp.headers?.ETag;
                    this.cache.set(cacheKey, {
                        data: result,
                        timestamp: Date.now(),
                        ttl: this.getCacheTTL(url),
                        etag: etag
                    });
                }

                return result;
            } finally {
                this.pendingRequests.delete(cacheKey);
            }
        })();

        this.pendingRequests.set(cacheKey, requestPromise as Promise<any>);

        return requestPromise as Promise<T>;
    }

    async batchGet<T = any>(requests: Array<{ url: string; config?: AxiosRequestConfig }>): Promise<T[]> {
        if (!this.batchConfig.enabled || requests.length === 1) {
            return Promise.all(requests.map(req => this.get<T>(req.url, req.config)));
        }

        const friendsRequests = requests.filter(req =>
            req.url.includes('/friends/') || req.url.includes('/statuses/')
        );

        if (friendsRequests.length > 1) {
            try {
                return await this.getBatchFriendsData() as T[];
            } catch (error) {
                console.warn('Batch friends endpoint failed, falling back to individual requests:', error);
                return Promise.all(requests.map(req => this.get<T>(req.url, req.config)));
            }
        }

        const batchSize = Math.min(requests.length, this.batchConfig.maxBatchSize);
        const batches: Array<Array<{ url: string; config?: AxiosRequestConfig }>> = [];

        for (let i = 0; i < requests.length; i += batchSize) {
            batches.push(requests.slice(i, i + batchSize));
        }

        const results: T[] = [];
        for (const batch of batches) {
            const batchResults = await Promise.all(
                batch.map(req => this.get<T>(req.url, req.config))
            );
            results.push(...batchResults);
        }

        return results;
    }


    private async getBatchFriendsData(): Promise<any> {
        return this.get('/auth/friends/batch/');
    }


    async post<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
        this.invalidateCache(url);

        const resp = await this.executeRequest<T>(url, { ...config, method: 'POST', data });
        return resp.data as T;
    }


    async put<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
        this.invalidateCache(url);
        const resp = await this.executeRequest<T>(url, { ...config, method: 'PUT', data });
        return resp.data as T;
    }


    async patch<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
        this.invalidateCache(url);
        const resp = await this.executeRequest<T>(url, { ...config, method: 'PATCH', data });
        return resp.data as T;
    }


    async delete<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
        this.invalidateCache(url);
        const resp = await this.executeRequest<T>(url, { ...config, method: 'DELETE' });
        return resp.data as T;
    }

    private async executeRequest<T>(url: string, config: AxiosRequestConfig): Promise<import('axios').AxiosResponse<T>> {
        const response = await this.client.request<T>({ url, ...config });
        return response;
    }

    private invalidateCache(url: string) {
        const keysToDelete: string[] = [];

        for (const key of this.cache.keys()) {
            if (key.includes(url) || this.isRelatedEndpoint(url, key)) {
                keysToDelete.push(key);
            }
        }

        keysToDelete.forEach(key => this.cache.delete(key));

        if (keysToDelete.length > 0) {
            console.log(`Invalidated ${keysToDelete.length} cache entries for ${url}`);
        }
    }


    private isRelatedEndpoint(url: string, cacheKey: string): boolean {
        const relationships = [
            ['/friends/', '/friends/batch/', '/friends/status/'],
            ['/profile/', '/init/'],
            ['/admin/', '/admin/dashboard/', '/admin/users/']
        ];

        for (const group of relationships) {
            if (group.some(endpoint => url.includes(endpoint)) &&
                group.some(endpoint => cacheKey.includes(endpoint))) {
                return true;
            }
        }

        return false;
    }

    clearCache() {
        this.cache.clear();
        console.log('API cache cleared');
    }

    async preloadCriticalData(): Promise<void> {
        const criticalEndpoints = [
            '/auth/init/',
            '/auth/friends/batch/'
        ];

        try {
            await Promise.all(
                criticalEndpoints.map(endpoint =>
                    this.get(endpoint).catch(error =>
                        console.warn(`Failed to preload ${endpoint}:`, error)
                    )
                )
            );
            console.log('Critical data preloaded successfully');
        } catch (error) {
            console.error('Failed to preload critical data:', error);
        }
    }

    async heartbeat(): Promise<{ success: boolean; timestamp: string }> {
        return this.post('/auth/heartbeat/', {});
    }

    public invalidateProfileCaches() {
        this.invalidateCache('/auth/profile/');
        this.invalidateCache('/auth/init/');
    }
}

export const apiClient = new ApiClient();

export const apiGet = apiClient.get.bind(apiClient);
export const apiPost = apiClient.post.bind(apiClient);
export const apiPut = apiClient.put.bind(apiClient);
export const apiPatch = apiClient.patch.bind(apiClient);
export const apiDelete = apiClient.delete.bind(apiClient);

export const apiBatchGet = apiClient.batchGet.bind(apiClient);
export const apiPreload = apiClient.preloadCriticalData.bind(apiClient);
export const apiHeartbeat = apiClient.heartbeat.bind(apiClient);
export const apiInvalidateProfile = apiClient.invalidateProfileCaches.bind(apiClient);

export default apiClient;
