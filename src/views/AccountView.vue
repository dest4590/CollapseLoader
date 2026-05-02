<template>
    <div class="max-w-3xl mx-auto p-6 slide-up">
        <div class="grid gap-6 grid-cols-1 md:grid-cols-2">
            <div class="col-span-1 md:col-span-2">
                <div class="card shadow-md border border-base-300">
                    <div class="card-body">
                        <div class="flex items-center gap-4 mb-4">
                            <UserAvatar
                                :name="
                                    userInfo.nickname ||
                                    userInfo.username ||
                                    'User'
                                "
                                size="lg"
                                :is-clickable="true"
                                :src="
                                    (useUser().profile.value as any)
                                        ?.avatar_url || null
                                "
                                :original-src="
                                    (useUser().profile.value as any)
                                        ?.avatar_url || null
                                "
                                @click="openAvatarModal"
                            />
                            <div class="flex-1">
                                <div class="flex items-start justify-between">
                                    <h2
                                        class="text-xl font-semibold text-primary-focus flex items-center gap-2"
                                    >
                                        {{
                                            userInfo.nickname ||
                                            userInfo.username ||
                                            "User"
                                        }}
                                        <span
                                            v-if="roleBadge"
                                            :class="
                                                roleBadge.className + ' text-sm'
                                            "
                                            @click="
                                                isLocalUser
                                                    ? openRoleModal()
                                                    : null
                                            "
                                            :style="
                                                isLocalUser
                                                    ? 'cursor: pointer'
                                                    : ''
                                            "
                                            :title="
                                                isLocalUser
                                                    ? t(
                                                          'modals.role_selection.description'
                                                      ) || 'Сменить роль'
                                                    : ''
                                            "
                                        >
                                            {{ roleBadge.text }}
                                        </span>
                                        <button
                                            @click="openNicknameModal"
                                            class="btn btn-ghost btn-xs p-1"
                                            :disabled="isLoadingFromCache"
                                        >
                                            <EditIcon class="w-3 h-3" />
                                        </button>
                                    </h2>
                                </div>

                                <p class="text-base-content/70 text-sm mt-1">
                                    @{{ userInfo.username }}
                                </p>

                                <p
                                    class="text-base-content/60 text-xs mt-1 flex items-center gap-2"
                                >
                                    <button
                                        class="btn btn-ghost btn-sm p-0 h-auto min-h-0 font-normal"
                                        @click="toggleShowEmail"
                                    >
                                        {{
                                            showEmail
                                                ? email || t("account.no_email")
                                                : maskedEmail ||
                                                  t("account.no_email")
                                        }}
                                    </button>
                                    <component
                                        :is="showEmail ? EyeOffIcon : EyeIcon"
                                        class="w-3 h-3 cursor-pointer opacity-50"
                                        @click="toggleShowEmail"
                                    />
                                </p>

                                <div class="flex items-center mt-3 text-sm">
                                    <div
                                        class="badge"
                                        :class="
                                            invisibleMode
                                                ? 'badge-secondary'
                                                : 'badge-success'
                                        "
                                    >
                                        {{
                                            invisibleMode
                                                ? t("time.offline")
                                                : t("time.online")
                                        }}
                                    </div>

                                    <span>
                                        <button
                                            @click="openSocialLinks"
                                            class="btn btn-primary btn-xs ml-3"
                                        >
                                            {{ t("account.social_links") }}
                                        </button>
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="col-span-1 md:col-span-2">
                <div class="card bg-base-200 shadow-md border border-base-300">
                    <div class="card-body">
                        <div class="flex items-center justify-between mb-4">
                            <h2 class="text-lg font-medium text-primary-focus">
                                {{ t("achievements.title") }}
                            </h2>
                            <span
                                v-if="achievements.length > 0"
                                class="text-sm text-base-content/60"
                            >
                                {{ unlockedCount }} / {{ achievements.length }}
                            </span>
                        </div>
                        <div
                            v-if="loadingAchievements"
                            class="flex justify-center py-4"
                        >
                            <span
                                class="loading loading-spinner loading-md"
                            ></span>
                        </div>
                        <div v-else>
                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                <AchievementCard
                                    v-for="ach in displayedAchievements"
                                    :key="ach.key"
                                    :achievement-key="ach.key"
                                    :icon-name="ach.icon"
                                    :locked="!isUnlocked(ach.id)"
                                    :unlocked-at="getUnlockedAt(ach.id)"
                                    :hidden="ach.hidden"
                                    :receive-percentage="ach.receivePercentage"
                                />
                            </div>

                            <div
                                v-if="achievements.length > initialDisplayCount"
                                class="flex justify-center mt-4"
                            >
                                <button
                                    @click="toggleAchievementsExpand"
                                    class="btn btn-ghost btn-sm"
                                >
                                    {{
                                        isAchievementsExpanded
                                            ? t("common.show_less")
                                            : t("common.show_more")
                                    }}
                                    <component
                                        :is="
                                            isAchievementsExpanded
                                                ? ChevronUp
                                                : ChevronDown
                                        "
                                        class="w-4 h-4 ml-1"
                                    />
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="col-span-1 md:col-span-2">
                <div class="card bg-base-200 shadow-md border border-base-300">
                    <div class="card-body">
                        <div class="flex items-center justify-between mb-4">
                            <h2
                                class="card-title text-lg font-medium text-primary-focus"
                            >
                                {{ t("account.playtime_stats") }}
                            </h2>
                            <div class="flex items-center gap-3">
                                <div
                                    class="flex items-center gap-1.5 text-sm text-base-content/60"
                                >
                                    <Timer class="w-4 h-4" />
                                    <span>{{
                                        formatPlaytime(
                                            localStats.totalPlaytimeMinutes
                                        )
                                    }}</span>
                                </div>
                                <button
                                    v-if="
                                        Object.keys(localStats.clientStats)
                                            .length > 3
                                    "
                                    @click="openPlaytimeModal"
                                    class="btn btn-ghost btn-xs gap-1 text-primary"
                                >
                                    <LayoutList class="w-3.5 h-3.5" />
                                    {{
                                        t("account.playtime_show_all", {
                                            count: Object.keys(
                                                localStats.clientStats
                                            ).length,
                                        })
                                    }}
                                </button>
                            </div>
                        </div>

                        <div
                            v-if="
                                Object.keys(localStats.clientStats).length === 0
                            "
                            class="text-center py-6 text-base-content/50"
                        >
                            <Gamepad2
                                class="w-10 h-10 mx-auto mb-2 opacity-30"
                            />
                            <p>{{ t("account.playtime_no_data") }}</p>
                        </div>

                        <div v-else class="space-y-2">
                            <div
                                v-for="(
                                    [clientName, stats], index
                                ) in topClientStats"
                                :key="clientName"
                                class="flex items-center gap-3 p-3 bg-base-300/50 rounded-lg"
                            >
                                <span
                                    class="text-base-content/30 text-xs font-mono w-4 text-right shrink-0"
                                    >{{ index + 1 }}</span
                                >
                                <Gamepad2
                                    class="w-4 h-4 text-primary shrink-0"
                                />
                                <div class="flex-1 min-w-0">
                                    <div
                                        class="flex items-center justify-between gap-2"
                                    >
                                        <span
                                            class="font-medium truncate text-sm"
                                            >{{ clientName }}</span
                                        >
                                        <span
                                            class="text-primary font-semibold shrink-0 text-sm"
                                            >{{
                                                formatPlaytime(
                                                    stats.playtimeMinutes
                                                )
                                            }}</span
                                        >
                                    </div>
                                    <div class="flex items-center gap-3 mt-1">
                                        <div
                                            class="flex-1 bg-base-content/10 rounded-full h-1"
                                        >
                                            <div
                                                class="bg-primary h-1 rounded-full transition-all"
                                                :style="{
                                                    width:
                                                        getPlaytimePercent(
                                                            stats.playtimeMinutes
                                                        ) + '%',
                                                }"
                                            ></div>
                                        </div>
                                        <span
                                            class="text-xs text-base-content/40 shrink-0"
                                        >
                                            {{
                                                t("account.playtime_launches", {
                                                    count: stats.launches,
                                                })
                                            }}
                                        </span>
                                    </div>
                                </div>
                            </div>

                            <button
                                v-if="
                                    Object.keys(localStats.clientStats).length >
                                    3
                                "
                                @click="openPlaytimeModal"
                                class="w-full flex items-center justify-center gap-2 py-2 rounded-lg text-sm text-base-content/40 hover:text-base-content/70 hover:bg-base-300/40 transition-colors"
                            >
                                <ChevronDown class="w-4 h-4" />
                                {{
                                    t("account.playtime_more", {
                                        count:
                                            Object.keys(localStats.clientStats)
                                                .length - 3,
                                    })
                                }}
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <div class="col-span-1 md:col-span-2">
                <div class="card bg-base-200 shadow-md border border-base-300">
                    <div class="card-body">
                        <h2
                            class="card-title text-lg font-medium text-primary-focus mb-4"
                        >
                            {{ t("account.favorite_client") }}
                        </h2>
                        <p class="text-sm text-base-content/70 mb-4">
                            {{ t("account.favorite_client_description") }}
                        </p>
                        <div
                            v-if="loadingClients"
                            class="flex justify-center py-4"
                        >
                            <span
                                class="loading loading-spinner loading-md"
                            ></span>
                        </div>
                        <div v-else class="form-control">
                            <select
                                v-model="selectedFavoriteClientId"
                                class="select select-bordered w-full bg-base-100"
                                @change="handleFavoriteClientChange"
                                :disabled="
                                    isLoadingFromCache || updatingFavoriteClient
                                "
                            >
                                <option :value="null">
                                    {{ t("account.no_favorite_client") }}
                                </option>
                                <option
                                    v-for="client in availableClients"
                                    :key="client.id"
                                    :value="client.id"
                                >
                                    {{ client.name }}
                                    {{ formatVersion(client.version) }}
                                </option>
                            </select>
                            <div
                                v-if="currentFavoriteClient"
                                class="mt-3 p-3 bg-base-300 rounded-lg"
                            >
                                <div class="flex items-center gap-2">
                                    <span class="text-sm font-medium"
                                        >{{
                                            t("account.current_favorite")
                                        }}:</span
                                    >
                                    <span class="badge badge-primary"
                                        >{{ currentFavoriteClient.name }}
                                        {{
                                            formatVersion(
                                                currentFavoriteClient.version
                                            )
                                        }}</span
                                    >
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div>
                <div
                    class="card bg-base-200 shadow-md border border-base-300 h-full"
                >
                    <div class="card-body">
                        <h2
                            class="card-title text-lg font-medium text-primary-focus mb-2"
                        >
                            {{ t("account.invisible_mode") }}
                        </h2>
                        <div class="flex items-center justify-between">
                            <div>
                                <h3 class="font-medium">
                                    {{ t("account.invisible_mode") }}
                                </h3>
                                <p class="text-sm text-base-content/70">
                                    {{
                                        t("account.invisible_mode_description")
                                    }}
                                </p>
                            </div>
                            <input
                                type="checkbox"
                                class="checkbox"
                                v-model="invisibleMode"
                                @change="handleInvisibleModeToggle"
                                :disabled="isLoadingFromCache"
                            />
                        </div>
                        <div
                            v-if="isLoadingFromCache"
                            class="text-sm text-warning mt-3"
                        >
                            <span>{{ t("account.using_cached_data") }}</span>
                        </div>
                    </div>
                </div>
            </div>

            <div>
                <div
                    class="card bg-base-200 shadow-md border border-base-300 h-full"
                >
                    <div class="card-body">
                        <h2
                            class="card-title text-lg font-medium text-primary-focus mb-2"
                        >
                            {{ t("account.streamer_mode") }}
                        </h2>
                        <div class="flex items-center justify-between">
                            <div>
                                <h3 class="font-medium">
                                    {{ t("account.streamer_mode") }}
                                </h3>
                                <p class="text-sm text-base-content/70">
                                    {{ t("account.streamer_mode_description") }}
                                </p>
                            </div>
                            <input
                                type="checkbox"
                                class="checkbox"
                                v-model="streamerMode"
                                @change="handleStreamerModeToggle"
                            />
                        </div>
                    </div>
                </div>
            </div>

            <div v-if="!isLocalUser" class="col-span-1 md:col-span-2">
                <div class="card bg-base-200 shadow-md border border-base-300">
                    <div class="card-body">
                        <h2
                            class="card-title text-lg font-medium text-primary-focus mb-4"
                        >
                            {{ t("account.change_password") }}
                        </h2>
                        <form @submit.prevent="handleChangePassword">
                            <div class="form-control mb-4">
                                <label class="label mb-2">
                                    <span class="label-text">{{
                                        t("account.current_password")
                                    }}</span>
                                </label>
                                <input
                                    v-model="currentPassword"
                                    type="password"
                                    autocomplete="current-password"
                                    :placeholder="t('account.current_password')"
                                    class="input input-bordered w-full bg-base-100"
                                    required
                                    :disabled="isLoadingFromCache"
                                />
                            </div>
                            <div class="form-control mb-4">
                                <label class="label mb-2">
                                    <span class="label-text">{{
                                        t("account.new_password")
                                    }}</span>
                                </label>
                                <input
                                    v-model="newPassword"
                                    type="password"
                                    autocomplete="new-password"
                                    :placeholder="t('account.new_password')"
                                    class="input input-bordered w-full bg-base-100"
                                    required
                                    :disabled="isLoadingFromCache"
                                />
                            </div>
                            <div class="form-control mb-4">
                                <label class="label mb-2">
                                    <span class="label-text">{{
                                        t("account.confirm_password")
                                    }}</span>
                                </label>
                                <input
                                    v-model="confirmNewPassword"
                                    type="password"
                                    autocomplete="new-password"
                                    :placeholder="t('account.confirm_password')"
                                    class="input input-bordered w-full bg-base-100"
                                    required
                                    :disabled="isLoadingFromCache"
                                />
                            </div>
                            <button
                                type="submit"
                                class="btn btn-primary w-full"
                                :disabled="isLoadingFromCache"
                            >
                                {{
                                    isLoadingFromCache
                                        ? t("account.using_cached_data")
                                        : t("account.change_password")
                                }}
                            </button>
                        </form>
                    </div>
                </div>
            </div>

            <div class="col-span-1 md:col-span-2">
                <div class="card bg-base-200 shadow-md border border-base-300">
                    <div class="card-body">
                        <h2
                            class="card-title text-lg font-medium text-primary-focus mb-4"
                        >
                            {{ t("common.logout") }}
                        </h2>
                        <p class="text-base-content/70 mb-4">
                            {{ t("auth.logout.confirm") }}
                        </p>
                        <button
                            @click="handleLogout"
                            class="btn btn-error w-full"
                        >
                            {{ t("common.logout") }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, onUnmounted, watch } from "vue";
import { useToast } from "@shared/composables/useToast";
import { useModal } from "@shared/composables/useModal";
import { useI18n } from "vue-i18n";
import EditNicknameModal from "@features/social/modals/EditNicknameModal.vue";
import SocialLinksModal from "@features/social/modals/SocialLinksModal.vue";
import ChangePasswordConfirmModal from "@features/social/modals/ChangePasswordConfirmModal.vue";
import LogoutConfirmModal from "@features/social/modals/LogoutConfirmModal.vue";
import AvatarUploadModal from "@features/social/modals/AvatarUploadModal.vue";
import RoleSelectionModal from "@features/social/modals/RoleSelectionModal.vue";
import UserAvatar from "@shared/components/ui/UserAvatar.vue";
import AchievementCard from "../components/features/profile/AchievementCard.vue";
import { useUser } from "@features/auth/useUser";
import { userService } from "@features/auth/userService";
import {
    achievementService,
    type Achievement,
    type UserAchievement,
} from "@features/social/achievementService";
import {
    EditIcon,
    EyeIcon,
    EyeOffIcon,
    ChevronDown,
    ChevronUp,
    Timer,
    Gamepad2,
    LayoutList,
} from "lucide-vue-next";
import { localTrackerService } from "../services/localTrackerService";
import PlaytimeStatsModal from "@/components/modals/common/PlaytimeStatsModal.vue";
import getRoleBadge from "@features/social/utils/roleBadge";
import { globalUserStatus } from "@features/auth/useUserStatus";
import { syncService, SyncServiceState } from "../services/syncService";
import { invoke } from "@tauri-apps/api/core";

interface Client {
    id: number;
    name: string;
    version: string;
    filename: string;
    md5_hash: string;
    main_class: string;
    show: boolean;
    working: boolean;
    insecure: boolean;
    launches: number;
    downloads: number;
    size: number;
}

const { t } = useI18n();
const { addToast } = useToast();
const { showModal, hideModal } = useModal();

const currentPassword = ref("");
const newPassword = ref("");
const confirmNewPassword = ref("");
const nickname = ref("");
const showEmail = ref(false);

const achievements = ref<Achievement[]>([]);
const userAchievements = ref<UserAchievement[]>([]);
const loadingAchievements = ref(false);
const isAchievementsExpanded = ref(false);
const initialDisplayCount = 4;

const availableClients = ref<Client[]>([]);
const loadingClients = ref(false);
const selectedFavoriteClientId = ref<number | null>(null);
const updatingFavoriteClient = ref(false);

const user = useUser();
const {
    username,
    email,
    nickname: userNickname,
    isLoading: isLoadingFromCache,
    isAuthenticated,
    refreshUserData,
    updateUserProfile: updateGlobalUserProfile,
    logout,
} = user;

const isLocalUser = computed(() =>
    localStorage.getItem("authToken")?.startsWith("local_")
);

const invisibleMode = computed({
    get: () => globalUserStatus.isInvisible.value,
    set: (_) => {},
});

const streamerMode = computed({
    get: () => globalUserStatus.isStreamer.value,
    set: (_) => {},
});

const userInfo = computed(() => {
    const nickname = userNickname.value;
    const user = username.value;
    const mail = email.value;
    const role =
        (useUser().profile.value && (useUser().profile.value as any).role) ||
        null;

    if (globalUserStatus.isStreamer.value) {
        return {
            nickname: "??????",
            username: "unknown",
            email: "unknown@*****.***",
            role: null,
        };
    }

    return {
        nickname: nickname,
        username: user,
        email: mail,
        role: role,
    };
});

const currentFavoriteClient = computed(() => {
    const profile = useUser().profile.value;
    if (profile && (profile as any).favorite_client_id) {
        const clientId = (profile as any).favorite_client_id;
        return availableClients.value.find((c) => c.id === clientId) || null;
    }
    return null;
});

const roleBadge = computed(() => {
    return getRoleBadge(userInfo.value.role, (k: string) => t(k));
});

const localStats = computed(() => localTrackerService.getStats());

const topClientStats = computed(() => {
    const stats = localStats.value.clientStats;
    return Object.entries(stats)
        .sort(([, a], [, b]) => b.playtimeMinutes - a.playtimeMinutes)
        .slice(0, 3);
});

const maxPlaytimeMinutes = computed(() => {
    const stats = localStats.value.clientStats;
    if (!Object.keys(stats).length) return 1;
    return Math.max(...Object.values(stats).map((s) => s.playtimeMinutes), 1);
});

const getPlaytimePercent = (minutes: number) => {
    return Math.round((minutes / maxPlaytimeMinutes.value) * 100);
};

const formatPlaytime = (minutes: number): string => {
    if (!minutes) return "0m";
    const h = Math.floor(minutes / 60);
    const m = minutes % 60;
    if (h > 0 && m > 0) return `${h}h ${m}m`;
    if (h > 0) return `${h}h`;
    return `${m}m`;
};

const openPlaytimeModal = () => {
    showModal(
        "playtime-stats",
        PlaytimeStatsModal,
        { title: t("account.playtime_stats") },
        {},
        { close: () => hideModal("playtime-stats") }
    );
};

const maskedEmail = computed(() => {
    const email = userInfo.value.email || "";
    if (!email) return "";
    const parts = email.split("@");
    if (parts.length !== 2) return "*****";
    const local = parts[0];
    const domain = parts[1];
    const maskedLocal =
        local.length > 2
            ? `${local[0]}***${local.slice(-1)}`
            : "*".repeat(Math.max(1, local.length - 1));

    const lastDot = domain.lastIndexOf(".");
    const maskedDomain =
        lastDot > 0 ? `*****${domain.slice(lastDot)}` : "*****";
    return `${maskedLocal}@${maskedDomain}`;
});

const unlockedCount = computed(() => userAchievements.value.length);

const sortedAchievements = computed(() => {
    return [...achievements.value].sort((a, b) => {
        const aUnlocked = isUnlocked(a.id);
        const bUnlocked = isUnlocked(b.id);
        if (aUnlocked && !bUnlocked) return -1;
        if (!aUnlocked && bUnlocked) return 1;
        return 0;
    });
});

const displayedAchievements = computed(() => {
    const sorted = sortedAchievements.value;
    if (isAchievementsExpanded.value) return sorted;
    return sorted.slice(0, initialDisplayCount);
});

const toggleAchievementsExpand = () => {
    isAchievementsExpanded.value = !isAchievementsExpanded.value;
};

const isUnlocked = (achievementId: number) => {
    return userAchievements.value.some(
        (ua) => ua.achievement.id === achievementId
    );
};

const getUnlockedAt = (achievementId: number) => {
    const ua = userAchievements.value.find(
        (ua) => ua.achievement.id === achievementId
    );
    return ua ? ua.unlockedAt : null;
};

const loadAchievements = async () => {
    loadingAchievements.value = true;
    try {
        const [all, user] = await Promise.all([
            achievementService.getAllAchievements(),
            achievementService.getUserAchievements(),
        ]);
        achievements.value = all || [];
        userAchievements.value = user || [];
    } catch (e) {
        console.error("Failed to load achievements", e);
    } finally {
        loadingAchievements.value = false;
    }
};

const toggleShowEmail = () => {
    showEmail.value = !showEmail.value;
};

const formatVersion = (version: string): string => {
    if (!version) return "";
    return version.replace(/^V/, "").replace(/_/g, ".");
};

const loadClients = async () => {
    try {
        loadingClients.value = true;
        const clients = await invoke<Client[]>("get_clients");
        availableClients.value = clients.filter((c) => c.show && c.working);

        const profile = useUser().profile.value;
        if (profile && (profile as any).favorite_client_id) {
            selectedFavoriteClientId.value = (
                profile as any
            ).favorite_client_id;
        }
    } catch (error) {
        console.error("Failed to load clients:", error);
        addToast(t("errors.clients_load_failed", { error }), "error");
    } finally {
        loadingClients.value = false;
    }
};

const handleFavoriteClientChange = async () => {
    try {
        updatingFavoriteClient.value = true;
        const result = await userService.updateUserProfile(
            null,
            selectedFavoriteClientId.value
        );

        if (result.success) {
            await refreshUserData();
            addToast(t("account.favorite_client_updated"), "success");
        } else {
            addToast(
                result.error || t("account.favorite_client_update_failed"),
                "error"
            );

            const profile = useUser().profile.value;

            if (profile && (profile as any).favorite_client_id) {
                selectedFavoriteClientId.value = (
                    profile as any
                ).favorite_client_id;
            } else {
                selectedFavoriteClientId.value = null;
            }
        }
    } catch (error) {
        console.error("Failed to update favorite client:", error);
        addToast(t("account.favorite_client_update_failed"), "error");
    } finally {
        updatingFavoriteClient.value = false;
    }
};

const syncState = ref<SyncServiceState>(syncService.getState());
let unsubscribeSyncService: (() => void) | null = null;

const emit = defineEmits(["logged-out"]);

onMounted(async () => {
    unsubscribeSyncService = syncService.subscribe((state) => {
        syncState.value = state;
    });

    await syncService.initializeSyncStatus();
    await loadAchievements();
    await loadClients();
});

watch(
    isAuthenticated,
    async (isAuthed, wasAuthed) => {
        if (isAuthed && !wasAuthed) {
            try {
                await refreshUserData();
                nickname.value = userNickname.value || "";
                await globalUserStatus.fetchCurrentStatus();
            } catch (error) {
                console.error(
                    "Failed to refresh account data after auth change:",
                    error
                );
            }
            return;
        }

        if (!isAuthed && wasAuthed) {
            nickname.value = "";
            showEmail.value = false;
            achievements.value = [];
            userAchievements.value = [];
        }
    },
    { immediate: true }
);

watch(
    () => (useUser().info.value as any)?.id,
    (newId) => {
        if (newId) {
            loadAchievements();
        }
    }
);

onUnmounted(() => {
    if (unsubscribeSyncService) {
        unsubscribeSyncService();
    }
});

const handleStreamerModeToggle = async () => {
    try {
        globalUserStatus.setStreamerMode(!streamerMode.value);
        addToast(
            streamerMode.value
                ? t("account.streamer_enabled")
                : t("account.streamer_disabled"),
            "success"
        );
    } catch (error) {
        console.error("Failed to toggle streamer mode:", error);
        addToast(t("account.streamer_failed"), "error");
    }
};

const handleInvisibleModeToggle = async () => {
    try {
        globalUserStatus.setInvisibleMode(!invisibleMode.value);
        await globalUserStatus.forceSyncStatus();
        addToast(
            invisibleMode.value
                ? t("account.invisible_enabled")
                : t("account.invisible_disabled"),
            "success"
        );
    } catch (error) {
        console.error("Failed to toggle invisible mode:", error);
        addToast(t("account.invisible_failed"), "error");
    }
};

const openNicknameModal = () => {
    showModal(
        "edit-nickname",
        EditNicknameModal,
        { title: t("account.nickname_label") },
        { currentNickname: nickname.value },
        {
            "nickname-updated": async (newNickname: string) => {
                const success = await updateGlobalUserProfile(newNickname);
                if (success) {
                    nickname.value = newNickname;
                    addToast(t("account.nickname_update_success"), "success");
                } else {
                    addToast(t("account.nickname_update_failed"), "error");
                }
            },
            close: () => hideModal("edit-nickname"),
        }
    );
};

const openRoleModal = () => {
    showModal(
        "role-selection",
        RoleSelectionModal,
        { title: t("roles.local_user") },
        { currentRole: userInfo.value.role },
        {
            "role-selected": async (newRole: string) => {
                const result = await userService.updateUserProfile(
                    null,
                    undefined,
                    newRole
                );
                if (result.success) {
                    await refreshUserData();
                    addToast(t("account.role_update_success"), "success");
                } else {
                    addToast(t("account.role_update_failed"), "error");
                }
            },
            close: () => hideModal("role-selection"),
        }
    );
};

const openSocialLinks = () => {
    showModal(
        "social-links",
        SocialLinksModal,
        { title: t("modals.social_links.title") },
        {},
        {
            close: () => hideModal("social-links"),
        }
    );
};

const handleChangePassword = async () => {
    if (newPassword.value !== confirmNewPassword.value) {
        addToast(t("account.password_mismatch"), "error");
        return;
    }

    if (newPassword.value.length < 8) {
        addToast(t("account.password_too_short"), "error");
        return;
    }

    showModal(
        "change-password-confirm",
        ChangePasswordConfirmModal,
        { title: t("account.change_password_confirm_title") },
        {
            currentPassword: currentPassword.value,
            newPassword: newPassword.value,
        },
        {
            confirm: async (passwordData: {
                currentPassword: string;
                newPassword: string;
            }) => {
                try {
                    await userService.changePassword(
                        passwordData.currentPassword,
                        passwordData.newPassword
                    );
                    addToast(t("account.password_change_success"), "success");
                    currentPassword.value = "";
                    newPassword.value = "";
                    confirmNewPassword.value = "";
                } catch (e) {
                    console.log("Failed to change password:", e);
                    addToast(t("account.password_change_failed"), "error");
                }
                hideModal("change-password-confirm");
            },
            close: () => hideModal("change-password-confirm"),
        }
    );
};

const handleLogout = () => {
    showModal(
        "logout-confirm",
        LogoutConfirmModal,
        { title: t("account.logout_confirm_title") },
        {},
        {
            confirm: () => {
                logout();
                emit("logged-out");
                addToast(t("auth.logout.success"), "success");
                hideModal("logout-confirm");
            },
            close: () => hideModal("logout-confirm"),
        }
    );
};

const openAvatarModal = () => {
    showModal(
        "avatar-upload",
        AvatarUploadModal,
        { title: t("account.upload_avatar") },
        { currentUrl: (useUser().profile.value as any)?.avatar_url || null },
        {
            uploaded: async () => {
                try {
                    await useUser().refreshUserData();
                } catch {
                    addToast(
                        t("account.avatar_upload_refresh_failed"),
                        "warning"
                    );
                }
                hideModal("avatar-upload");
            },
            close: () => hideModal("avatar-upload"),
        }
    );
};
</script>

<style scoped>
.slide-up {
    animation: slideUp 0.5s ease-out forwards;
}

@keyframes slideUp {
    0% {
        opacity: 0;
        transform: translateY(20px);
    }

    100% {
        opacity: 1;
        transform: translateY(0);
    }
}
</style>
