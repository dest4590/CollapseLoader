import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useModal } from '../services/modalService';
import SlowDownloadWarningModal from '../components/modals/common/SlowDownloadWarningModal.vue';

export interface DownloadSpeedConfig {
    slowSpeedThreshold?: number;
    slowSpeedConsecutiveCount?: number;
    stalledTimeout?: number;
}

export function useDownloadSpeedMonitor(config: DownloadSpeedConfig = {}) {
    const {
        slowSpeedThreshold = 100 * 1024,
        slowSpeedConsecutiveCount = 3,
        stalledTimeout = 3000,
    } = config;

    const { t } = useI18n();
    const { showModal, hideModal } = useModal();

    const lastDownloaded = ref(0);
    const lastTimestamp = ref(0);
    const currentSpeed = ref(0);
    const slowSpeedCount = ref(0);
    const slowSpeedWarningShown = ref(false);
    const stalledWarningShown = ref(false);
    const isMonitoring = ref(false);
    const stalledCheckTimer = ref<ReturnType<typeof setTimeout> | null>(null);
    const lastProgressTime = ref(0);

    const calculateSpeed = (downloaded: number): number => {
        const now = Date.now();
        lastProgressTime.value = now;

        if (lastTimestamp.value > 0 && now - lastTimestamp.value > 0) {
            const timeDiff = (now - lastTimestamp.value) / 1000;
            const bytesDiff = downloaded - lastDownloaded.value;
            if (timeDiff > 0 && bytesDiff >= 0) {
                currentSpeed.value = bytesDiff / timeDiff;
                console.log(`[DownloadMonitor] Speed: ${(currentSpeed.value / 1024).toFixed(1)} KB/s`);
            }
        }
        lastDownloaded.value = downloaded;
        lastTimestamp.value = now;

        return currentSpeed.value;
    };

    const checkSlowSpeed = () => {
        if (currentSpeed.value > 0 && currentSpeed.value < slowSpeedThreshold) {
            slowSpeedCount.value++;
            console.log(`[DownloadMonitor] Slow speed! Count: ${slowSpeedCount.value}/${slowSpeedConsecutiveCount}`);
        } else {
            slowSpeedCount.value = 0;
        }

        if (
            !slowSpeedWarningShown.value &&
            slowSpeedCount.value >= slowSpeedConsecutiveCount
        ) {
            showSlowWarningModal();
        }
    };

    const checkStalled = () => {
        if (!isMonitoring.value || stalledWarningShown.value) {
            return;
        }

        const now = Date.now();
        const timeSinceLastProgress = now - lastProgressTime.value;

        if (lastProgressTime.value > 0 && timeSinceLastProgress >= stalledTimeout) {
            console.log(`[DownloadMonitor] Download stalled! No progress for ${(timeSinceLastProgress / 1000).toFixed(0)}s`);
            showStalledWarningModal();
        }
    };

    const showSlowWarningModal = () => {
        slowSpeedWarningShown.value = true;
        showModal(
            'slow-download-warning',
            SlowDownloadWarningModal,
            {
                title: t('modals.slow_download.modal_title'),
            },
            {
                currentSpeed: currentSpeed.value,
                isStalled: false,
            },
            {
                close: () => hideModal('slow-download-warning'),
            }
        );
    };

    const showStalledWarningModal = () => {
        stalledWarningShown.value = true;
        showModal(
            'slow-download-warning',
            SlowDownloadWarningModal,
            {
                title: t('modals.slow_download.stalled_modal_title'),
            },
            {
                currentSpeed: 0,
                isStalled: true,
            },
            {
                close: () => hideModal('slow-download-warning'),
            }
        );
    };

    const startMonitoring = () => {
        isMonitoring.value = true;
        lastDownloaded.value = 0;
        lastTimestamp.value = 0;
        currentSpeed.value = 0;
        slowSpeedCount.value = 0;
        slowSpeedWarningShown.value = false;
        stalledWarningShown.value = false;
        lastProgressTime.value = Date.now();

        if (stalledCheckTimer.value) {
            clearInterval(stalledCheckTimer.value);
        }
        stalledCheckTimer.value = setInterval(checkStalled, 5000);
    };

    const stopMonitoring = () => {
        isMonitoring.value = false;
        if (stalledCheckTimer.value) {
            clearInterval(stalledCheckTimer.value);
            stalledCheckTimer.value = null;
        }
    };

    const onProgress = (downloaded: number, percentage: number) => {
        if (!isMonitoring.value) {
            startMonitoring();
        }

        calculateSpeed(downloaded);

        if (percentage > 5 && percentage < 95) {
            checkSlowSpeed();
        }
    };

    const reset = () => {
        stopMonitoring();
        lastDownloaded.value = 0;
        lastTimestamp.value = 0;
        currentSpeed.value = 0;
        slowSpeedCount.value = 0;
        slowSpeedWarningShown.value = false;
        stalledWarningShown.value = false;
        lastProgressTime.value = 0;
    };

    return {
        currentSpeed,
        isMonitoring,
        slowSpeedWarningShown,
        stalledWarningShown,

        startMonitoring,
        stopMonitoring,
        onProgress,
        reset,
    };
}
