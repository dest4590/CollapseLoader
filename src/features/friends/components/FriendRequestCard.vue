<template>
    <div
        class="card bg-base-200 shadow-sm border border-base-300 h-full flex flex-col"
    >
        <div class="card-body p-4 flex-1 flex items-center">
            <div class="flex items-center justify-between w-full">
                <div class="flex items-center gap-3">
                    <div @click="$emit('viewProfile', user.id)">
                        <UserAvatar
                            :name="displayNickname"
                            :is-clickable="true"
                            :src="(props.user as any).avatar_url || null"
                            :original-src="
                                (props.user as any).avatar_url || null
                            "
                        />
                    </div>
                    <div class="min-h-14">
                        <p class="font-medium">{{ displayNickname }}</p>
                        <p class="text-sm text-base-content/70">
                            @{{ displayUsername }}
                        </p>
                    </div>
                </div>
                <div v-if="type === 'received'" class="flex gap-2 items-center">
                    <button
                        @click="$emit('accept', requestId)"
                        class="btn btn-success btn-sm"
                    >
                        <Check class="w-4 h-4" />
                    </button>
                    <button
                        @click="$emit('reject', requestId)"
                        class="btn btn-error btn-sm"
                    >
                        <X class="w-4 h-4" />
                    </button>
                    <button
                        @click="$emit('report', user)"
                        class="btn btn-warning btn-outline btn-sm"
                        :title="t('reports.report_user')"
                    >
                        <Flag class="w-4 h-4" />
                    </button>
                </div>
                <div
                    v-else-if="type === 'sent'"
                    class="flex gap-2 items-center"
                >
                    <button
                        @click="confirmCancel"
                        class="btn btn-error btn-outline btn-sm"
                    >
                        <X class="w-4 h-4" />
                        {{ t("common.cancel") }}
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { Check, Flag, X } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useModal } from "@shared/composables/useModal";
import UserAvatar from "@shared/components/ui/UserAvatar.vue";
import type { Friend } from "@features/auth/userService";
import { useStreamerMode } from "@features/social/useStreamerMode";
import { computed } from "vue";

const { t } = useI18n();
const { showConfirm } = useModal();
const streamer = useStreamerMode();

const props = defineProps<{
    user: Friend;
    requestId: number;
    type: "received" | "sent";
}>();

const emit = defineEmits<{
    accept: [requestId: number];
    reject: [requestId: number];
    viewProfile: [userId: number];
    cancel: [requestId: number];
    report: [user: Friend];
}>();

const displayNickname = computed(() => {
    return streamer.getDisplayName(props.user.nickname, props.user.username);
});

const displayUsername = computed(() => {
    return streamer.getDisplayUsername(props.user.username);
});

const confirmCancel = async () => {
    const confirmed = await showConfirm({
        title: t("friends.cancel_friend_request"),
        message: t("modals.cancel_friend_request_confirm.message", {
            displayName: streamer.getDisplayName(
                props.user.nickname,
                props.user.username
            ),
        }),
        confirmLabel: t("modals.cancel_friend_request_confirm.yes_cancel"),
        cancelLabel: t("modals.cancel_friend_request_confirm.keep_request"),
    });

    if (!confirmed) {
        return;
    }

    emit("cancel", props.requestId);
};
</script>
