import { reactive, computed } from 'vue';
import { userService, type UserProfile, type UserInfo } from '../services/userService';

interface GlobalUserState {
    profile: UserProfile | null;
    info: UserInfo | null;
    adminStatus: { is_admin: boolean; username: string } | null;
    syncStatus: { last_sync_timestamp: string | null; has_cloud_data: boolean } | null;
    isLoading: boolean;
    isLoaded: boolean;
    lastUpdated: string | null;
}

const globalUserState = reactive<GlobalUserState>({
    profile: null,
    info: null,
    adminStatus: null,
    syncStatus: null,
    isLoading: false,
    isLoaded: false,
    lastUpdated: null
});

export function useUser() {
    const isAuthenticated = computed(() => !!localStorage.getItem('authToken'));

    const displayName = computed(() => {
        if (!globalUserState.info && !globalUserState.profile) return '';
        return globalUserState.profile?.nickname || globalUserState.info?.username || '';
    });

    const username = computed(() => globalUserState.info?.username || '');
    const email = computed(() => globalUserState.info?.email || '');
    const nickname = computed(() => globalUserState.profile?.nickname || '');

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
            globalUserState.adminStatus = initData.admin_status;
            globalUserState.syncStatus = initData.sync_status;
            globalUserState.lastUpdated = new Date().toISOString();
            globalUserState.isLoaded = true;

            console.log('Global user data loaded successfully via combined endpoint');
        } catch (error) {
            console.error('Failed to load global user data:', error);

            try {
                console.log('Falling back to individual API calls...');
                const [profileResult, infoResult] = await Promise.all([
                    userService.loadUserProfile(),
                    userService.loadUserInfo()
                ]);

                globalUserState.profile = profileResult.data;
                globalUserState.info = infoResult.data;
                globalUserState.lastUpdated = new Date().toISOString();
                globalUserState.isLoaded = true;

                console.log('Global user data loaded successfully via fallback');
            } catch (fallbackError) {
                console.error('Failed to load user data via fallback:', fallbackError);
            }
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
            console.error('Failed to update user profile:', error);
            return false;
        }
    };

    const clearUserData = (): void => {
        globalUserState.profile = null;
        globalUserState.info = null;
        globalUserState.adminStatus = null;
        globalUserState.syncStatus = null;
        globalUserState.isLoading = false;
        globalUserState.isLoaded = false;
        globalUserState.lastUpdated = null;
    };

    const refreshUserData = (): Promise<void> => {
        return loadUserData(true);
    };

    return {
        profile: computed(() => globalUserState.profile),
        info: computed(() => globalUserState.info),
        adminStatus: computed(() => globalUserState.adminStatus),
        syncStatus: computed(() => globalUserState.syncStatus),
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
        refreshUserData
    };
}