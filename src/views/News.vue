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
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { useToast } from "@shared/composables/useToast";
import { getCurrentLanguage } from "@core/i18n";
import { formatDate } from "@shared/utils/utils";
import { RefreshCcw } from "lucide-vue-next";

interface NewsArticle {
    id: number;
    title: string;
    content: string;
    language: string;
    created_at: string;
    updated_at: string;
}

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

const getNewsUniqueId = (article: NewsArticle): string => {
    return `news_${article.language}_${article.id}`;
};

const isNewsRead = (article: NewsArticle): boolean => {
    const uniqueId = getNewsUniqueId(article);
    const readNews = JSON.parse(localStorage.getItem("readNews") || "[]");
    return readNews.includes(uniqueId);
};

const markNewsAsRead = (article: NewsArticle) => {
    if (isNewsRead(article)) return;
    const uniqueId = getNewsUniqueId(article);
    const readNews = JSON.parse(localStorage.getItem("readNews") || "[]");
    if (!readNews.includes(uniqueId)) {
        readNews.push(uniqueId);
        localStorage.setItem("readNews", JSON.stringify(readNews));
        updateUnreadCount();
    }
};

const updateUnreadCount = () => {
    unreadCount.value = news.value.filter(
        (article) => !isNewsRead(article)
    ).length;
    emit("unread-count-updated", unreadCount.value);
};

const markAllNewsAsRead = () => {
    if (news.value && news.value.length > 0) {
        const readNews = JSON.parse(localStorage.getItem("readNews") || "[]");
        let updated = false;

        news.value.forEach((article) => {
            const uniqueId = getNewsUniqueId(article);
            if (!readNews.includes(uniqueId)) {
                readNews.push(uniqueId);
                updated = true;
            }
        });

        if (updated) {
            localStorage.setItem("readNews", JSON.stringify(readNews));
            updateUnreadCount();
        }
    }
};

const fetchNews = async () => {
    loading.value = true;
    error.value = null;

    try {
        const response = await invoke<string>("api_request", {
            method: "GET",
            url: "https://t.me/s/collapseloader",
            headers: {
                "User-Agent":
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            },
            body: null,
        });

        const htmlText = response;
        const parser = new DOMParser();
        const doc = parser.parseFromString(htmlText, "text/html");

        const messageWraps = doc.querySelectorAll(".tgme_widget_message_wrap");
        const telegramNews: NewsArticle[] = [];

        messageWraps.forEach((wrap) => {
            const textElement = wrap.querySelector(".tgme_widget_message_text");
            const dateElement = wrap.querySelector("time");
            const linkElement = wrap.querySelector(".tgme_widget_message_date");
            const photoElement = wrap.querySelector(
                ".tgme_widget_message_photo_wrap"
            );
            const videoElement = wrap.querySelector(
                ".tgme_widget_message_video_player"
            );

            if (
                dateElement &&
                linkElement &&
                (textElement || photoElement || videoElement)
            ) {
                const href = linkElement.getAttribute("href") || "";
                const parts = href.split("/");
                const postIdStr = parts[parts.length - 1];
                const postId =
                    parseInt(postIdStr, 10) ||
                    Math.floor(Math.random() * 1000000);

                let contentHtml = "";

                if (photoElement) {
                    const style = photoElement.getAttribute("style") || "";
                    const urlMatch = style.match(/url\(['"]?([^'"]+)['"]?\)/);
                    if (urlMatch && urlMatch[1]) {
                        let imageUrl = urlMatch[1];
                        const bestImg = photoElement.querySelector("img");
                        if (bestImg && bestImg.src) {
                            imageUrl = bestImg.src;
                        }

                        contentHtml += `<div class="mt-2 mb-4 overflow-hidden rounded-lg border border-base-content/5 bg-base-300 flex justify-center items-center">
                            <img src="${imageUrl}" 
                                 class="max-h-[300px] w-auto h-auto object-contain" 
                                 style="image-rendering: -webkit-optimize-contrast;"
                                 alt="Telegram image" />
                        </div>`;
                    }
                }

                if (videoElement) {
                    const thumbElement = videoElement.querySelector(
                        ".tgme_widget_message_video_thumb"
                    );
                    if (thumbElement) {
                        const style = thumbElement.getAttribute("style") || "";
                        const urlMatch = style.match(
                            /url\(['"]?([^'"]+)['"]?\)/
                        );
                        if (urlMatch && urlMatch[1]) {
                            contentHtml += `<div class="relative mb-3"><img src="${urlMatch[1]}" class="rounded-lg w-full shadow-sm opacity-80" /><div class="absolute inset-0 flex items-center justify-center"><div class="bg-black/50 rounded-full p-2">▶️</div></div></div>`;
                        }
                    }
                }

                if (textElement) {
                    let textHtml = textElement.innerHTML;
                    textHtml = textHtml.replace(
                        /href="([^"]+)"/g,
                        'href="$1" target="_blank" rel="noopener noreferrer"'
                    );
                    contentHtml += textHtml;
                }

                telegramNews.push({
                    id: postId,
                    title: "Telegram Update",
                    content: contentHtml,
                    language: currentLanguage.value,
                    created_at:
                        dateElement.getAttribute("datetime") ||
                        new Date().toISOString(),
                    updated_at:
                        dateElement.getAttribute("datetime") ||
                        new Date().toISOString(),
                });
            }
        });

        if (telegramNews.length === 0 && htmlText) {
            throw new Error(
                "Telegram parsing failed. HTML starts with: " +
                    htmlText.substring(0, 150)
            );
        }

        news.value = telegramNews.sort(
            (a, b) =>
                new Date(b.created_at).getTime() -
                new Date(a.created_at).getTime()
        );

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

.news-content :deep(a),
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
