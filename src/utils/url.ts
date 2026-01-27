import { getApiUrl } from '../config';

const ABSOLUTE_URL_REGEX = /^[a-z][a-z0-9+.-]*:/i;

export function resolveApiAssetUrl(url?: string | null): string | undefined {
    if (!url) return undefined;
    if (ABSOLUTE_URL_REGEX.test(url)) return url;

    const baseUrl = getApiUrl();
    if (!baseUrl) return url;

    if (url.startsWith('/')) {
        return `${baseUrl}${url}`;
    }

    return `${baseUrl}/${url}`;
}
