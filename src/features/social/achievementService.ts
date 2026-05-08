import { STORAGE_KEYS } from "@shared/utils/storageKeys";

export interface Achievement {
    id: number;
    key: string;
    icon: string;
    hidden: boolean;
    receivePercentage: number;
}

export interface UserAchievement {
    achievement: Achievement;
    unlockedAt: string;
}

const ALL_ACHIEVEMENTS: Achievement[] = [
    {
        id: 1,
        key: "FIRST_GAME",
        icon: "Rocket",
        hidden: false,
        receivePercentage: 0,
    },
    {
        id: 2,
        key: "PLAYED_1Hour",
        icon: "Clock",
        hidden: false,
        receivePercentage: 0,
    },
    {
        id: 3,
        key: "PLAYED_10Hours",
        icon: "Award",
        hidden: false,
        receivePercentage: 0,
    },
    {
        id: 4,
        key: "SECRET_FINDER",
        icon: "Search",
        hidden: true,
        receivePercentage: 0,
    },
    {
        id: 6,
        key: "FREQUENT_FLYER",
        icon: "Zap",
        hidden: false,
        receivePercentage: 0,
    },
    {
        id: 8,
        key: "BETA_TESTER",
        icon: "TestTube",
        hidden: false,
        receivePercentage: 0,
    },
    {
        id: 9,
        key: "COLLECTOR",
        icon: "Star",
        hidden: false,
        receivePercentage: 0,
    },
    {
        id: 10,
        key: "NIGHT_OWL",
        icon: "Moon",
        hidden: false,
        receivePercentage: 0,
    },
    {
        id: 11,
        key: "EARLY_BIRD",
        icon: "Sun",
        hidden: false,
        receivePercentage: 0,
    },
    {
        id: 12,
        key: "WEEKEND_WARRIOR",
        icon: "Calendar",
        hidden: false,
        receivePercentage: 0,
    },
    {
        id: 13,
        key: "CHAT_FIRST_MESSAGE",
        icon: "MessageSquare",
        hidden: false,
        receivePercentage: 0,
    },
    {
        id: 14,
        key: "THEME_CHANGER",
        icon: "Palette",
        hidden: false,
        receivePercentage: 0,
    },
    {
        id: 15,
        key: "NIGHT_SESSION",
        icon: "Coffee",
        hidden: true,
        receivePercentage: 0,
    },
    {
        id: 16,
        key: "SPEED_RUNNER",
        icon: "Timer",
        hidden: false,
        receivePercentage: 0,
    },
    {
        id: 17,
        key: "LOYAL_USER",
        icon: "Heart",
        hidden: false,
        receivePercentage: 0,
    },
    {
        id: 18,
        key: "MULTI_CLIENT",
        icon: "Layers",
        hidden: false,
        receivePercentage: 0,
    },
];

class AchievementService {
    private storageKey = STORAGE_KEYS.LOCAL_ACHIEVEMENTS;
    private cachedUnlockedKeys: Record<string, string> | null = null;

    private parseUnlockedKeys(stored: string | null): Record<string, string> {
        if (!stored) return {};

        try {
            const parsed = JSON.parse(stored);
            if (
                typeof parsed === "object" &&
                parsed !== null &&
                !Array.isArray(parsed)
            ) {
                return Object.entries(parsed).reduce(
                    (result, [key, value]) => {
                        if (typeof key === "string" && typeof value === "string") {
                            result[key] = value;
                        }
                        return result;
                    },
                    {} as Record<string, string>
                );
            }
        } catch (e) {
            console.warn("Invalid local achievement storage, resetting:", e);
        }

        return {};
    }

    private getUnlockedKeys(): Record<string, string> {
        if (this.cachedUnlockedKeys) {
            return this.cachedUnlockedKeys;
        }

        const stored = localStorage.getItem(this.storageKey);
        const unlockedKeys = this.parseUnlockedKeys(stored);
        this.cachedUnlockedKeys = unlockedKeys;
        return unlockedKeys;
    }

    private saveUnlockedKeys(unlockedKeys: Record<string, string>): void {
        this.cachedUnlockedKeys = { ...unlockedKeys };
        localStorage.setItem(this.storageKey, JSON.stringify(this.cachedUnlockedKeys));
    }

    async getAllAchievements(): Promise<Achievement[]> {
        return ALL_ACHIEVEMENTS;
    }

    async getUserAchievements(_userId?: number): Promise<UserAchievement[]> {
        const unlockedKeys = this.getUnlockedKeys();
        return Object.entries(unlockedKeys)
            .map(([key, unlockedAt]) => {
                const achievement = ALL_ACHIEVEMENTS.find(
                    (a) => a.key === key
                );
                if (!achievement) return null;
                return { achievement, unlockedAt };
            })
            .filter((ua): ua is UserAchievement => ua !== null);
    }

    async unlockAchievement(key: string): Promise<void> {
        const unlockedKeys = this.getUnlockedKeys();
        if (key in unlockedKeys) {
            return;
        }

        const achievement = ALL_ACHIEVEMENTS.find((a) => a.key === key);
        if (!achievement) return;

        const unlockedAt = new Date().toISOString();
        unlockedKeys[key] = unlockedAt;
        this.saveUnlockedKeys(unlockedKeys);

        window.dispatchEvent(
            new CustomEvent("achievement-unlocked", {
                detail: { key, achievement },
            })
        );
    }
}

export const achievementService = new AchievementService();
