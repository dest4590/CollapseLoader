import { invoke } from "@tauri-apps/api/core";
import { useToast } from "./toastService";
import { getCurrentLanguage } from "../i18n";
import { getApiBaseWithVersion, ensureApiUrl } from "../config";

export interface ApiResponse<T> {
    success: boolean;
    data: T;
    error: string | null;
}

export class ApiResponseError extends Error {
    response: { status: number; data: ApiResponse<any> };

    constructor(resp: ApiResponse<any>, status = 200) {
        super(resp?.error || "Request failed");
        this.name = "ApiResponseError";
        this.response = { status, data: resp };
    }
}

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

        const token = localStorage.getItem("authToken");

        const requestHeaders: Record<string, string> = {
            ...headers,
            "Accept-Language": getCurrentLanguage() || "en",
        };

        if (token) {
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
                body: body,
            });

            return this.unwrapResponse<T>(response, requestHeaders);
        } catch (error: any) {
            if (headers["X-Skip-Toast"] !== "true") {
                const { addToast } = useToast();
                addToast(error.toString(), "error");
            }
            throw error;
        }
    }

    private unwrapResponse<T>(
        payload: any,
        headers: Record<string, string>
    ): T {
        if (this.isApiResponse(payload)) {
            if (!payload.success) {
                if (headers["X-Skip-Toast"] !== "true") {
                    const { addToast } = useToast();
                    addToast(payload.error || "Request failed", "error");
                }
                throw new ApiResponseError(payload);
            }
            return payload.data as T;
        }
        return payload as T;
    }

    private isApiResponse(value: any): value is ApiResponse<any> {
        return (
            !!value &&
            typeof value === "object" &&
            typeof value.success === "boolean" &&
            "data" in value
        );
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
