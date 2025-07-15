import axios from 'axios';
import { getApiUrl } from '../config';

const apiClient = axios.create();

apiClient.interceptors.request.use((config) => {
    const baseUrl = getApiUrl();

    if (config.url?.startsWith('/')) {
        config.url = `${baseUrl}${config.url}`;
    }

    return config;
});

export const apiGet = (url: string, config?: any): Promise<any> => {
    return apiClient.get(url, config);
};

export const apiGetClientDetails = async (clientId: number): Promise<any> => {
    return apiClient.get(`/api/client/${clientId}/detailed`);
};

export default apiClient;
