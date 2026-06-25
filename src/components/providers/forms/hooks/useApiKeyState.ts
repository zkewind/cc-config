import { useEffect, useState, useCallback } from "react";
import type { ProviderCategory } from "@/types";
import {
  getApiKeyFromConfig,
  setApiKeyInConfig,
  hasApiKeyField,
} from "@/utils/providerConfigUtils";

interface UseApiKeyStateProps {
  initialConfig?: string;
  onConfigChange: (config: string) => void;
  selectedPresetId: string | null;
  category?: ProviderCategory;
  apiKeyField?: string;
}

export function useApiKeyState({
  initialConfig,
  onConfigChange,
  selectedPresetId,
  category,
  apiKeyField,
}: UseApiKeyStateProps) {
  const [apiKey, setApiKey] = useState(() => {
    if (initialConfig) {
      return getApiKeyFromConfig(initialConfig);
    }
    return "";
  });

  useEffect(() => {
    if (!initialConfig) return;

    try {
      JSON.parse(initialConfig);
    } catch {
      return;
    }

    const extracted = getApiKeyFromConfig(initialConfig);
    if (extracted !== apiKey) {
      setApiKey(extracted);
    }
  }, [initialConfig, apiKey]);

  const handleApiKeyChange = useCallback(
    (key: string) => {
      setApiKey(key);

      const configString = setApiKeyInConfig(
        initialConfig || "{}",
        key.trim(),
        {
          createIfMissing:
            selectedPresetId !== null &&
            category !== undefined &&
            category !== "official",
          apiKeyField,
        },
      );

      onConfigChange(configString);
    },
    [initialConfig, selectedPresetId, category, apiKeyField, onConfigChange],
  );

  const showApiKey = useCallback(
    (config: string, isEditMode: boolean) => {
      return (
        selectedPresetId !== null || (isEditMode && hasApiKeyField(config))
      );
    },
    [selectedPresetId],
  );

  return {
    apiKey,
    setApiKey,
    handleApiKeyChange,
    showApiKey,
  };
}
