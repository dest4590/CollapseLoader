export interface EventInfo {
    isActive: boolean;
    name: string;
    emoji: string;
    theme?: string;
}

export function isWinterEvent(): boolean {
    const now = new Date();
    const month = now.getMonth();
    const day = now.getDate();

    if (month === 11 && day >= 10) return true;
    if (month === 0 && day <= 5) return true;

    return false;
}


export function isHalloweenEvent(): boolean {
    const now = new Date();
    const month = now.getMonth();
    const day = now.getDate();

    if (month === 9 && day >= 25) {
        return true;
    }

    if (month === 10 && day <= 5) {
        return true;
    }

    return false;
}

export function getCurrentEvent(): EventInfo | null {
    if (isHalloweenEvent()) {
        return {
            isActive: true,
            name: 'halloween',
            emoji: '🎃',
            theme: 'spooky',
        };
    }

    if (isWinterEvent()) {
        return {
            isActive: true,
            name: 'winter',
            emoji: '❄️',
            theme: 'winter',
        };
    }

    return null;
}

export async function applyCursorForEvent(): Promise<void> {
    const event = getCurrentEvent();

    try {
        if (typeof document !== 'undefined') {
            const root = document.documentElement;
            root.classList.remove('winter-event');
            root.classList.remove('halloween-cursor');

            const existing = document.getElementById('halloween-cursor-style');
            if (existing && existing.parentNode) existing.parentNode.removeChild(existing);
        }
    } catch {
    }

    if (event?.name === 'halloween') {
        try {
            if (typeof document !== 'undefined') {
                const active = isHalloweenEvent();
                const styleId = 'halloween-cursor-style';

                const pointerUrl = new URL('../assets/images/cursores/bat-pointer.png', import.meta.url).href;
                const cursorUrl = new URL('../assets/images/cursores/bat-cursor.png', import.meta.url).href;

                const existing = document.getElementById(styleId);
                if (existing && existing.parentNode) existing.parentNode.removeChild(existing);

                if (active) {
                    const styleEl = document.createElement('style');
                    styleEl.id = styleId;
                    styleEl.textContent = `
                    :root.halloween-cursor,
                    body.halloween-cursor,
                    .halloween-cursor * {
                        cursor: url("${cursorUrl}") 4 4, url("${pointerUrl}") 4 4, auto !important;
                    }
                    .btn,
                    .btn * {
                        cursor: url("${pointerUrl}") 4 4, auto !important;
                    }
                    .input,
                    select {
                                 cursor: url("${pointerUrl}") 4 4, auto !important;
                    }
                    
                    `.trim();
                    document.head.appendChild(styleEl);
                    document.documentElement.classList.add('halloween-cursor');
                } else {
                    document.documentElement.classList.remove('halloween-cursor');
                }
            }
        } catch (e) {
            console.error('Failed to apply halloween cursor:', e);
        }
    }

    if (event?.name === 'winter') {
        try {
            if (typeof document !== 'undefined') {
                document.documentElement.classList.add('winter-event');
            }
        } catch (e) {
            console.error('Failed to apply winter UI:', e);
        }
    }
}