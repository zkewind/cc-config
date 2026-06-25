import { invoke } from "@tauri-apps/api/core";
import type { UsageResult } from "@/types";
import type { AppId } from "./types";

export const usageApi = {
  testScript: async (
    providerId: string,
    appId: AppId,
    scriptCode: string,
    timeout?: number,
    apiKey?: string,
    baseUrl?: string,
    accessToken?: string,
    userId?: string,
    templateType?: "custom" | "general" | "newapi",
  ): Promise<UsageResult> =>
    invoke("testUsageScript", {
      providerId,
      app: appId,
      scriptCode,
      timeout,
      apiKey,
      baseUrl,
      accessToken,
      userId,
      templateType,
    }),
};
