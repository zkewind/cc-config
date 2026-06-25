import { useState, useCallback, useRef, useEffect } from "react";
import type { ProviderCategory } from "@/types";
import type { AppId } from "@/lib/api";

interface UseBaseUrlStateProps {
  appType: AppId;
  category: ProviderCategory | undefined;
  settingsConfig: string;
  onSettingsConfigChange: (config: string) => void;
}

export function useBaseUrlState({
  appType,
  category,
  settingsConfig,
  onSettingsConfigChange,
}: UseBaseUrlStateProps) {
  const [baseUrl, setBaseUrl] = useState("");
  const isUpdatingRef = useRef(false);

  useEffect(() => {
    if (appType !== "claude") return;
    if (category === "official") return;
    if (isUpdatingRef.current) return;

    try {
      const config = JSON.parse(settingsConfig || "{}");
      const envUrl: unknown = config?.env?.ANTHROPIC_BASE_URL;
      const nextUrl = typeof envUrl === "string" ? envUrl.trim() : "";
      if (nextUrl !== baseUrl) {
        setBaseUrl(nextUrl);
      }
    } catch {
      // ignore
    }
  }, [appType, category, settingsConfig, baseUrl]);

  const handleClaudeBaseUrlChange = useCallback(
    (url: string) => {
      const sanitized = url.trim();
      setBaseUrl(sanitized);
      isUpdatingRef.current = true;

      try {
        const config = JSON.parse(settingsConfig || "{}");
        if (!config.env) {
          config.env = {};
        }
        config.env.ANTHROPIC_BASE_URL = sanitized;
        onSettingsConfigChange(JSON.stringify(config, null, 2));
      } catch {
        // ignore
      } finally {
        setTimeout(() => {
          isUpdatingRef.current = false;
        }, 0);
      }
    },
    [settingsConfig, onSettingsConfigChange],
  );

  return {
    baseUrl,
    setBaseUrl,
    handleClaudeBaseUrlChange,
  };
}
