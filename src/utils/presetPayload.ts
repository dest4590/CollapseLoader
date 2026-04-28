import type { CreatePresetInput, MarketplaceTheme } from "../types/presets";

export function buildPresetCreatePayload(
    name: string,
    description: string | undefined,
    theme: MarketplaceTheme
): CreatePresetInput {
    const { customCSS = "", enableCustomCSS = false, ...colorFields } = theme;
    return { name, description, customCSS, enableCustomCSS, ...colorFields };
}
