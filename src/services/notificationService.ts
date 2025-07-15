import {
    isPermissionGranted,
    requestPermission,
    sendNotification
} from '@tauri-apps/plugin-notification'

export async function sendNativeNotification(
    title: string,
    body: string,
    icon?: string
): Promise<boolean> {
    try {
        let permissionGranted = await isPermissionGranted();

        if (!permissionGranted) {
            const permission = await requestPermission();
            permissionGranted = permission === 'granted';
        }

        if (permissionGranted) {
            sendNotification({
                title,
                body,
                icon
            });
            return true;
        }

        return false;
    } catch (error) {
        console.error('Failed to send notification:', error);
        return false;
    }
}