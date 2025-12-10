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
}

export interface UpdatePresetInput extends CreatePresetInput {
    id: string;
}