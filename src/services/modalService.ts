import { reactive, markRaw } from 'vue';

export interface ModalOptions {
    title?: string;
    contentClass?: string;
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
        const contentClass = options.contentClass || 'medium full-mobile';

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

    return {
        showModal,
        hideModal,
        getModals,
    };
}