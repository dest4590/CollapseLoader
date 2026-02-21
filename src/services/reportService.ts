import { apiClient } from "./apiClient";

export interface CreateReportRequest {
    reportedUserId: number;
    reason: string;
    description?: string;
}

export interface UserReportResponse {
    id: number;
    status: string;
    createdAt: string;
}

class ReportService {
    public async createReport(
        request: CreateReportRequest
    ): Promise<UserReportResponse> {
        return apiClient.post<UserReportResponse>("/reports", request);
    }
}

export const reportService = new ReportService();
