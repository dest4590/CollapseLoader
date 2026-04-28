export function createPolling(
    callback: () => Promise<void> | void,
    interval: number
) {
    let timer: ReturnType<typeof setInterval> | null = null;

    const visibilityHandler = () => {
        if (document.visibilityState === "visible") {
            Promise.resolve(callback()).catch(() => {});
        }
    };

    const start = () => {
        stop();
        timer = setInterval(() => {
            Promise.resolve(callback()).catch(() => {});
        }, interval);
        document.addEventListener("visibilitychange", visibilityHandler);
    };

    const stop = () => {
        if (timer !== null) {
            clearInterval(timer);
            timer = null;
        }
        document.removeEventListener("visibilitychange", visibilityHandler);
    };

    return { start, stop };
}

export type PollingController = ReturnType<typeof createPolling>;
