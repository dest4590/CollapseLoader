import { invoke } from "@tauri-apps/api/core";
import type { MarketplacePreset, MarketplaceTheme } from "../types/presets";

const TG_CHANNEL = "CollapseTheme";
const TG_URL = `https://t.me/s/${TG_CHANNEL}`;
const CACHE_KEY = "tg_themes_cache";
const CACHE_TTL_MS = 5 * 60 * 1000;

interface CacheEntry {
    themes: MarketplacePreset[];
    fetchedAt: number;
}

function parseThemesFromHtml(html: string): MarketplacePreset[] {
    const results: MarketplacePreset[] = [];
    const allMsgBlocks = extractMessageBlocks(html);

    for (const rawHtml of allMsgBlocks) {
        const text = htmlToCleanText(rawHtml);
        const jsonCandidates = extractJsonObjects(text);

        for (const candidate of jsonCandidates) {
            const repaired = repairJsonStrings(candidate);
            try {
                const parsed = JSON.parse(repaired);
                if (isValidTheme(parsed)) {
                    results.push(normalizeTheme(parsed));
                }
            } catch {
                
            }
        }
    }

    return results;
}

function extractMessageBlocks(html: string): string[] {
    const blocks: string[] = [];
    const marker = 'tgme_widget_message_text';
    let pos = 0;

    while (pos < html.length) {
        const start = html.indexOf(marker, pos);
        if (start === -1) break;

        const openEnd = html.indexOf('>', start);
        if (openEnd === -1) break;

        let depth = 1;
        let i = openEnd + 1;
        while (i < html.length && depth > 0) {
            if (html[i] === '<') {
                if (html.slice(i, i + 2) === '</') {
                    const tagEnd = html.indexOf('>', i);
                    if (tagEnd !== -1) {
                        const tag = html.slice(i + 2, tagEnd).trim().split(/\s/)[0].toLowerCase();
                        if (tag === 'div') depth--;
                        i = tagEnd + 1;
                        continue;
                    }
                } else {
                    const tagEnd = html.indexOf('>', i);
                    if (tagEnd !== -1) {
                        const tagContent = html.slice(i + 1, tagEnd);
                        const tag = tagContent.trim().split(/[\s/]/)[0].toLowerCase();
                        const selfClosing = tagContent.trimEnd().endsWith('/');
                        if (tag === 'div' && !selfClosing) depth++;
                        i = tagEnd + 1;
                        continue;
                    }
                }
            }
            i++;
        }

        blocks.push(html.slice(openEnd + 1, i - (depth === 0 ? 6 : 0)));
        pos = openEnd + 1;
    }

    return blocks;
}

function htmlToCleanText(rawHtml: string): string {
    let text = rawHtml.replace(
        /<a\s[^>]*href="([^"]*)"[^>]*>([\s\S]*?)<\/a>/gi,
        (_match, href, inner) => {
            const decodedHref = decodeHtmlEntities(href);
            if (decodedHref.startsWith('?') || decodedHref.startsWith('#')) {
                return decodeHtmlEntities(stripTags(inner));
            }
            return decodedHref;
        }
    );

    text = text.replace(/<br\s*\/?>/gi, ' ');
    text = text.replace(/<[^>]+>/g, '');
    text = decodeHtmlEntities(text);

    return text.trim();
}

function stripTags(html: string): string {
    return html.replace(/<[^>]+>/g, '');
}

function decodeHtmlEntities(str: string): string {
    return str
        .replace(/&#092;/g, '\\')
        .replace(/&#33;/g, '!')
        .replace(/&#39;/g, "'")
        .replace(/&amp;/g, '&')
        .replace(/&lt;/g, '<')
        .replace(/&gt;/g, '>')
        .replace(/&quot;/g, '"')
        .replace(/&nbsp;/g, ' ')
        .replace(/&#(\d+);/g, (_, code) => String.fromCharCode(Number(code)))
        .replace(/&#x([0-9a-fA-F]+);/g, (_, hex) => String.fromCharCode(parseInt(hex, 16)));
}

function repairJsonStrings(json: string): string {
    let result = '';
    let inString = false;
    let escape = false;

    for (let i = 0; i < json.length; i++) {
        const ch = json[i];

        if (escape) {
            result += ch;
            escape = false;
            continue;
        }

        if (ch === '\\') {
            escape = true;
            result += ch;
            continue;
        }

        if (ch === '"') {
            inString = !inString;
            result += ch;
            continue;
        }

        if (inString && (ch === '\n' || ch === '\r')) {
            result += ' ';
        } else {
            result += ch;
        }
    }

    return result;
}

function extractJsonObjects(text: string): string[] {
    const results: string[] = [];
    let i = 0;
    while (i < text.length) {
        if (text[i] === '{') {
            let depth = 0;
            let j = i;
            let inString = false;
            let escape = false;
            while (j < text.length) {
                const ch = text[j];
                if (escape) {
                    escape = false;
                } else if (ch === '\\') {
                    escape = true;
                } else if (ch === '"') {
                    inString = !inString;
                } else if (!inString) {
                    if (ch === '{') depth++;
                    else if (ch === '}') {
                        depth--;
                        if (depth === 0) {
                            results.push(text.slice(i, j + 1));
                            i = j + 1;
                            break;
                        }
                    }
                }
                j++;
            }
            if (depth !== 0) i++;
        } else {
            i++;
        }
    }
    return results;
}

function isValidTheme(obj: any): boolean {
    if (!obj || typeof obj !== 'object') return false;
    return (
        typeof obj.name === 'string' &&
        obj.name.trim().length > 0 &&
        (obj.primary ||
            obj.base100 ||
            obj.customCSS ||
            obj.backgroundImage ||
            obj.enableCustomCSS !== undefined)
    );
}

function normalizeTheme(raw: any): MarketplacePreset {
    const theme: MarketplaceTheme = {
        customCSS: raw.customCSS ?? raw.custom_css ?? '',
        enableCustomCSS: raw.enableCustomCSS ?? raw.enable_custom_css ?? false,
        primary: raw.primary,
        primaryContent: raw.primaryContent ?? raw.primary_content,
        base100: raw.base100,
        base200: raw.base200,
        base300: raw.base300,
        baseContent: raw.baseContent ?? raw.base_content,
        secondary: raw.secondary,
        secondaryContent: raw.secondaryContent ?? raw.secondary_content,
        accent: raw.accent,
        accentContent: raw.accentContent ?? raw.accent_content,
        neutral: raw.neutral,
        neutralContent: raw.neutralContent ?? raw.neutral_content,
        info: raw.info,
        infoContent: raw.infoContent ?? raw.info_content,
        success: raw.success,
        successContent: raw.successContent ?? raw.success_content,
        warning: raw.warning,
        warningContent: raw.warningContent ?? raw.warning_content,
        error: raw.error,
        errorContent: raw.errorContent ?? raw.error_content,
        backgroundImage: raw.backgroundImage ?? raw.background_image,
        backgroundBlur:
            raw.backgroundBlur != null ? Number(raw.backgroundBlur) : undefined,
        backgroundOpacity:
            raw.backgroundOpacity != null
                ? Number(raw.backgroundOpacity)
                : undefined,
    };

    return {
        id: raw.id ?? `tg-${raw.name}-${raw.createdAt ?? Date.now()}`,
        name: raw.name,
        title: raw.name,
        description: raw.description ?? null,
        is_public: true,
        likes_count: 0,
        downloads_count: 0,
        comments_count: 0,
        liked: false,
        author: { username: TG_CHANNEL, displayName: `@${TG_CHANNEL}` },
        owner_username: TG_CHANNEL,
        theme,
        preset_data: theme,
    } as MarketplacePreset;
}

class TelegramThemeService {
    private cache: CacheEntry | null = null;

    constructor() {
        try {
            const raw = localStorage.getItem(CACHE_KEY);
            if (raw) {
                const entry: CacheEntry = JSON.parse(raw);
                if (Date.now() - entry.fetchedAt < CACHE_TTL_MS) {
                    this.cache = entry;
                }
            }
        } catch {
            
        }
    }

    async fetchThemes(forceRefresh = false): Promise<MarketplacePreset[]> {
        if (!forceRefresh && this.cache) {
            return this.cache.themes;
        }

        try {
            const allThemes: MarketplacePreset[] = [];
            let url = TG_URL;
            const visited = new Set<string>();

            while (url && !visited.has(url)) {
                visited.add(url);

                const html = await invoke<string>("api_request", {
                    method: "GET",
                    url,
                    headers: {
                        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
                        "Accept": "text/html,application/xhtml+xml",
                    },
                    body: null,
                });

                const htmlStr = typeof html === "string" ? html : JSON.stringify(html);
                const pageThemes = parseThemesFromHtml(htmlStr);
                allThemes.push(...pageThemes);

                const msgIds = [...htmlStr.matchAll(/data-post="[^/]+\/(\d+)"/g)]
                    .map(m => parseInt(m[1]))
                    .filter(n => !isNaN(n));

                if (msgIds.length === 0) break;

                const minId = Math.min(...msgIds);
                if (minId <= 1) break;

                url = `${TG_URL}?before=${minId}`;
            }

            const seen = new Set<string>();
            const themes = allThemes.filter(t => {
                const key = String(t.id);
                if (seen.has(key)) return false;
                seen.add(key);
                return true;
            });

            const entry: CacheEntry = { themes, fetchedAt: Date.now() };
            this.cache = entry;
            try {
                localStorage.setItem(CACHE_KEY, JSON.stringify(entry));
            } catch {
                // ignore
            }

            return themes;
        } catch (err) {
            console.error('[TelegramThemeService] Failed to fetch themes:', err);
            return this.cache?.themes ?? [];
        }
    }

    clearCache() {
        this.cache = null;
        localStorage.removeItem(CACHE_KEY);
    }
}

export const telegramThemeService = new TelegramThemeService();
