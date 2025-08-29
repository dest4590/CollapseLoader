import i18n from '../i18n';

export type Category = 'feature' | 'improvement' | 'bugfix' | 'other';

export interface ChangeItem {
    category: Category;
    description_key: string;
    icon?: string;
}

export interface ChangelogEntry {
    version: string;
    changes: ChangeItem[];
    date?: string;
    highlights?: string[];
}

const CATEGORY_KEYS: Record<Category, string> = {
    feature: 'changelog.category.feature',
    improvement: 'changelog.category.improvement',
    bugfix: 'changelog.category.bugfix',
    other: 'changelog.category.other',
};

export function resolveDescription(keyOrText: string) {
    if (!keyOrText) return '';
    if (keyOrText.includes('.')) {
        return i18n.global.t(keyOrText) as string;
    }
    return keyOrText;
}

export function parseChangelog(raw: any): ChangelogEntry[] {
    if (!raw) return [];

    if (!Array.isArray(raw)) return [];

    return raw.map((entry: any) => {
        const changes: ChangeItem[] = Array.isArray(entry.changes)
            ? entry.changes.map((c: any) => ({
                category: (c.category || 'other').toLowerCase() as Category,
                description_key: c.description_key || c.description || c.text || '',
                icon: c.icon || (c.category === 'Feature' ? '✨' : undefined),
            }))
            : [];

        return {
            version: entry.version || entry.tag || '',
            changes,
            date: entry.date || entry.published_at || '',
            highlights: Array.isArray(entry.highlights) ? entry.highlights : [],
        } as ChangelogEntry;
    });
}

export function renderChange(change: ChangeItem) {
    return {
        icon: change.icon || defaultIconForCategory(change.category),
        category: i18n.global.t(CATEGORY_KEYS[change.category]) as string,
        description: resolveDescription(change.description_key),
    };
}

function defaultIconForCategory(category: Category) {
    switch (category) {
        case 'feature':
            return '✨';
        case 'improvement':
            return '🔧';
        case 'bugfix':
            return '🐛';
        default:
            return '🔹';
    }
}

export default {
    parseChangelog,
    renderChange,
    resolveDescription,
};
