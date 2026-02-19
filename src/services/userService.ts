import { apiClient } from "./apiClient";

type FriendshipStatus =
    | "friends"
    | "request_sent"
    | "request_received"
    | "blocked"
    | null;

export interface SocialLink {
    platform: string;
    url: string;
}

export interface UserProfile {
    id: number;
    nickname: string | null;
    avatar_url: string | null;
    role: string | null;
    social_links: SocialLink[];
    created_at: string;
    updated_at: string;
    favorite_client_id: number | null;
}

export interface UserInitData {
    user: {
        id: number;
        username: string;
        email: string;
        role: string;
        created_at: string;
        updated_at: string;
        last_login_at: string | null;
        profile: UserProfile;
        status: UserStatus;
    };
    preferences: UserPreference[];
    favorites: UserFavorite[];
    accounts: UserExternalAccount[];
    friends: {
        friends: any[];
        requests: {
            sent: any[];
            received: any[];
            blocked: any[];
        };
    };
}

export interface UserInfo {
    id: number;
    username: string;
    email: string | null;
    role: string;
    created_at: string;
    updated_at: string;
    last_login_at: string | null;
}

interface CachedUserData {
    profile: UserProfile | null;
    info: UserInfo | null;
    lastUpdated: string;
}

export interface UserStatus {
    status: string;
    client_name: string | null;
    updated_at: string | null;
    started_at?: string | null;
}

export interface ClientUserStatus {
    is_online: boolean;
    last_seen: string | null;
    current_client: string | null;
    client_version?: string | null;
    updated_at: string | null;
    started_at: string | null;
    raw_status: string | null;
}

export interface Friend {
    id: number;
    username: string;
    nickname: string | null;
    avatar_url: string | null;
    status: ClientUserStatus;
}

export interface FriendRequest {
    id: number;
    requester: Friend;
    addressee: Friend;
    status: FriendshipStatus | string;
    created_at: string;
    updated_at: string;
}

export interface FriendRequestsBatch {
    sent: FriendRequest[];
    received: FriendRequest[];
    blocked: FriendRequest[];
}

export interface SearchUser {
    id: number;
    username: string;
    nickname: string | null;
    avatar_url: string | null;
    friendship_status: FriendshipStatus;
}

export interface PublicUserProfile {
    id: number;
    username: string;
    nickname: string | null;
    friendship_status: FriendshipStatus;
    status: ClientUserStatus;
    member_since: string | null;
    avatar_url: string | null;
    social_links: SocialLink[];
    role: string | null;
    achievements?: any[];
    presets?: any[];
    favorite_client_id: number | null;
}

export interface UserPreference {
    key: string;
    value: unknown;
    created_at: string;
    updated_at: string;
}

export interface UserFavorite {
    id: number;
    type: string;
    reference: string;
    metadata: unknown | null;
    created_at: string;
}

export interface UserExternalAccount {
    id: number;
    provider: string;
    external_id: string;
    display_name: string | null;
    metadata: unknown | null;
    created_at: string;
    updated_at: string;
}

export interface SyncData {
    settings_data: any;
    favorites_data: number[];
    accounts_data: any[];
    last_sync_timestamp?: string | null;
}

export interface SyncStatus {
    last_sync_timestamp: string | null;
    has_cloud_data: boolean;
}

export interface UserInitData {
    user: {
        id: number;
        username: string;
        email: string;
        role: string;
        created_at: string;
        updated_at: string;
        last_login_at: string | null;
        profile: UserProfile;
        status: UserStatus;
    };
    preferences: UserPreference[];
    favorites: UserFavorite[];
    accounts: UserExternalAccount[];
    friends: {
        friends: any[];
        requests: {
            sent: any[];
            received: any[];
            blocked: any[];
        };
    };
}

const CACHE_KEY = "userData";
const CACHE_EXPIRY_HOURS = 24;

const SYNC_SETTINGS_PREF_KEY = "collapseloader.settings";
const SYNC_FAVORITE_TYPE = "client";
const SYNC_ACCOUNT_PROVIDER = "collapseloader";

class UserService {
    private getCachedData(): CachedUserData | null {
        try {
            const cached = localStorage.getItem(CACHE_KEY);
            if (!cached) return null;
            const parsedData: any = JSON.parse(cached);
            if (
                !parsedData ||
                typeof parsedData !== "object" ||
                !parsedData.lastUpdated
            ) {
                localStorage.removeItem(CACHE_KEY);
                return null;
            }
            const now = new Date();
            const cacheTime = new Date(parsedData.lastUpdated);
            if (isNaN(cacheTime.getTime())) {
                localStorage.removeItem(CACHE_KEY);
                return null;
            }
            const hoursDiff =
                (now.getTime() - cacheTime.getTime()) / (1000 * 60 * 60);
            if (hoursDiff > CACHE_EXPIRY_HOURS) {
                localStorage.removeItem(CACHE_KEY);
                return null;
            }
            return parsedData as CachedUserData;
        } catch (error) {
            console.error("Error reading cached user data:", error);
            localStorage.removeItem(CACHE_KEY);
            return null;
        }
    }

    private setCachedData(data: Partial<CachedUserData>): void {
        try {
            const existing = this.getCachedData() || {
                profile: null,
                info: null,
                lastUpdated: new Date().toISOString(),
            };
            const updated = {
                ...existing,
                ...data,
                lastUpdated: new Date().toISOString(),
            };
            localStorage.setItem(CACHE_KEY, JSON.stringify(updated));
        } catch (error) {
            console.error("Error caching user data:", error);
        }
    }

    private stripShowFields(input: any): any {
        if (input === null || input === undefined) return input;
        if (Array.isArray(input))
            return input.map((v) => this.stripShowFields(v));
        if (typeof input === "object") {
            const out: any = {};
            for (const k of Object.keys(input)) {
                if (k === "show") continue;
                out[k] = this.stripShowFields((input as any)[k]);
            }
            return out;
        }
        return input;
    }

    private mapStatus(status: UserStatus | null | undefined): ClientUserStatus {
        const raw = status?.status ?? null;
        const normalized = (raw || "").toUpperCase();
        const isOnline =
            normalized.length > 0 &&
            normalized !== "OFFLINE" &&
            normalized !== "INVISIBLE";
        const updatedAt = status?.updated_at ?? null;
        return {
            is_online: isOnline,
            current_client: status?.client_name ?? null,
            last_seen: isOnline ? null : updatedAt,
            updated_at: updatedAt,
            started_at: status?.started_at ?? null,
            raw_status: raw,
        };
    }

    public mapFriend(friend: any): Friend {
        return {
            id: friend.id,
            username: friend.username,
            nickname: friend.nickname ?? null,
            avatar_url: friend.avatar_url ?? null,
            status: this.mapStatus(friend.status),
        };
    }

    public mapFriendRequest(request: any): FriendRequest {
        return {
            id: request.id,
            requester: this.mapFriend(request.requester),
            addressee: this.mapFriend(request.addressee),
            status: request.status,
            created_at: request.created_at,
            updated_at: request.updated_at,
        };
    }

    private maxIsoTimestamp(
        timestamps: Array<string | null | undefined>
    ): string | null {
        let max: number | null = null;
        let maxIso: string | null = null;
        for (const ts of timestamps) {
            if (!ts) continue;
            const time = new Date(ts).getTime();
            if (Number.isNaN(time)) continue;
            if (max === null || time > max) {
                max = time;
                maxIso = ts;
            }
        }
        return maxIso;
    }

    private async getCurrentUserId(): Promise<number | null> {
        const cached = this.getCachedData();
        if (cached?.info?.id) return cached.info.id;
        try {
            const me = await apiClient.get<any>("/users/me");
            return typeof me?.id === "number" ? me.id : null;
        } catch {
            return null;
        }
    }

    async updateUserProfile(
        nickname: string | null,
        favoriteClientId?: number | null
    ): Promise<{ success: boolean; error?: string }> {
        try {
            const payload: any = { nickname };
            if (favoriteClientId !== undefined) {
                payload.favorite_client_id = favoriteClientId;
            }
            await apiClient.patch("/users/me/profile", payload);
            await this.refreshCachedUser();
            return { success: true };
        } catch (error: any) {
            console.error("Failed to update user profile:", error);
            const errorMessage =
                error.response?.data?.error || "Failed to update profile";
            return { success: false, error: errorMessage };
        }
    }

    async uploadAvatar(
        file: File
    ): Promise<{ success: boolean; profile?: UserProfile; error?: string }> {
        try {
            const form = new FormData();
            form.append("avatar", file);
            await apiClient.post("/users/me/avatar", form);
            const profile = await this.refreshCachedUser();
            return { success: true, profile };
        } catch (error: any) {
            const errorMessage =
                error.response?.data?.error || "Failed to upload avatar";
            return { success: false, error: errorMessage };
        }
    }

    async resetAvatar(): Promise<{
        success: boolean;
        profile?: UserProfile;
        error?: string;
    }> {
        try {
            await apiClient.post("/users/me/avatar/reset");
            const profile = await this.refreshCachedUser();
            return { success: true, profile };
        } catch (error: any) {
            const errorMessage =
                error.response?.data?.error || "Failed to reset avatar";
            return { success: false, error: errorMessage };
        }
    }

    async getUserStatus(): Promise<UserStatus> {
        try {
            return await apiClient.get("/users/me/status");
        } catch (error) {
            console.error("Failed to get user status:", error);
            throw error;
        }
    }

    async changePassword(
        currentPassword: string,
        newPassword: string
    ): Promise<{ success: boolean }> {
        try {
            await apiClient.post("/auth/setPassword", {
                newPassword: newPassword,
                currentPassword: currentPassword,
            });
            return { success: true };
        } catch (error) {
            console.error("Failed to change password:", error);
            throw error;
        }
    }

    async initializeUser(): Promise<{
        profile: UserProfile;
        user_info: UserInfo;
        status: UserStatus;
    }> {
        try {
            const data = await apiClient.get<any>("/users/me");
            const result = {
                profile: data.profile,
                user_info: {
                    id: data.id,
                    username: data.username,
                    email: data.email,
                    role: data.role,
                    created_at: data.created_at,
                    updated_at: data.updated_at,
                    last_login_at: data.last_login_at ?? null,
                },
                status: data.status,
            };
            this.setCachedData({
                profile: result.profile,
                info: result.user_info as UserInfo,
            });
            return result;
        } catch (error) {
            console.error("Failed to initialize user:", error);
            throw error;
        }
    }

    async initializeUserFull(): Promise<UserInitData> {
        try {
            const data = await apiClient.get<UserInitData>("/users/init");
            const userInfo = {
                id: data.user.id,
                username: data.user.username,
                email: data.user.email,
                role: data.user.role,
                created_at: data.user.created_at,
                updated_at: data.user.updated_at,
                last_login_at: data.user.last_login_at ?? null,
            };
            this.setCachedData({
                profile: data.user.profile,
                info: userInfo as UserInfo,
            });
            return data;
        } catch (error) {
            console.error("Failed to initialize user (full):", error);
            throw error;
        }
    }

    async getUserProfile(
        userId: number,
        include: string[] = []
    ): Promise<PublicUserProfile> {
        const query = include.length > 0 ? `?include=${include.join(",")}` : "";
        const publicUser = await apiClient.get<any>(`/users/${userId}${query}`);

        const profile = publicUser.profile || null;
        const base: PublicUserProfile = {
            id: publicUser.id,
            username: publicUser.username,
            nickname: profile?.nickname ?? null,
            friendship_status: null,
            status: this.mapStatus(publicUser.status),
            member_since: profile?.created_at ?? null,
            avatar_url: profile?.avatar_url ?? null,
            social_links: profile?.social_links ?? [],
            role: profile?.role ?? null,
            achievements: publicUser.achievements,
            presets: publicUser.presets,
            favorite_client_id: profile?.favorite_client_id ?? null,
        };

        if (publicUser.friendship_status) {
            return { ...base, friendship_status: publicUser.friendship_status };
        }

        const token = localStorage.getItem("authToken");
        if (!token) return base;

        try {
            const batch = await this.getFriendsBatch();
            const isFriend = batch.friends.some((f) => f.id === userId);
            if (isFriend) return { ...base, friendship_status: "friends" };

            const sent = batch.requests.sent.find(
                (r) => r.addressee.id === userId
            );
            if (sent) {
                if ((sent.status || "").toString() === "blocked")
                    return { ...base, friendship_status: "blocked" };
                return { ...base, friendship_status: "request_sent" };
            }

            const received = batch.requests.received.find(
                (r) => r.requester.id === userId
            );
            if (received) {
                if ((received.status || "").toString() === "blocked")
                    return { ...base, friendship_status: "blocked" };
                return { ...base, friendship_status: "request_received" };
            }

            const anyBlocked =
                batch.requests.sent.some(
                    (r) => r.status === "blocked" && r.addressee.id === userId
                ) ||
                batch.requests.received.some(
                    (r) => r.status === "blocked" && r.requester.id === userId
                );
            return {
                ...base,
                friendship_status: anyBlocked ? "blocked" : null,
            };
        } catch {
            return base;
        }
    }

    async searchUsers(query: string, limit = 20): Promise<SearchUser[]> {
        const resp = await apiClient.get<any[]>("/users/search", {
            params: { q: query, limit },
        });
        return (resp || []).map((u: any) => ({
            id: u.id,
            username: u.username,
            nickname: u.nickname ?? null,
            avatar_url: u.avatar_url ?? null,
            friendship_status: (u.friendship_status ?? null) as FriendshipStatus,
        }));
    }

    async getFriendsBatch(): Promise<{
        friends: Friend[];
        requests: FriendRequestsBatch;
    }> {
        const resp = await apiClient.get<any>("/friends/batch");

        const friends: Friend[] = (resp.friends || []).map((f: any) =>
            this.mapFriend(f)
        );
        const requests: FriendRequestsBatch = {
            sent: (resp.requests?.sent || []).map((r: any) =>
                this.mapFriendRequest(r)
            ),
            received: (resp.requests?.received || []).map((r: any) =>
                this.mapFriendRequest(r)
            ),
            blocked: (resp.requests?.blocked || []).map((r: any) =>
                this.mapFriendRequest(r)
            ),
        };

        return { friends, requests };
    }

    async getFriends(): Promise<Friend[]> {
        const batch = await this.getFriendsBatch();
        return batch.friends;
    }

    async getFriendRequests(): Promise<FriendRequestsBatch> {
        const batch = await this.getFriendsBatch();
        return batch.requests;
    }

    async sendFriendRequest(userId: number): Promise<FriendRequest> {
        const resp = await apiClient.post<any>("/friends/requests", {
            user_id: userId,
        });
        const batch = await this.getFriendsBatch();
        const created =
            batch.requests.sent.find((r) => r.id === resp?.id) ||
            batch.requests.sent.find((r) => r.addressee.id === userId);
        if (created) return created;
        return this.mapFriendRequest(resp);
    }

    async respondToFriendRequest(
        requestId: number,
        action: "accept" | "reject"
    ): Promise<void> {
        if (action === "accept") {
            await apiClient.post(`/friends/requests/${requestId}/accept`);
        } else {
            await apiClient.post(`/friends/requests/${requestId}/decline`);
        }
    }

    async cancelFriendRequest(requestId: number): Promise<void> {
        await apiClient.post(`/friends/requests/${requestId}/decline`);
    }

    async blockUser(userId: number): Promise<void> {
        await apiClient.post("/friends/block", { user_id: userId });
    }

    async unblockUser(userId: number): Promise<void> {
        await apiClient.post("/friends/unblock", { user_id: userId });
    }

    async getBlockedUsers(): Promise<Friend[]> {
        const meId = await this.getCurrentUserId();
        const batch = await this.getFriendsBatch();

        const blocked: Friend[] = [];
        const seen = new Set<number>();
        const add = (f: Friend) => {
            if (seen.has(f.id)) return;
            seen.add(f.id);
            blocked.push(f);
        };

        const consider = (r: FriendRequest) => {
            if (meId && r.requester.id === meId) add(r.addressee);
            else if (meId && r.addressee.id === meId) add(r.requester);
            else {
                add(r.requester);
                add(r.addressee);
            }
        };

        batch.requests.blocked.forEach(consider);

        return blocked;
    }

    async removeFriend(userId: number): Promise<void> {
        await apiClient.delete(`/friends/${userId}`);
    }

    async getSocialLinks(): Promise<SocialLink[]> {
        return await apiClient.get("/users/me/social-links");
    }

    async updateSocialLinks(links: SocialLink[]): Promise<SocialLink[]> {
        return await apiClient.put("/users/me/social-links", { links });
    }

    async getPreferences(): Promise<UserPreference[]> {
        return await apiClient.get("/users/me/preferences");
    }

    async setPreference(key: string, value: unknown): Promise<UserPreference> {
        const safeKey = encodeURIComponent(key);
        let payloadValue: unknown = value;
        try {
            if (
                key === SYNC_SETTINGS_PREF_KEY &&
                value &&
                typeof value === "object"
            ) {
                payloadValue = this.stripShowFields(value);
            }
        } catch (err) {
            console.warn(
                "Failed to strip show fields from settings payload:",
                err
            );
        }

        return await apiClient.put(`/users/me/preferences/${safeKey}`, {
            value: payloadValue,
        });
    }

    async deletePreference(key: string): Promise<void> {
        const safeKey = encodeURIComponent(key);
        await apiClient.delete(`/users/me/preferences/${safeKey}`);
    }

    async getFavorites(): Promise<UserFavorite[]> {
        return await apiClient.get("/users/me/favorites");
    }

    async addFavorite(
        favorite: Omit<UserFavorite, "id" | "created_at">
    ): Promise<UserFavorite> {
        return await apiClient.post("/users/me/favorites", favorite);
    }

    async deleteFavorite(favoriteId: number): Promise<void> {
        await apiClient.delete(`/users/me/favorites/${favoriteId}`);
    }

    async getExternalAccounts(): Promise<UserExternalAccount[]> {
        return await apiClient.get("/users/me/accounts");
    }

    async addExternalAccount(
        account: Omit<UserExternalAccount, "id" | "created_at" | "updated_at">
    ): Promise<UserExternalAccount> {
        return await apiClient.post("/users/me/accounts", account);
    }

    async deleteExternalAccount(accountId: number): Promise<void> {
        await apiClient.delete(`/users/me/accounts/${accountId}`);
    }

    async getSyncStatus(): Promise<SyncStatus | null> {
        try {
            const [preferences, favorites, accounts] = await Promise.all([
                this.getPreferences(),
                this.getFavorites(),
                this.getExternalAccounts(),
            ]);

            const settingsPref =
                preferences.find((p) => p.key === SYNC_SETTINGS_PREF_KEY) ||
                null;

            const lastSync = this.maxIsoTimestamp([
                settingsPref?.updated_at ?? null,
                ...favorites.map((f) => f.created_at ?? null),
                ...accounts.map((a) => a.updated_at ?? null),
            ]);

            return {
                last_sync_timestamp: lastSync,
                has_cloud_data:
                    !!settingsPref ||
                    favorites.length > 0 ||
                    accounts.length > 0,
            };
        } catch (error) {
            console.error("Failed to get sync status:", error);
            return null;
        }
    }

    async syncToCloud(syncData: SyncData): Promise<void> {
        await this.setPreference(
            SYNC_SETTINGS_PREF_KEY,
            syncData.settings_data ?? {}
        );

        const [remoteFavorites, remoteAccounts] = await Promise.all([
            this.getFavorites(),
            this.getExternalAccounts(),
        ]);

        const localFavoriteRefs = new Set(
            (syncData.favorites_data || []).map((id) => String(id))
        );
        const remoteClientFavorites = remoteFavorites.filter(
            (f) => f.type === SYNC_FAVORITE_TYPE
        );

        await Promise.all(
            remoteClientFavorites
                .filter((f) => !localFavoriteRefs.has(f.reference))
                .map((f) => this.deleteFavorite(f.id))
        );

        const remoteFavoriteRefs = new Set(
            remoteClientFavorites.map((f) => f.reference)
        );
        for (const favRef of localFavoriteRefs) {
            if (remoteFavoriteRefs.has(favRef)) continue;
            await this.addFavorite({
                type: SYNC_FAVORITE_TYPE,
                reference: favRef,
                metadata: null,
            });
        }

        const localAccounts = (syncData.accounts_data || [])
            .map((acc: any) => ({
                username: acc?.username,
                tags: acc?.tags,
            }))
            .filter(
                (acc) =>
                    typeof acc.username === "string" && acc.username.length > 0
            );

        const localAccountIds = new Set(localAccounts.map((a) => a.username));
        const remoteManagedAccounts = remoteAccounts.filter(
            (a) => a.provider === SYNC_ACCOUNT_PROVIDER
        );

        await Promise.all(
            remoteManagedAccounts
                .filter((a) => !localAccountIds.has(a.external_id))
                .map((a) => this.deleteExternalAccount(a.id))
        );

        const remoteAccountIds = new Set(
            remoteManagedAccounts.map((a) => a.external_id)
        );
        for (const acc of localAccounts) {
            if (remoteAccountIds.has(acc.username)) continue;
            await this.addExternalAccount({
                provider: SYNC_ACCOUNT_PROVIDER,
                external_id: acc.username,
                display_name: acc.username,
                metadata: { tags: acc.tags || [] },
            });
        }
    }

    formatSyncData(
        preferences: UserPreference[],
        favorites: UserFavorite[],
        accounts: UserExternalAccount[]
    ): SyncData {
        const settingsPref =
            preferences.find((p) => p.key === SYNC_SETTINGS_PREF_KEY) || null;

        const favorites_data = (favorites || [])
            .filter((f) => f.type === SYNC_FAVORITE_TYPE)
            .map((f) => Number.parseInt(String(f.reference), 10))
            .filter((n) => Number.isFinite(n));

        const accounts_data = (accounts || [])
            .filter((a) => a.provider === SYNC_ACCOUNT_PROVIDER)
            .map((a) => {
                const meta: any =
                    a.metadata && typeof a.metadata === "object"
                        ? a.metadata
                        : {};
                return {
                    username: a.external_id,
                    tags: Array.isArray(meta.tags) ? meta.tags : ["cloud-sync"],
                };
            });

        const last_sync_timestamp = this.maxIsoTimestamp([
            settingsPref?.updated_at ?? null,
            ...favorites.map((f) => f.created_at ?? null),
            ...accounts.map((a) => a.updated_at ?? null),
        ]);

        return {
            settings_data: (settingsPref?.value ?? {}) as any,
            favorites_data,
            accounts_data,
            last_sync_timestamp,
        };
    }

    async downloadFromCloud(): Promise<SyncData | null> {
        try {
            const [preferences, favorites, accounts] = await Promise.all([
                this.getPreferences(),
                this.getFavorites(),
                this.getExternalAccounts(),
            ]);

            return this.formatSyncData(preferences, favorites, accounts);
        } catch (error) {
            console.error("Failed to download from cloud:", error);
            return null;
        }
    }

    clearCache(): void {
        localStorage.removeItem(CACHE_KEY);
        console.log("User data cache cleared");
    }

    private async refreshCachedUser(): Promise<UserProfile | undefined> {
        const data = await apiClient.get<any>("/users/me");
        const profile = data.profile as UserProfile;
        const info: UserInfo = {
            id: data.id,
            username: data.username,
            email: data.email,
            role: data.role,
            created_at: data.created_at,
            updated_at: data.updated_at,
            last_login_at: data.last_login_at ?? null,
        };
        this.setCachedData({ profile, info });
        return profile;
    }
}

export const userService = new UserService();
