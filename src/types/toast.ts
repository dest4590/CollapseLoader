export type ToastType = 'success' | 'error' | 'info' | 'warning';
export type ToastPosition = 'bottom-right' | 'bottom-left' | 'top-right' | 'top-left' | 'bottom-center' | 'top-center';

export interface ToastMessage {
    id: number;
    message: string;
    type: ToastType;
    duration?: number;
    timeoutId?: number;
    remainingDuration?: number;
    startTime?: number;
    count?: number;
}
