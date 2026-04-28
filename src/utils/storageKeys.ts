export const STORAGE_KEYS = {
    AUTH_TOKEN: "authToken",
    THEME: "theme",
    LANGUAGE: "language",
    LOCAL_PROFILES: "local_profiles",
    ACTIVE_LOCAL_PROFILE: "active_local_profile",
    LOCAL_ACHIEVEMENTS: "local_achievements",
    LOCAL_USER_STATS: "local_user_stats",
    LOCAL_UNIQUE_FAVORITES: "local_unique_favorites",
    USER_DATA: "userData",
    STREAMER_MODE: "streamerModeEnabled",
    READ_NEWS: "readNews",
} as const;

export type StorageKey = (typeof STORAGE_KEYS)[keyof typeof STORAGE_KEYS];
