import { type UserProfile, type UserInfo } from "./userService";

export interface LocalUser {
    id: string;
    username: string;
    nickname: string | null;
    avatarUrl: string | null;
    role: string;
    createdAt: string;
}

class LocalUserService {
    private storageKey = "local_profiles";
    private activeProfileKey = "active_local_profile";

    getProfiles(): LocalUser[] {
        const stored = localStorage.getItem(this.storageKey);
        return stored ? JSON.parse(stored) : [];
    }

    getActiveProfile(): LocalUser | null {
        const profileId = localStorage.getItem(this.activeProfileKey);
        if (!profileId) return null;
        return this.getProfiles().find((p) => p.id === profileId) || null;
    }

    createProfile(username: string): LocalUser {
        const profiles = this.getProfiles();
        const newProfile: LocalUser = {
            id: crypto.randomUUID(),
            username: username,
            nickname: username,
            avatarUrl: null,
            role: "LOCAL_USER",
            createdAt: new Date().toISOString(),
        };
        profiles.push(newProfile);
        localStorage.setItem(this.storageKey, JSON.stringify(profiles));
        this.setActiveProfile(newProfile.id);
        return newProfile;
    }

    updateProfile(id: string, updates: Partial<LocalUser>): LocalUser | null {
        const profiles = this.getProfiles();
        const index = profiles.findIndex((p) => p.id === id);
        if (index === -1) return null;

        profiles[index] = { ...profiles[index], ...updates };
        localStorage.setItem(this.storageKey, JSON.stringify(profiles));
        return profiles[index];
    }

    setActiveProfile(id: string) {
        const profile = this.getProfiles().find((p) => p.id === id);
        if (profile) {
            const userData = {
                profile: {
                    id: 1,
                    nickname: profile.nickname,
                    avatar_url: profile.avatarUrl,
                    role: profile.role,
                    social_links: [],
                    created_at: profile.createdAt,
                    updated_at: new Date().toISOString(),
                    favorite_client_id: null,
                } as UserProfile,
                info: {
                    id: 1,
                    username: profile.username,
                    email: "local@user.none",
                    role: profile.role,
                    created_at: profile.createdAt,
                    updated_at: new Date().toISOString(),
                    last_login_at: new Date().toISOString(),
                } as UserInfo,
                lastUpdated: new Date().toISOString(),
            };
            localStorage.setItem("userData", JSON.stringify(userData));
        }

        localStorage.setItem(this.activeProfileKey, id);
        localStorage.setItem("authToken", `local_${id}`);
    }

    logout() {
        localStorage.removeItem(this.activeProfileKey);
        localStorage.removeItem("authToken");
        localStorage.removeItem("userData");
    }
}

export const localUserService = new LocalUserService();
