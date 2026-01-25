import axios, { AxiosRequestConfig, AxiosResponse } from 'axios';
import { getApiBaseWithVersion, ensureApiUrl } from '../config';
import { getCurrentLanguage } from '../i18n';

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
        const resp = await this.executeRequest<T>(url, { ...config, method: 'GET' });
        return resp.data as T;
    }


    async post<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
        const resp = await this.executeRequest<T>(url, { ...config, method: 'POST', data });
        return resp.data as T;
    }


    async put<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
        const resp = await this.executeRequest<T>(url, { ...config, method: 'PUT', data });
        return resp.data as T;
    }


    async patch<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
        const resp = await this.executeRequest<T>(url, { ...config, method: 'PATCH', data });
        return resp.data as T;
    }


    async delete<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
        const resp = await this.executeRequest<T>(url, { ...config, method: 'DELETE' });
        return resp.data as T;
    }

    private async executeRequest<T>(url: string, config: AxiosRequestConfig): Promise<AxiosResponse<T>> {
        return await this.client.request<T>({ url, ...config });
    }
}

export const apiClient = new ApiClient();

export const apiGet = apiClient.get.bind(apiClient);
export const apiPost = apiClient.post.bind(apiClient);
export const apiPut = apiClient.put.bind(apiClient);
export const apiPatch = apiClient.patch.bind(apiClient);
export const apiDelete = apiClient.delete.bind(apiClient);
