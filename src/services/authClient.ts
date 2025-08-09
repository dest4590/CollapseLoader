import { apiClient } from './apiClient';

export const apiGet = apiClient.get.bind(apiClient);
export const apiPost = apiClient.post.bind(apiClient);
export const apiPut = apiClient.put.bind(apiClient);
export const apiPatch = apiClient.patch.bind(apiClient);
export const apiDelete = apiClient.delete.bind(apiClient);

export const apiBatchGet = apiClient.batchGet.bind(apiClient);
export const apiPreload = apiClient.preloadCriticalData.bind(apiClient);
export const apiMetrics = apiClient.getMetrics.bind(apiClient);
export const apiCacheStats = apiClient.getCacheStats.bind(apiClient);

export default apiClient;
