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
    async getAllAchievements(): Promise<Achievement[]> {
        return await apiClient.get<Achievement[]>('/achievements');
    }

    async getUserAchievements(userId: number): Promise<UserAchievement[]> {
        return await apiClient.get<UserAchievement[]>(`/achievements/users/${userId}`);
    }

    async unlockAchievement(key: string): Promise<void> {
        return await apiClient.post(`/achievements/unlock/${key}`, {});
    }
}

export const achievementService = new AchievementService();
