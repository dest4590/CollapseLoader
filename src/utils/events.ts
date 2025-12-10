export interface EventInfo {
    isActive: boolean;
    name: string;
    emoji: string;
    theme?: string;
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

    return null;
}

export function getEventGreeting(): string | null {
    const event = getCurrentEvent();

    if (event?.name === 'halloween') {
        const greetings = [
            'Happy Halloween!',
            'Spooky Season!',
            'Trick or Treat!',
            'Boo! 👻',
        ];
        return greetings[Math.floor(Math.random() * greetings.length)];
    }

    return null;
}


export async function applyCursorForEvent(): Promise<void> {
    const event = getCurrentEvent();

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
}