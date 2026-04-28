export function maxIsoTimestamp(
    timestamps: Array<string | null | undefined>
): string | null {
    let max: number | null = null;
    let maxIso: string | null = null;
    for (const ts of timestamps) {
        if (!ts) continue;
        const time = new Date(ts).getTime();
        if (Number.isNaN(time)) continue;
        if (max === null || time > max) {
            max = time;
            maxIso = ts;
        }
    }
    return maxIso;
}

export function formatTime(date: Date = new Date()): string {
    return `${date.getHours().toString().padStart(2, "0")}:${date.getMinutes().toString().padStart(2, "0")}`;
}

export function formatDate(dateString: string): string {
    try {
        const date = new Date(dateString);
        const day = String(date.getDate()).padStart(2, "0");
        const month = String(date.getMonth() + 1).padStart(2, "0");
        const year = date.getFullYear();
        const hours = String(date.getHours()).padStart(2, "0");
        const minutes = String(date.getMinutes()).padStart(2, "0");

        return `${day}/${month}/${year} ${hours}:${minutes}`;
    } catch (e) {
        console.error("Invalid date string:", dateString, e);
        return "N/A";
    }
}
