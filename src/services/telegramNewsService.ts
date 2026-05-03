import { invoke } from "@tauri-apps/api/core";

export interface NewsArticle {
    id: number;
    title: string;
    content: string;
    language: string;
    created_at: string;
    updated_at: string;
}

export const telegramNewsService = {
    async fetchNews(language: string = "en"): Promise<NewsArticle[]> {
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

            const messageWraps = doc.querySelectorAll(
                ".tgme_widget_message_wrap"
            );
            const telegramNews: NewsArticle[] = [];

            messageWraps.forEach((wrap) => {
                const textElement = wrap.querySelector(
                    ".tgme_widget_message_text"
                );
                const dateElement = wrap.querySelector("time");
                const linkElement = wrap.querySelector(
                    ".tgme_widget_message_date"
                );
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
                        const urlMatch = style.match(
                            /url\(['"]?([^'"]+)['"]?\)/
                        );
                        if (urlMatch && urlMatch[1]) {
                            let imageUrl = urlMatch[1];
                            const bestImg = photoElement.querySelector("img");
                            if (bestImg && bestImg.src) {
                                imageUrl = bestImg.src;
                            }

                            contentHtml += `<div class="mt-2 mb-4 overflow-hidden rounded-lg border border-base-content/5 bg-base-300 flex justify-center items-center">
                                <img src="${imageUrl}" 
                                     class="max-h-75 w-auto h-auto object-contain" 
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
                            const style =
                                thumbElement.getAttribute("style") || "";
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
                        language: language,
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

            return telegramNews.sort(
                (a, b) =>
                    new Date(b.created_at).getTime() -
                    new Date(a.created_at).getTime()
            );
        } catch (error) {
            console.error("Failed to fetch Telegram news:", error);
            throw error;
        }
    },

    getUnreadCount(news: NewsArticle[]): number {
        const readNews = JSON.parse(localStorage.getItem("readNews") || "[]");
        return news.filter((article) => {
            const uniqueId = `news_${article.language}_${article.id}`;
            return !readNews.includes(uniqueId);
        }).length;
    },

    isRead(article: NewsArticle): boolean {
        const uniqueId = `news_${article.language}_${article.id}`;
        const readNews = JSON.parse(localStorage.getItem("readNews") || "[]");
        return readNews.includes(uniqueId);
    },

    markAsRead(article: NewsArticle) {
        const uniqueId = `news_${article.language}_${article.id}`;
        const readNews = JSON.parse(localStorage.getItem("readNews") || "[]");
        if (!readNews.includes(uniqueId)) {
            readNews.push(uniqueId);
            localStorage.setItem("readNews", JSON.stringify(readNews));
        }
    },

    markAllAsRead(news: NewsArticle[]) {
        const readNews = JSON.parse(localStorage.getItem("readNews") || "[]");
        let updated = false;
        news.forEach((article) => {
            const uniqueId = `news_${article.language}_${article.id}`;
            if (!readNews.includes(uniqueId)) {
                readNews.push(uniqueId);
                updated = true;
            }
        });
        if (updated) {
            localStorage.setItem("readNews", JSON.stringify(readNews));
        }
    },
};
