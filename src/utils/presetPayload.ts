import type { CreatePresetInput, MarketplaceTheme } from "../types/presets";

export function buildPresetCreatePayload(
    name: string,
    description: string | undefined,
    theme: MarketplaceTheme
): CreatePresetInput {
    return {
        name,
        description,
        customCSS: theme.customCSS ?? "",
        enableCustomCSS: theme.enableCustomCSS ?? false,
        base100: theme.base100,
        base200: theme.base200,
        base300: theme.base300,
        baseContent: theme.baseContent,
        primary: theme.primary,
        primaryContent: theme.primaryContent,
        secondary: theme.secondary,
        secondaryContent: theme.secondaryContent,
        accent: theme.accent,
        accentContent: theme.accentContent,
        neutral: theme.neutral,
        neutralContent: theme.neutralContent,
        info: theme.info,
        infoContent: theme.infoContent,
        success: theme.success,
        successContent: theme.successContent,
        warning: theme.warning,
        warningContent: theme.warningContent,
        error: theme.error,
        errorContent: theme.errorContent,
        backgroundImage: theme.backgroundImage,
        backgroundBlur: theme.backgroundBlur,
        backgroundOpacity: theme.backgroundOpacity,
    };
}
