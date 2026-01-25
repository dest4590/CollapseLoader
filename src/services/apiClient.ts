import axios, { AxiosRequestConfig, AxiosResponse } from 'axios';
import { getApiBaseWithVersion, ensureApiUrl } from '../config';
import { getCurrentLanguage } from '../i18n';

export interface ApiResponse<T> {
    success: boolean;
    data: T;
    error: string | null;
    timestamp: string;
}

const isApiResponse = (value: any): value is ApiResponse<any> => {
    return (
        !!value &&
        typeof value === 'object' &&
        typeof value.success === 'boolean' &&
        'data' in value &&
        'timestamp' in value
    );
};

export class ApiResponseError extends Error {
    response: { status: number; data: ApiResponse<any> };

    constructor(resp: ApiResponse<any>) {
        super(resp?.error || 'Request failed');
        this.name = 'ApiResponseError';
        this.response = { status: 200, data: resp };
    }
}

class ApiClient {
    private client = axios.create({
        baseURL: ''
    });

    constructor() {
        this.setupInterceptors();
    }

    private setupInterceptors() {
        this.client.interceptors.request.use(async (config) => {
            await ensureApiUrl();
            const baseUrl = getApiBaseWithVersion();

            if (config.url?.startsWith('/')) {
                config.url = `${baseUrl}${config.url}`;
            }

            const token = localStorage.getItem('authToken');
            if (token) {
                config.headers = config.headers || {};
                config.headers['Authorization'] = `Bearer ${token}`;
            }

            config.headers = config.headers || {};
            config.headers['Accept-Language'] = getCurrentLanguage() || 'en';

            return config;
        });
    }

    async get<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
        const resp = await this.executeRequest<any>(url, { ...config, method: 'GET' });
        return this.unwrapResponse<T>(resp.data);
    }


    async post<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
        const resp = await this.executeRequest<any>(url, { ...config, method: 'POST', data });
        return this.unwrapResponse<T>(resp.data);
    }


    async put<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
        const resp = await this.executeRequest<any>(url, { ...config, method: 'PUT', data });
        return this.unwrapResponse<T>(resp.data);
    }


    async patch<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
        const resp = await this.executeRequest<any>(url, { ...config, method: 'PATCH', data });
        return this.unwrapResponse<T>(resp.data);
    }


    async delete<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
        const resp = await this.executeRequest<any>(url, { ...config, method: 'DELETE' });
        return this.unwrapResponse<T>(resp.data);
    }

    private async executeRequest<T>(url: string, config: AxiosRequestConfig): Promise<AxiosResponse<T>> {
        return await this.client.request<T>({ url, ...config });
    }

    private unwrapResponse<T>(payload: any): T {
        if (isApiResponse(payload)) {
            if (!payload.success) {
                throw new ApiResponseError(payload);
            }
            return payload.data as T;
        }
        return payload as T;
    }
}

export const apiClient = new ApiClient();

export const apiGet = apiClient.get.bind(apiClient);
export const apiPost = apiClient.post.bind(apiClient);
export const apiPut = apiClient.put.bind(apiClient);
export const apiPatch = apiClient.patch.bind(apiClient);
export const apiDelete = apiClient.delete.bind(apiClient);
