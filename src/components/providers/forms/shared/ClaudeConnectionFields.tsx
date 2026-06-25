import { useTranslation } from "react-i18next";
import type { ProviderCategory } from "@/types";
import { ApiKeySection } from "./ApiKeySection";
import { EndpointField } from "./EndpointField";

interface ClaudeConnectionFieldsProps {
  showApiKey?: boolean;
  showEndpoint?: boolean;
  apiKey: string;
  onApiKeyChange: (key: string) => void;
  baseUrl: string;
  onBaseUrlChange: (url: string) => void;
  category?: ProviderCategory;
  shouldShowApiKeyLink?: boolean;
  websiteUrl?: string;
  isPartner?: boolean;
  partnerPromotionKey?: string;
  apiKeyDisabled?: boolean;
  apiKeyPlaceholder?: {
    official: string;
    thirdParty: string;
  };
  endpointHint?: string;
  showEndpointTools?: boolean;
  onEndpointManageClick?: () => void;
  showFullUrlToggle?: boolean;
  isFullUrl?: boolean;
  onFullUrlChange?: (value: boolean) => void;
}

export function ClaudeConnectionFields({
  showApiKey = true,
  showEndpoint = true,
  apiKey,
  onApiKeyChange,
  baseUrl,
  onBaseUrlChange,
  category,
  shouldShowApiKeyLink = false,
  websiteUrl = "",
  isPartner,
  partnerPromotionKey,
  apiKeyDisabled,
  apiKeyPlaceholder,
  endpointHint,
  showEndpointTools = true,
  onEndpointManageClick,
  showFullUrlToggle = false,
  isFullUrl = false,
  onFullUrlChange,
}: ClaudeConnectionFieldsProps) {
  const { t } = useTranslation();

  return (
    <>
      {showApiKey ? (
        <ApiKeySection
          value={apiKey}
          onChange={onApiKeyChange}
          category={category}
          shouldShowLink={shouldShowApiKeyLink}
          websiteUrl={websiteUrl}
          placeholder={apiKeyPlaceholder}
          disabled={apiKeyDisabled}
          isPartner={isPartner}
          partnerPromotionKey={partnerPromotionKey}
        />
      ) : null}

      {showEndpoint ? (
        <EndpointField
          id="baseUrl"
          label={t("providerForm.apiEndpoint")}
          value={baseUrl}
          onChange={onBaseUrlChange}
          placeholder={t("providerForm.apiEndpointPlaceholder")}
          hint={endpointHint ?? t("providerForm.apiHint")}
          showManageButton={showEndpointTools}
          onManageClick={showEndpointTools ? onEndpointManageClick : undefined}
          showFullUrlToggle={showEndpointTools && showFullUrlToggle}
          isFullUrl={isFullUrl}
          onFullUrlChange={onFullUrlChange}
        />
      ) : null}
    </>
  );
}
