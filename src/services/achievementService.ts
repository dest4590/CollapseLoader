import { apiClient } from './apiClient';

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

class AchievementService {
    private achievementsCache: Achievement[] | null = null;

    async getAllAchievements(): Promise<Achievement[]> {
        if (this.achievementsCache) {
            return this.achievementsCache;
        }
        const achievements = await apiClient.get<Achievement[]>('/achievements');
        this.achievementsCache = achievements;
        return achievements;
    }

    async getUserAchievements(userId: number): Promise<UserAchievement[]> {
        return await apiClient.get<UserAchievement[]>(`/achievements/users/${userId}`);
    }

    async unlockAchievement(key: string): Promise<void> {
        return await apiClient.post(`/achievements/unlock/${key}`, {});
    }
}

export const achievementService = new AchievementService();
