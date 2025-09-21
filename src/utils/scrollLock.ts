let openCount = 0;
let originalBodyOverflow: string | null = null;
let originalHtmlOverflow: string | null = null;
let originalBodyPaddingRight: string | null = null;

const getScrollbarWidth = () => {
    if (typeof window === 'undefined') return 0;
    return window.innerWidth - document.documentElement.clientWidth;
};

export const lockScroll = () => {
    if (typeof document === 'undefined') return;
    if (openCount === 0) {
        originalBodyOverflow = document.body.style.overflow;
        originalHtmlOverflow = document.documentElement.style.overflow;
        originalBodyPaddingRight = document.body.style.paddingRight;

        const scrollbarWidth = getScrollbarWidth();
        if (scrollbarWidth > 0) {
            const currentPadding = parseInt(getComputedStyle(document.body).paddingRight || '0', 10) || 0;
            document.body.style.paddingRight = `${currentPadding + scrollbarWidth}px`;
        }

        document.body.style.overflow = 'hidden';
        document.documentElement.style.overflow = 'hidden';
    }
    openCount++;
};

export const unlockScroll = () => {
    if (typeof document === 'undefined') return;
    openCount = Math.max(0, openCount - 1);
    if (openCount === 0) {
        document.body.style.overflow = originalBodyOverflow || '';
        document.documentElement.style.overflow = originalHtmlOverflow || '';
        if (originalBodyPaddingRight !== null) {
            document.body.style.paddingRight = originalBodyPaddingRight;
        } else {
            document.body.style.paddingRight = '';
        }
    }
};

export const getOpenScrollLocks = () => openCount;
