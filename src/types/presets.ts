export interface ThemePreset {
    id: string;
    name: string;
    description?: string;
    createdAt: string;
    customCSS: string;
    enableCustomCSS: boolean;
    primary?: string;

    base100?: string;
    base200?: string;
    base300?: string;
    baseContent?: string;

    primaryContent?: string;
    secondary?: string;
    secondaryContent?: string;
    accent?: string;
    accentContent?: string;
    neutral?: string;
    neutralContent?: string;
    info?: string;
    infoContent?: string;
    success?: string;
    successContent?: string;
    warning?: string;
    warningContent?: string;
    error?: string;
    errorContent?: string;
    backgroundImage?: string;
    backgroundBlur?: number;
    backgroundOpacity?: number;
}

export interface CreatePresetInput {
    name: string;
    description?: string;
    customCSS: string;
    enableCustomCSS: boolean;

    base100?: string;
    base200?: string;
    base300?: string;
    baseContent?: string;

    primary?: string;
    primaryContent?: string;
    secondary?: string;
    secondaryContent?: string;
    accent?: string;
    accentContent?: string;
    neutral?: string;
    neutralContent?: string;
    info?: string;
    infoContent?: string;
    success?: string;
    successContent?: string;
    warning?: string;
    warningContent?: string;
    error?: string;
    errorContent?: string;
    backgroundImage?: string;
    backgroundBlur?: number;
    backgroundOpacity?: number;
}

export interface UpdatePresetInput extends CreatePresetInput {
    id: string;
}

export interface MarketplaceTheme {
    customCSS?: string;
    enableCustomCSS?: boolean;
    base100?: string;
    base200?: string;
    base300?: string;
    baseContent?: string;
    primary?: string;
    primaryContent?: string;
    secondary?: string;
    secondaryContent?: string;
    accent?: string;
    accentContent?: string;
    neutral?: string;
    neutralContent?: string;
    info?: string;
    infoContent?: string;
    success?: string;
    successContent?: string;
    warning?: string;
    warningContent?: string;
    error?: string;
    errorContent?: string;
    backgroundImage?: string;
    backgroundBlur?: number;
    backgroundOpacity?: number;
}

export interface MarketplaceAuthor {
    id?: number | string;
    username?: string;
    displayName?: string;
    name?: string;
    [key: string]: any;
}

export interface MarketplacePreset {
    id: number | string;
    name?: string;
    title?: string;
    description?: string;
    created_at?: string;
    is_public?: boolean;
    likes_count?: number;
    downloads_count?: number;
    comments_count?: number;
    liked?: boolean;
    owner_username?: string;
    author?: MarketplaceAuthor;
    theme?: MarketplaceTheme;
    preset_data?: Record<string, any>;
    [key: string]: any;
}
