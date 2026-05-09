import { reactive, markRaw } from "vue";
import ConfirmModal from "@shared/components/common/ConfirmModal.vue";

export interface ModalOptions {
    title?: string;
    contentClass?: string;
    size?: "sm" | "md" | "lg" | "xl" | "full";
    [key: string]: any;
}

export interface ConfirmOptions extends ModalOptions {
    message?: string;
    confirmLabel?: string;
    cancelLabel?: string;
}

export interface ModalConfig extends ModalOptions {
    id: string;
    component: any;
    props?: Record<string, any>;
    listeners?: Record<string, (...args: any[]) => void>;
    open: boolean;
}

const modals = reactive<Record<string, ModalConfig>>({});

export function useModal() {
    const showModal = (
        id: string,
        component: any,
        options: ModalOptions = {},
        props: Record<string, any> = {},
        listeners: Record<string, (...args: any[]) => void> = {}
    ) => {
        const contentClass = options.contentClass || "medium full-mobile";

        modals[id] = {
            id,
            component: markRaw(component),
            props,
            listeners,
            open: true,
            ...options,
            contentClass,
        };
    };

    const hideModal = (id: string) => {
        if (modals[id]) {
            modals[id].open = false;
            setTimeout(() => {
                delete modals[id];
            }, 300);
        }
    };

    const getModals = () => modals;

    const showConfirm = (options: ConfirmOptions = {}) => {
        const confirmId = `confirm-${Date.now()}-${Math.random().toString(36).slice(2)}`;
        return new Promise<boolean>((resolve) => {
            showModal(
                confirmId,
                ConfirmModal,
                {
                    title: options.title,
                    size: options.size,
                    contentClass: options.contentClass,
                },
                {
                    title: options.title,
                    message: options.message,
                    confirmLabel: options.confirmLabel,
                    cancelLabel: options.cancelLabel,
                },
                {
                    confirm: () => {
                        hideModal(confirmId);
                        resolve(true);
                    },
                    cancel: () => {
                        hideModal(confirmId);
                        resolve(false);
                    },
                }
            );
        });
    };

    return {
        showModal,
        hideModal,
        getModals,
        showConfirm,
    };
}
