import { reactive, computed, ref, watch } from "vue";
import {
    userService,
    type UserProfile,
    type UserInfo,
} from "@core/auth/userService";
import { localUserService } from "@core/auth/localUserService";
import { STORAGE_KEYS } from "@shared/utils/storageKeys";

interface GlobalUserState {
    profile: UserProfile | null;
    info: UserInfo | null;
    isLoading: boolean;
    isLoaded: boolean;
    lastUpdated: string | null;
}

const globalUserState = reactive<GlobalUserState>({
    profile: null,
    info: null,
    isLoading: false,
    isLoaded: false,
    lastUpdated: null,
});

const authToken = ref(localStorage.getItem(STORAGE_KEYS.AUTH_TOKEN));
const isAuthenticated = computed(() => !!authToken.value);

let authWatcherAttached = false;

window.addEventListener("storage", (event) => {
    if (event.key === STORAGE_KEYS.AUTH_TOKEN) {
        authToken.value = event.newValue;
    }
});

const originalSetItem = localStorage.setItem;
localStorage.setItem = function (key: string, value: string) {
    if (key === STORAGE_KEYS.AUTH_TOKEN) {
        authToken.value = value;
    }
    originalSetItem.apply(this, [key, value]);
};

const originalRemoveItem = localStorage.removeItem;
localStorage.removeItem = function (key: string) {
    if (key === STORAGE_KEYS.AUTH_TOKEN) {
        authToken.value = null;
    }
    originalRemoveItem.apply(this, [key]);
};

const displayName = computed(() => {
    if (!globalUserState.info && !globalUserState.profile) return "";
    return (
        globalUserState.profile?.nickname ||
        globalUserState.info?.username ||
        ""
    );
});

const username = computed(() => globalUserState.info?.username || "");
const email = computed(() => globalUserState.info?.email || "");
const nickname = computed(() => globalUserState.profile?.nickname || "");

const loadUserData = async (forceRefresh = false): Promise<void> => {
    if (!isAuthenticated.value) {
        clearUserData();
        return;
    }

    if (globalUserState.isLoaded && !forceRefresh) {
        return;
    }

    if (globalUserState.isLoading) {
        return;
    }

    globalUserState.isLoading = true;

    try {
        const initData = await userService.initializeUser();

        globalUserState.profile = initData.profile;
        globalUserState.info = initData.user_info;
        globalUserState.lastUpdated = new Date().toISOString();
        globalUserState.isLoaded = true;

        console.log("Global user data loaded successfully");
    } catch (error) {
        console.error("Failed to load global user data:", error);
    } finally {
        globalUserState.isLoading = false;
    }
};

const updateUserProfile = async (newNickname: string): Promise<boolean> => {
    try {
        const result = await userService.updateUserProfile(newNickname);

        if (result.success && globalUserState.profile) {
            globalUserState.profile.nickname = newNickname;
            globalUserState.lastUpdated = new Date().toISOString();
        }

        return result.success;
    } catch (error) {
        console.error("Failed to update user profile:", error);
        return false;
    }
};

const clearUserData = (): void => {
    globalUserState.profile = null;
    globalUserState.info = null;
    globalUserState.isLoading = false;
    globalUserState.isLoaded = false;
    globalUserState.lastUpdated = null;
};

const hydrateUser = (profile: UserProfile, info: UserInfo): void => {
    globalUserState.profile = profile;
    globalUserState.info = info;
    globalUserState.lastUpdated = new Date().toISOString();
    globalUserState.isLoaded = true;
    globalUserState.isLoading = false;
};

const refreshUserData = (): Promise<void> => {
    return loadUserData(true);
};

const logout = (): void => {
    localUserService.logout();
    userService.clearCache();
    clearUserData();
};

const attachAuthWatcher = () => {
    if (authWatcherAttached) return;
    authWatcherAttached = true;
    watch(authToken, async (newToken, oldToken) => {
        if (newToken && newToken !== oldToken) {
            userService.clearCache();
            await loadUserData(true);
            return;
        }
        if (!newToken && oldToken) {
            clearUserData();
        }
    });
};

export function useUser() {
    attachAuthWatcher();

    return {
        profile: computed(() => globalUserState.profile),
        info: computed(() => globalUserState.info),
        isLoading: computed(() => globalUserState.isLoading),
        isLoaded: computed(() => globalUserState.isLoaded),
        lastUpdated: computed(() => globalUserState.lastUpdated),

        displayName,
        username,
        email,
        nickname,
        isAuthenticated,

        loadUserData,
        updateUserProfile,
        clearUserData,
        hydrateUser,
        refreshUserData,
        logout,
    };
}
