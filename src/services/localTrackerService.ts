import { achievementService } from "./achievementService";

interface LocalStats {
    totalLaunches: number;
    totalPlaytimeMinutes: number;
    firstLaunchDate: string | null;
    lastLaunchDate: string | null;
    clientStats: Record<string, { launches: number, playtimeMinutes: number }>;
}

class LocalTrackerService {
    private storageKey = "local_user_stats";
    private stats: LocalStats;
    private playtimeTimer: ReturnType<typeof setInterval> | null = null;

    constructor() {
        const stored = localStorage.getItem(this.storageKey);
        this.stats = stored ? JSON.parse(stored) : {
            totalLaunches: 0,
            totalPlaytimeMinutes: 0,
            firstLaunchDate: null,
            lastLaunchDate: null,
            clientStats: {}
        };
    }

    private save() {
        localStorage.setItem(this.storageKey, JSON.stringify(this.stats));
    }

    trackLaunch(clientName: string) {
        this.stats.totalLaunches++;
        this.stats.lastLaunchDate = new Date().toISOString();
        if (!this.stats.firstLaunchDate) {
            this.stats.firstLaunchDate = this.stats.lastLaunchDate;
        }

        if (!this.stats.clientStats[clientName]) {
            this.stats.clientStats[clientName] = { launches: 0, playtimeMinutes: 0 };
        }
        this.stats.clientStats[clientName].launches++;

        this.save();
        this.checkLaunchAchievements();
    }

    startPlaytimeTracking(clientName: string) {
        if (this.playtimeTimer) clearInterval(this.playtimeTimer);

        this.playtimeTimer = setInterval(() => {
            this.stats.totalPlaytimeMinutes += 1;
            if (this.stats.clientStats[clientName]) {
                this.stats.clientStats[clientName].playtimeMinutes += 1;
            }
            this.save();
            this.checkPlaytimeAchievements();
        }, 60000);
    }

    stopPlaytimeTracking() {
        if (this.playtimeTimer) {
            clearInterval(this.playtimeTimer);
            this.playtimeTimer = null;
        }
    }

    private async checkLaunchAchievements() {
        if (this.stats.totalLaunches >= 1) {
            await achievementService.unlockAchievement("FIRST_GAME");
        }
        if (this.stats.totalLaunches >= 50) {
            await achievementService.unlockAchievement("FREQUENT_FLYER");
        }

        const now = new Date();
        const hour = now.getHours();
        const day = now.getDay();

        if (hour >= 2 && hour < 5) {
            await achievementService.unlockAchievement("NIGHT_OWL");
        }
        if (hour >= 5 && hour < 8) {
            await achievementService.unlockAchievement("EARLY_BIRD");
        }
        if (day === 0 || day === 6) {
            await achievementService.unlockAchievement("WEEKEND_WARRIOR");
        }
    }

    private async checkPlaytimeAchievements() {
        const hours = this.stats.totalPlaytimeMinutes / 60;
        if (hours >= 1) {
            await achievementService.unlockAchievement("PLAYED_1Hour");
        }
        if (hours >= 10) {
            await achievementService.unlockAchievement("PLAYED_10Hours");
        }
    }

    getStats() {
        return this.stats;
    }
}

export const localTrackerService = new LocalTrackerService();
