import { invoke } from "@tauri-apps/api/core";
import { useToast } from "@shared/composables/useToast";
import { getCurrentLanguage } from "@services/i18n";
import { getApiBaseWithVersion, ensureApiUrl } from "@/config";
import { STORAGE_KEYS } from "@shared/utils/storageKeys";

class ApiClient {
    async request<T = any>(
        method: string,
        url: string,
        data?: any,
        headers: Record<string, string> = {}
    ): Promise<T> {
        await ensureApiUrl();
        const baseUrl = getApiBaseWithVersion();

        let fullUrl = url;
        if (url.startsWith("/")) {
            fullUrl = `${baseUrl}${url}`;
        }

        const token = localStorage.getItem(STORAGE_KEYS.AUTH_TOKEN);

        const requestHeaders: Record<string, string> = {
            ...headers,
            "Accept-Language": getCurrentLanguage() || "en",
        };

        if (token && url.startsWith("/")) {
            requestHeaders["Authorization"] = `Bearer ${token}`;
        }

        try {
            let body = data;
            if (data instanceof FormData) {
                const formData: Record<string, any> = {};
                for (const [key, value] of (data as any).entries()) {
                    if (value instanceof File) {
                        const buffer = await value.arrayBuffer();
                        const bytes = new Uint8Array(buffer);
                        let binary = "";
                        const chunkSize = 8192;
                        for (let i = 0; i < bytes.length; i += chunkSize) {
                            binary += String.fromCharCode.apply(
                                null,
                                Array.from(bytes.subarray(i, i + chunkSize))
                            );
                        }
                        const base64 = btoa(binary);
                        formData[key] = {
                            __type: "file",
                            name: value.name,
                            type: value.type,
                            data: base64,
                        };
                    } else {
                        formData[key] = value;
                    }
                }
                body = formData;
            }

            const response = await invoke<any>("api_request", {
                method,
                url: fullUrl,
                headers: requestHeaders,
                body: body || null,
            });

            return response;
        } catch (error: any) {
            if (headers["X-Skip-Toast"] !== "true") {
                const { addToast } = useToast();
                addToast(error.toString(), "error");
            }
            throw error;
        }
    }

    get<T = any>(url: string, data?: any, config: any = {}): Promise<T> {
        return this.request<T>("GET", url, data, config.headers);
    }

    post<T = any>(url: string, data?: any, config: any = {}): Promise<T> {
        return this.request<T>("POST", url, data, config.headers);
    }

    put<T = any>(url: string, data?: any, config: any = {}): Promise<T> {
        return this.request<T>("PUT", url, data, config.headers);
    }

    patch<T = any>(url: string, data?: any, config: any = {}): Promise<T> {
        return this.request<T>("PATCH", url, data, config.headers);
    }

    delete<T = any>(url: string, config: any = {}): Promise<T> {
        return this.request<T>("DELETE", url, undefined, config.headers);
    }
}

export const apiClient = new ApiClient();
export const apiGet = apiClient.get.bind(apiClient);
export const apiPost = apiClient.post.bind(apiClient);
export const apiPut = apiClient.put.bind(apiClient);
export const apiPatch = apiClient.patch.bind(apiClient);
export const apiDelete = apiClient.delete.bind(apiClient);
