import { apiGet, apiPost, apiPatch, apiDelete } from "@api/clients/internal";
import type {
    MarketplacePreset,
    MarketplaceTheme,
} from "@features/presets/types";

const themeFieldAliases: Record<keyof MarketplaceTheme, string[]> = {
    customCSS: ["customCSS", "custom_css", "customCss"],
    enableCustomCSS: [
        "enableCustomCSS",
        "enable_custom_css",
        "enableCustomCss",
    ],
    base100: ["base100"],
    base200: ["base200"],
    base300: ["base300"],
    baseContent: ["baseContent", "base_content"],
    primary: ["primary"],
    primaryContent: ["primaryContent", "primary_content"],
    secondary: ["secondary"],
    secondaryContent: ["secondaryContent", "secondary_content"],
    accent: ["accent"],
    accentContent: ["accentContent", "accent_content"],
    neutral: ["neutral"],
    neutralContent: ["neutralContent", "neutral_content"],
    info: ["info"],
    infoContent: ["infoContent", "info_content"],
    success: ["success"],
    successContent: ["successContent", "success_content"],
    warning: ["warning"],
    warningContent: ["warningContent", "warning_content"],
    error: ["error"],
    errorContent: ["errorContent", "error_content"],
    backgroundImage: ["backgroundImage", "background_image"],
    backgroundBlur: ["backgroundBlur", "background_blur"],
    backgroundOpacity: ["backgroundOpacity", "background_opacity"],
};

function extractTheme(source: Record<string, any> = {}): MarketplaceTheme {
    const theme: MarketplaceTheme = {};
    for (const key of Object.keys(
        themeFieldAliases
    ) as (keyof MarketplaceTheme)[]) {
        const aliases = themeFieldAliases[key];
        for (const alias of aliases) {
            if (Object.prototype.hasOwnProperty.call(source, alias)) {
                let val = source[alias];

                if (
                    (key === "backgroundBlur" || key === "backgroundOpacity") &&
                    val !== null &&
                    val !== undefined
                ) {
                    val = Number(val);
                }
                theme[key] = val;
                break;
            }
        }
    }
    return theme;
}

function normalizePreset(raw: Record<string, any>): MarketplacePreset {
    const themeSource = raw.theme ?? raw.preset_data ?? raw;
    const theme = extractTheme(themeSource);
    const presetId: number | string =
        raw.id ?? raw.presetId ?? raw.preset_id ?? raw.name ?? raw.title ?? "";
    const ownerUsername =
        raw.author?.username ??
        raw.author?.name ??
        raw.owner_username ??
        raw.owner?.username ??
        raw.owner?.name;

    return {
        ...raw,
        id: presetId,
        title: raw.title ?? raw.name,
        name: raw.name ?? raw.title,
        owner_username: ownerUsername,
        author: raw.author,
        liked: raw.liked ?? raw.liked_by_user ?? false,
        theme,
        preset_data:
            raw.preset_data ??
            (raw.theme
                ? raw.theme
                : themeSource === raw
                  ? undefined
                  : themeSource),
    };
}

function normalizeList(payload: any): MarketplacePreset[] {
    if (!payload) return [];
    if (Array.isArray(payload)) return payload.map(normalizePreset);
    const list =
        payload.results ??
        payload.items ??
        payload.presets ??
        payload.data ??
        [];
    if (Array.isArray(list)) return list.map(normalizePreset);
    return [];
}

export class MarketplaceService {
    async listPresets(
        params: Record<string, any> = {}
    ): Promise<MarketplacePreset[]> {
        const data = await apiGet("/presets", { params });
        return normalizeList(data);
    }

    async getPreset(id: number | string): Promise<MarketplacePreset | null> {
        const data = await apiGet(`/presets/${id}`);
        return data ? normalizePreset(data) : null;
    }

    async createPreset(
        payload: Record<string, any>
    ): Promise<MarketplacePreset> {
        const data = await apiPost("/presets", payload);
        return normalizePreset(data);
    }

    async likePreset(id: number | string): Promise<any> {
        return apiPost(`/presets/${id}/like`);
    }

    async unlikePreset(id: number | string): Promise<any> {
        return apiPost(`/presets/${id}/unlike`);
    }

    async downloadPreset(id: number | string): Promise<any> {
        return apiPost(`/presets/${id}/download`);
    }

    async updatePreset(
        id: number | string,
        payload: Record<string, any>
    ): Promise<MarketplacePreset> {
        const data = await apiPatch(`/presets/${id}`, payload);
        return normalizePreset(data);
    }

    async deletePreset(id: number | string): Promise<void> {
        await apiDelete(`/presets/${id}`);
    }

    async listComments(presetId: number | string): Promise<any[]> {
        return apiGet(`/presets/${presetId}/comments`);
    }

    async addComment(presetId: number | string, text: string): Promise<any> {
        return apiPost(`/presets/${presetId}/comments`, { text });
    }

    async deleteComment(
        presetId: number | string,
        commentId: number | string
    ): Promise<any> {
        return apiDelete(`/presets/${presetId}/comments/${commentId}`);
    }
}

export const marketplaceService = new MarketplaceService();
