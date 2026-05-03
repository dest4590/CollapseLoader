<template>
    <div class="slide-up">
        <div
            class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-6"
        >
            <h1 class="text-2xl font-semibold text-primary-focus">
                {{ t("navigation.news") }}
                <span
                    v-if="unreadCount > 0"
                    class="ml-2 badge badge-primary badge-sm align-middle animate-pulse"
                >
                    {{ unreadCount }}
                </span>
            </h1>
            <div class="flex gap-2 items-center">
                <input
                    v-model="searchQuery"
                    type="text"
                    class="input input-bordered input-sm w-48 bg-base-100"
                    :placeholder="t('news.search_placeholder')"
                />
                <a
                    href="https://t.me/collapseloader"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="btn btn-sm btn-ghost gap-2 text-primary"
                    :title="t('news.telegram')"
                >
                    <Send class="w-4 h-4" />
                    <span class="hidden md:inline">{{
                        t("news.telegram")
                    }}</span>
                </a>
                <button
                    @click="fetchNews"
                    :disabled="loading"
                    class="btn btn-sm btn-ghost"
                    :title="t('news.refresh')"
                >
                    <RefreshCcw class="w-4 h-4" />
                </button>
                <button
                    v-if="unreadCount > 0"
                    @click="markAllNewsAsRead"
                    class="btn btn-primary btn-sm"
                    :disabled="loading"
                >
                    {{ t("news.mark_all_read") }}
                </button>
            </div>
        </div>

        <div v-if="loading" class="flex justify-center items-center py-12">
            <div class="text-center space-y-3">
                <span
                    class="loading loading-spinner loading-md text-primary"
                ></span>
                <p class="text-base-content/70">{{ t("news.loading") }}</p>
            </div>
        </div>

        <div
            v-else-if="error"
            class="bg-error/10 border border-error/20 rounded-xl p-6"
        >
            <div class="flex items-center gap-3 mb-3">
                <div class="text-error text-xl">⚠️</div>
                <h3 class="text-lg font-semibold text-error">
                    {{ t("news.error_title") }}
                </h3>
            </div>
            <p class="text-base-content/70 mb-4">{{ error }}</p>
            <button
                @click="fetchNews"
                class="btn btn-primary btn-sm"
                :disabled="loading"
            >
                {{ t("news.retry") }}
            </button>
        </div>

        <div
            v-else-if="filteredNews.length === 0"
            class="bg-base-200 rounded-xl border border-base-300 p-12"
        >
            <div class="text-center space-y-3">
                <div class="text-6xl opacity-30">📰</div>
                <h3 class="text-lg font-semibold text-base-content/80">
                    {{ t("news.no_news") }}
                </h3>
                <p class="text-base-content/60">
                    {{ t("news.no_news_description") }}
                </p>
            </div>
        </div>

        <div v-else class="max-w-3xl mx-auto space-y-8 pb-12">
            <div
                v-for="(article, index) in filteredNews"
                :key="article.id"
                :ref="(el) => (newsCardRefs[article.id] = el)"
                class="card bg-base-200 shadow-md border border-base-300 news-card hover:shadow-lg transition-all duration-300"
                :class="{ 'unread-article': !isNewsRead(article) }"
                :style="{ 'animation-delay': index * 0.1 + 's' }"
            >
                <div class="card-body p-6">
                    <div class="flex justify-between items-start mb-4">
                        <div class="flex items-center gap-2">
                            <h2
                                class="card-title text-xl font-bold text-primary-focus"
                            >
                                {{ article.title }}
                            </h2>
                            <div
                                v-if="!isNewsRead(article)"
                                class="badge badge-sm badge-primary animate-pulse"
                            >
                                {{ t("news.new") }}
                            </div>
                        </div>
                        <div
                            class="text-xs text-base-content/60 whitespace-nowrap ml-4"
                        >
                            {{ formatDate(article.created_at) }}
                        </div>
                    </div>

                    <div
                        class="prose prose-sm max-w-none text-base-content/80 news-content"
                        v-html="article.content"
                    ></div>

                    <div
                        v-if="article.updated_at !== article.created_at"
                        class="mt-4 pt-4 border-t border-base-300/50"
                    >
                        <p class="text-xs text-base-content/50">
                            {{ t("news.updated_at") }}:
                            {{ formatDate(article.updated_at) }}
                        </p>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import {
    ref,
    onMounted,
    watch,
    computed,
    onBeforeUnmount,
    nextTick,
} from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@shared/composables/useToast";
import { getCurrentLanguage } from "@services/i18n";
import { formatDate } from "@shared/utils/utils";
import { RefreshCcw, Send } from "lucide-vue-next";
import {
    telegramNewsService,
    type NewsArticle,
} from "@services/telegramNewsService";

const { t } = useI18n();
const { addToast } = useToast();

const news = ref<NewsArticle[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const currentLanguage = ref(getCurrentLanguage() || "en");
const searchQuery = ref("");
const unreadCount = ref(0);
const newsCardRefs = ref<Record<number, any>>({});
let observer: IntersectionObserver | null = null;

const emit = defineEmits<{
    "change-view": [view: string];
    "unread-count-updated": [count: number];
}>();

const isNewsRead = (article: NewsArticle): boolean => {
    return telegramNewsService.isRead(article);
};

const markNewsAsRead = (article: NewsArticle) => {
    if (isNewsRead(article)) return;
    telegramNewsService.markAsRead(article);
    updateUnreadCount();
};

const updateUnreadCount = () => {
    unreadCount.value = telegramNewsService.getUnreadCount(news.value);
    emit("unread-count-updated", unreadCount.value);
};

const markAllNewsAsRead = () => {
    if (news.value && news.value.length > 0) {
        telegramNewsService.markAllAsRead(news.value);
        updateUnreadCount();
    }
};

const fetchNews = async () => {
    loading.value = true;
    error.value = null;

    try {
        news.value = await telegramNewsService.fetchNews(currentLanguage.value);
        updateUnreadCount();
    } catch (err: any) {
        console.error("Failed to fetch news:", err);

        if (err.response?.status === 404) {
            error.value = t("news.not_found");
        } else if (err.response?.status >= 500) {
            error.value = t("news.server_error");
        } else if (
            err.code === "ENOTFOUND" ||
            err.message?.includes("Network Error")
        ) {
            error.value = t("news.network_error");
        } else {
            error.value = t("news.fetch_failed", {
                error: err.message || "Unknown error",
            });
        }

        addToast(t("news.fetch_failed_toast"), "error");
    } finally {
        loading.value = false;
    }
};

const filteredNews = computed(() => {
    if (!searchQuery.value.trim()) return news.value;
    const q = searchQuery.value.toLowerCase();
    return news.value.filter(
        (article) =>
            article.title.toLowerCase().includes(q) ||
            article.content.toLowerCase().includes(q)
    );
});

const setupObserver = () => {
    if (observer) {
        observer.disconnect();
    }

    observer = new IntersectionObserver(
        (entries) => {
            entries.forEach((entry) => {
                if (entry.isIntersecting) {
                    const element = entry.target;
                    const articleId = Object.keys(newsCardRefs.value).find(
                        (id) => newsCardRefs.value[parseInt(id)] === element
                    );

                    if (articleId) {
                        const article = news.value.find(
                            (a) => a.id === parseInt(articleId)
                        );
                        if (article) {
                            markNewsAsRead(article);
                        }
                    }
                }
            });
        },
        {
            threshold: 0.5,
            rootMargin: "0px",
        }
    );

    Object.values(newsCardRefs.value).forEach((element) => {
        if (element) {
            observer!.observe(element);
        }
    });
};

watch(filteredNews, () => {
    nextTick(() => {
        setupObserver();
    });
});

watch(
    () => getCurrentLanguage(),
    (newLang) => {
        if (newLang !== currentLanguage.value) {
            currentLanguage.value = newLang || "en";
            fetchNews();
        }
    }
);

onMounted(() => {
    fetchNews();
});

onBeforeUnmount(() => {
    if (observer) {
        observer.disconnect();
    }
});
</script>

<style scoped>
.slide-up {
    animation: slideUp 0.6s ease-out forwards;
}

@keyframes slideUp {
    from {
        opacity: 0;
        transform: translateY(20px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.news-card {
    opacity: 0;
    transform: translateY(15px);
    animation: fadeInUp 0.5s ease-out forwards;
}

.news-card:hover {
    transform: translateY(-2px);
}

.unread-article {
    position: relative;
    background-color: hsl(var(--b2) / 0.8);
}

.unread-article::before {
    content: "";
    position: absolute;
    left: -3px;
    top: 0;
    bottom: 0;
    width: 3px;
    background-color: hsl(var(--p));
    animation: pulseGlow 2s infinite;
}

@keyframes pulseGlow {
    0%,
    100% {
        opacity: 0.7;
    }

    50% {
        opacity: 1;
    }
}

@keyframes fadeInUp {
    from {
        opacity: 0;
        transform: translateY(15px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.animate-fadeIn {
    animation: fadeInGeneral 0.5s ease-in-out;
}

@keyframes fadeInGeneral {
    from {
        opacity: 0;
    }

    to {
        opacity: 1;
    }
}

.news-content {
    line-height: 1.6;
    white-space: normal;
    overflow-wrap: anywhere;
    word-break: break-word;
}

.news-content :deep(a) {
    text-decoration: underline;
    color: hsl(var(--p));
    transition: opacity 0.2s;
}

.news-content :deep(a):hover {
    opacity: 0.8;
}

.news-content :deep(span),
.news-content :deep(div) {
    overflow-wrap: anywhere;
    word-break: break-all;
    white-space: normal;
}

.news-content :deep(pre),
.news-content :deep(code) {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    word-break: break-word;
}

.news-content :deep(b),
.news-content :deep(strong) {
    font-weight: 600;
    color: hsl(var(--p));
}

.news-content :deep(i),
.news-content :deep(em) {
    font-style: italic;
}

.news-content :deep(u) {
    text-decoration: underline;
}

.news-content :deep(p) {
    margin-bottom: 0.75rem;
}

.news-content :deep(p:last-child) {
    margin-bottom: 0;
}

.news-content :deep(br) {
    display: block;
    margin: 0.5rem 0;
    content: "";
}

.news-content :deep(h1),
.news-content :deep(h2),
.news-content :deep(h3),
.news-content :deep(h4),
.news-content :deep(h5),
.news-content :deep(h6) {
    font-weight: 600;
    color: hsl(var(--p));
    margin: 1rem 0 0.5rem 0;
}

.news-content :deep(h1:first-child),
.news-content :deep(h2:first-child),
.news-content :deep(h3:first-child),
.news-content :deep(h4:first-child),
.news-content :deep(h5:first-child),
.news-content :deep(h6:first-child) {
    margin-top: 0;
}
</style>
