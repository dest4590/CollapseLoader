import { ref } from 'vue';

type RouteName = string;

const currentRoute = ref<RouteName>('home');
const currentParams = ref<Record<string, any> | null>(null);
const historyStack = ref<Array<{ name: RouteName; params: Record<string, any> | null }>>([]);

function push(name: RouteName, params?: Record<string, any>) {
    historyStack.value.push({ name: currentRoute.value, params: currentParams.value });
    currentRoute.value = name;
    currentParams.value = params || null;
}

function replace(name: RouteName, params?: Record<string, any>) {
    currentRoute.value = name;
    currentParams.value = params || null;
}

function back() {
    const last = historyStack.value.pop();
    if (last) {
        currentRoute.value = last.name;
        currentParams.value = last.params || null;
        return true;
    }
    return false;
}

function canGoBack() {
    return historyStack.value.length > 0;
}

function clearHistory() {
    historyStack.value = [];
}

export const router = {
    currentRoute,
    currentParams,
    push,
    replace,
    back,
    canGoBack,
    clearHistory,
};

export function useRouter() {
    return router;
}
