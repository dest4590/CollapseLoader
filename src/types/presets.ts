export interface ThemePreset {
    id: string;
    name: string;
    description?: string;
    created_at: string;
    custom_css: string;
    enable_custom_css: boolean;
    primary?: string;

    base100?: string;
    base200?: string;
    base300?: string;
    base_content?: string;

    primary_content?: string;
    secondary?: string;
    secondary_content?: string;
    accent?: string;
    accent_content?: string;
    neutral?: string;
    neutral_content?: string;
    info?: string;
    info_content?: string;
    success?: string;
    success_content?: string;
    warning?: string;
    warning_content?: string;
    error?: string;
    error_content?: string;
}

export interface CreatePresetInput {
    name: string;
    description?: string;
    custom_css: string;
    enable_custom_css: boolean;
    
    base100?: string;
    base200?: string;
    base300?: string;
    base_content?: string;
    
    primary?: string;
    primary_content?: string;
    secondary?: string;
    secondary_content?: string;
    accent?: string;
    accent_content?: string;
    neutral?: string;
    neutral_content?: string;
    info?: string;
    info_content?: string;
    success?: string;
    success_content?: string;
    warning?: string;
    warning_content?: string;
    error?: string;
    error_content?: string;
}

export interface UpdatePresetInput extends CreatePresetInput {
    id: string;
}