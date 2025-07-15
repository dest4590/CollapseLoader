import axios from 'axios';
import { getAuthUrl } from '../config';

const authClient = axios.create();

authClient.interceptors.request.use((config) => {
    const baseUrl = getAuthUrl();

    if (config.url?.startsWith('/')) {
        config.url = `${baseUrl}${config.url}`;
    }

    return config;
});

export const apiGet = (url: string, config?: any): Promise<any> => {
    return authClient.get(url, config);
};

export const apiPost = (url: string, data?: any, config?: any): Promise<any> => {
    return authClient.post(url, data, config);
};

export const apiPatch = (url: string, data?: any, config?: any): Promise<any> => {
    return authClient.patch(url, data, config);
};

export const apiDelete = (url: string, config?: any): Promise<any> => {
    return authClient.delete(url, config);
};

export default authClient;
