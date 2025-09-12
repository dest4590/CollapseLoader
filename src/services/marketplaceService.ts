import { apiGet, apiPost, apiPatch, apiDelete } from './apiClient';

class MarketplaceService {
    async listPresets(params: Record<string, any> = {}): Promise<any> {
        return apiGet('/api/presets/', { params });
    }

    async getPreset(id: number): Promise<any> {
        return apiGet(`/api/presets/${id}/`);
    }

    async createPreset(payload: any): Promise<any> {
        return apiPost('/api/presets/', payload);
    }

    async likePreset(id: number): Promise<any> {
        return apiPost(`/api/presets/${id}/like/`);
    }

    async unlikePreset(id: number): Promise<any> {
        return apiPost(`/api/presets/${id}/unlike/`);
    }

    async downloadPreset(id: number): Promise<any> {
        return apiPost(`/api/presets/${id}/download/`);
    }

    async updatePreset(id: number, payload: Partial<{ title: string; description: string; is_public: boolean }>): Promise<any> {
        return apiPatch(`/api/presets/${id}/`, payload);
    }

    async deletePreset(id: number): Promise<any> {
        return apiDelete(`/api/presets/${id}/`);
    }

    async listComments(presetId: number): Promise<any[]> {
        return apiGet(`/api/presets/${presetId}/comments/`);
    }

    async addComment(presetId: number, text: string): Promise<any> {
        return apiPost(`/api/presets/${presetId}/comments/`, { text });
    }

    async deleteComment(presetId: number, commentId: number): Promise<any> {
        return apiDelete(`/api/presets/${presetId}/comments/${commentId}/`);
    }
}

export const marketplaceService = new MarketplaceService();