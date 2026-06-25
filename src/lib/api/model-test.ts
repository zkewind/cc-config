import { invoke } from "@tauri-apps/api/core";
import type { AppId } from "./types";

export type HealthStatus = "operational" | "degraded" | "failed" | "checking";

export interface StreamCheckConfig {
  timeoutSecs: number;
  maxRetries: number;
  degradedThresholdMs: number;
  claudeModel: string;
  testPrompt: string;
}

export interface StreamCheckResult {
  status: HealthStatus;
  success: boolean;
  message: string;
  responseTimeMs?: number;
  httpStatus?: number;
  modelUsed: string;
  testedAt: number;
  retryCount: number;
  errorCategory?: string;
}

export async function getStreamCheckConfig(): Promise<StreamCheckConfig> {
  return await invoke("get_stream_check_config");
}

export async function saveStreamCheckConfig(
  config: StreamCheckConfig,
): Promise<void> {
  await invoke("save_stream_check_config", { config });
}

export async function streamCheckProvider(
  appType: AppId,
  providerId: string,
): Promise<StreamCheckResult> {
  return await invoke("stream_check_provider", { appType, providerId });
}
