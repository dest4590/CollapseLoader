import { useToast } from "@shared/composables/useToast";
import i18n from "@core/i18n";

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
];

class AchievementService {
    private storageKey = "local_achievements";

    async getAllAchievements(): Promise<Achievement[]> {
        return ALL_ACHIEVEMENTS;
    }

    async getUserAchievements(_userId?: number): Promise<UserAchievement[]> {
        const stored = localStorage.getItem(this.storageKey);
        if (!stored) return [];

        try {
            const unlockedKeys = JSON.parse(stored) as Record<string, string>;
            return Object.entries(unlockedKeys)
                .map(([key, unlockedAt]) => {
                    const achievement = ALL_ACHIEVEMENTS.find(
                        (a) => a.key === key
                    );
                    if (!achievement) return null;
                    return { achievement, unlockedAt };
                })
                .filter((ua): ua is UserAchievement => ua !== null);
        } catch (e) {
            console.error("Failed to parse local achievements", e);
            return [];
        }
    }

    async unlockAchievement(key: string): Promise<void> {
        const userAchievements = await this.getUserAchievements();
        if (userAchievements.some((ua) => ua.achievement.key === key)) {
            return;
        }

        const achievement = ALL_ACHIEVEMENTS.find((a) => a.key === key);
        if (!achievement) return;

        const unlockedAt = new Date().toISOString();
        const stored = localStorage.getItem(this.storageKey);
        const unlockedKeys = stored ? JSON.parse(stored) : {};
        unlockedKeys[key] = unlockedAt;
        localStorage.setItem(this.storageKey, JSON.stringify(unlockedKeys));

        window.dispatchEvent(
            new CustomEvent("achievement-unlocked", {
                detail: { key, achievement },
            })
        );

        const { addToast } = useToast();

        const achievementName = i18n.global.t(`achievements.list.${key}.name`);

        addToast(
            i18n.global.t("achievements.unlocked_title", {
                name: achievementName,
            }),
            "success"
        );
    }
}

export const achievementService = new AchievementService();
