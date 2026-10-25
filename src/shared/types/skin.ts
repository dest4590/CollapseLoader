export type SkinVariant = "classic" | "slim";

export interface SkinCape {
    id: string;
    url: string;
    alias?: string | null;
    active: boolean;
}

export interface SkinProfile {
    id: string;
    name: string;
    skinUrl: string | null;
    skinVariant: string;
    capes: SkinCape[];
}

export interface SavedSkin {
    id: string;
    name: string;
    variant: SkinVariant;
    textureUrl: string;
    textureHash: string;
    createdAt: string;
}

export interface CatalogSkinEntry {
    id: string;
    name: string;
    section: string;
    variant: SkinVariant;
    textureData: string;
}
